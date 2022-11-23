use std::sync::Mutex;
use std::thread;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use crate::dns_structs::{dns_message::DNSEntry, domain_database_struct::DomainDatabase};

pub fn db_sync_listener(db: Arc<Mutex<HashMap<String, DomainDatabase>>>) {
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(lst) => lst,
        Err(err) => panic!("Couldn't bind tcp listener"),
    };

    for stream in listener.incoming() {
        // falta fazer o check se o ss que se ta a tentar conecatar e realmente ss do dominio
        // make thread for every ss that asks for connection
        if let Ok(mut stream) = stream {
            let db_clone = db.clone();
            thread::spawn(move || db_sync_handler(&mut stream, db_clone));
        } else {
            println!("Couldn't connect to incoming tcp stream");
        }
    }
}

fn db_sync_handler(stream: &mut TcpStream, db: Arc<Mutex<HashMap<String, DomainDatabase>>>) {
    // ler dominio pedido na stream
    // enviar numero de entries da db desse dominio
    let mut buf = [0u8; 1000];
    let byte_num = match stream.read(&mut buf) {
        Ok(bytes) => bytes,
        Err(err) => panic!("{err}"),
    };
    let domain_name_bin = buf[0..byte_num].to_vec();
    let domain_name = String::from_utf8(domain_name_bin).unwrap();
    println!("Lenght socket:{}", domain_name.len());

    let db_lock = db.lock().unwrap();
    let domain_db = match db_lock.get(&domain_name) {
        Some(ddb) => ddb,
        None => panic!("Database not found for {}", domain_name.to_owned()),
    };

    let mut entries_to_send: Vec<DNSEntry> = Vec::new();
    // get all SOA
    for entry in domain_db.get_config_list().values() {
        entries_to_send.push(entry.to_owned());
    }

    // get all ns entries
    for ns_records in domain_db.get_ns_records().unwrap().values() {
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
    stream.write(&mut entry_num_bin);

    stream.read(&mut entry_num_bin).unwrap();

    let _recived_entry_num = (entry_num_bin[0] as u16 * 256) + entry_num_bin[1] as u16;
    let mut seq_number: u16 = 0;
    for entry in entries_to_send {
        println!(
            "{} {} {} {}",
            entry.name, entry.type_of_value, entry.value, entry.ttl
        );
        let mut ebuf: Vec<u8> = Vec::new();
        ebuf.push((seq_number >> 8) as u8);
        ebuf.push(seq_number as u8);
        ebuf.append(&mut entry.get_string().as_bytes().to_vec());

        stream.write(ebuf.as_slice()).unwrap();
        stream.flush().unwrap();
        seq_number += 1;
    }
}
