use log::{debug, error, info};
use std::net::IpAddr;
use std::thread;
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::dns_structs::dns_domain_name::Domain;
use crate::dns_structs::server_config::ServerConfig;
use crate::dns_structs::{dns_message::DNSEntry, domain_database_struct::DomainDatabase};

pub fn db_sync_listener(db: HashMap<Domain, DomainDatabase>, config: ServerConfig) {
    let default_listener = "0.0.0.0:8000";
    let listener = match TcpListener::bind(default_listener) {
        Ok(lst) => lst,
        Err(err) => {
            error!("FL @ zone-transfer-listener-fail {}", default_listener);
            return;
        }
    };

    for stream in listener.incoming() {
        // falta fazer o check se o ss que se ta a tentar conecatar e realmente ss do dominio

        // make thread for every ss that asks for connection
        if let Ok(mut stream) = stream {
            if let Ok(incoming_addr) = stream.peer_addr() {
                if config
                    .get_all_ss()
                    .iter()
                    .map(|s| s.ip())
                    .collect::<Vec<IpAddr>>()
                    .contains(&incoming_addr.ip())
                {
                    let new_db = db.clone();
                    thread::spawn(move || db_sync_handler(&mut stream, new_db));
                } else {
                    debug!("EZ denied-zone-transfer {} SP", incoming_addr.to_string(),);
                }
            }
        } else {
            debug!("EV @ zone-transfer-tcp-fail");
        }
    }
}

fn db_sync_handler(stream: &mut TcpStream, db: HashMap<Domain, DomainDatabase>) {
    // ler dominio pedido na stream
    // enviar numero de entries da db desse dominio
    let now = Instant::now();
    let peer_addr = stream.peer_addr().unwrap();
    let mut buf = [0u8; 1000];
    let mut total_bytes_transfered = 0;
    let mut byte_num = stream.read(&mut buf).unwrap();
    total_bytes_transfered += byte_num;

    let domain_name_bin = buf[0..byte_num].to_vec();
    let domain_name = Domain::new(String::from_utf8(domain_name_bin).unwrap());

    let domain_db = match db.get(&domain_name) {
        Some(ddb) => ddb,
        None => {
            debug!("EZ {} SP", peer_addr);
            return;
        }
    };

    // enviar SERIAL
    let soas = domain_db.get_soa_records();
    let serial: u32 = soas.get_serial_value();
    let serial_buf = serial.to_ne_bytes();
    stream.write(&serial_buf).unwrap();
    // receber byte de confirmacao 0 para nao enviar, 1 caso contrario
    let mut confirm_byte = [0u8; 1];
    stream.read_exact(&mut confirm_byte).unwrap();
    total_bytes_transfered += 1;

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
    stream.write(&entry_num_bin).unwrap();

    byte_num = stream.read(&mut entry_num_bin).unwrap();
    total_bytes_transfered += byte_num;

    let _recived_entry_num = (entry_num_bin[0] as u16 * 256) + entry_num_bin[1] as u16;
    let mut seq_number: u16 = 0;
    let mut ebuf: Vec<u8> = Vec::new();
    stream.set_nodelay(true).unwrap();
    for entry in entries_to_send {
        ebuf.push((seq_number >> 8) as u8);
        ebuf.push(seq_number as u8);
        ebuf.append(&mut entry.get_string().as_bytes().to_vec());
        stream.write_all(ebuf.as_slice()).unwrap();
        stream.flush().unwrap();
        ebuf.clear();
        seq_number += 1;
        thread::sleep(Duration::new(0, 100000));
    }

    debug!(
        "ZT {} SP {} {}",
        peer_addr,
        now.elapsed().as_millis(),
        total_bytes_transfered
    );
}
