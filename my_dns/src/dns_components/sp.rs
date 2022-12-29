use std::net::IpAddr;
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::dns_structs::dns_domain_name::Domain;
use crate::dns_structs::server_config::{self, ServerConfig};
use crate::dns_structs::{dns_message::DNSEntry, domain_database_struct::DomainDatabase};

pub fn db_sync_listener(db: HashMap<Domain, DomainDatabase>, config: ServerConfig) {
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(lst) => lst,
        Err(err) => panic!("Couldn't bind tcp listener"),
    };

    for stream in listener.incoming() {
        // falta fazer o check se o ss que se ta a tentar conecatar e realmente ss do dominio

        // make thread for every ss that asks for connection
        if let Ok(mut stream) = stream {
            if let Ok(incoming_addr) = stream.peer_addr() {
                if config.get_all_ss().iter().map(|s| s.ip()).collect::<Vec<IpAddr>>().contains(&incoming_addr.ip()) {
                    let new_db = db.clone();
                    thread::spawn(move || db_sync_handler(&mut stream, new_db));
                } else {
                    println!("Denied zone transfer for {}", incoming_addr);
                }
            }
        } else {
            println!("Couldn't connect to incoming tcp stream");
        }
    }
}

fn db_sync_handler(stream: &mut TcpStream, db: HashMap<Domain, DomainDatabase>) {
    // ler dominio pedido na stream
    // enviar numero de entries da db desse dominio
    let mut buf = [0u8; 1000];
    let byte_num = match stream.read(&mut buf) {
        Ok(bytes) => bytes,
        Err(err) => panic!("{err}"),
    };
    let domain_name_bin = buf[0..byte_num].to_vec();
    let domain_name = Domain::new(String::from_utf8(domain_name_bin).unwrap());

    let domain_db = match db.get(&domain_name) {
        Some(ddb) => ddb,
        None => panic!("Database not found for {}", domain_name.to_string()),
    };

    // enviar SERIAL
    let soas = domain_db.get_soa_records();
    let serial: u32 = soas.get_serial_value();
    let serial_buf = serial.to_ne_bytes();
    stream.write(&serial_buf);
    // receber byte de confirmacao 0 para nao enviar, 1 caso contrario
    let mut confirm_byte = [0u8; 1];
    stream.read_exact(&mut confirm_byte);
    if confirm_byte[0] == 0u8 {
        return;
    }

    let mut entries_to_send: Vec<DNSEntry> = Vec::new();
    // get all SOA

    entries_to_send.push(soas.primary_ns);
    entries_to_send.push(soas.contact_email);
    entries_to_send.push(soas.serial);
    entries_to_send.push(soas.refresh);
    entries_to_send.push(soas.retry);
    entries_to_send.push(soas.expire);

    // get all ns entries
    for ns_records in domain_db.get_ns_records().values() {
        for entry in ns_records {
            entries_to_send.push(entry.to_owned());
        }
    }

    // get all A record
    if let Some(a_records) = domain_db.get_a_records() {
        for entry in a_records {
            entries_to_send.push(entry);
        }
    }

    // get all A record
    if let Some(cname_records) = domain_db.get_cname_records() {
        for entry in cname_records {
            entries_to_send.push(entry);
        }
    }

    // get all mx records
    if let Some(mx_records) = domain_db.get_mx_records() {
        for entry in mx_records {
            entries_to_send.push(entry);
        }
    }

    // get all ptr records
    if let Some(ptr_records) = domain_db.get_ptr_records() {
        for entry in ptr_records {
            entries_to_send.push(entry);
        }
    }
    // to string em todas as entries
    // sequence number u16 antes de enviar
    let entry_num: u16 = entries_to_send.len().try_into().unwrap();

    let mut entry_num_bin = [0u8, 2];
    entry_num_bin[0] = (entry_num >> 8) as u8;
    entry_num_bin[1] = entry_num as u8;
    stream.write(&entry_num_bin);

    stream.read(&mut entry_num_bin).unwrap();

    let _recived_entry_num = (entry_num_bin[0] as u16 * 256) + entry_num_bin[1] as u16;
    let mut seq_number: u16 = 0;
    let mut ebuf: Vec<u8> = Vec::new();
    stream.set_nodelay(true).unwrap();
    for entry in entries_to_send {
        println!(
            "{} {} {} {}",
            entry.domain_name.to_string(), entry.type_of_value, entry.value, entry.ttl
        );
        ebuf.push((seq_number >> 8) as u8);
        ebuf.push(seq_number as u8);
        ebuf.append(&mut entry.get_string().as_bytes().to_vec());
        stream.write_all(ebuf.as_slice()).unwrap();
        stream.flush().unwrap();
        ebuf.clear();
        seq_number += 1;
        thread::sleep(Duration::new(0, 100000));
    }
}
