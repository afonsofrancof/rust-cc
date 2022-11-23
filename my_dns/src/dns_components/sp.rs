use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    ops::Add,
    thread, io::{Read, Write},
};

use crate::{
    dns_make::dns_send,
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::{DNSEntry, DNSMessage, QueryType},
        domain_database_struct::DomainDatabase,
        server_config::ServerConfig,
    },
};

pub fn start_sp(config_path: String, port: u16) {
    // parsing da config
    let config: ServerConfig = match server_config_parse::get(config_path) {
        Ok(config) => config,
        Err(_err) => panic!("Server config path not found!"),
    };

    let mut database: HashMap<String, DomainDatabase> = HashMap::new();

    for (domain_name, domain_config) in config.get_domain_configs().iter() {
        let db_path = match domain_config.get_domain_db() {
            Some(db) => db,
            None => {
                println!("No DB entry found for domain {domain_name} in the config file");
                continue;
            }
        };
        let db = match domain_database_parse::get(db_path) {
            Ok(db_parsed) => database.insert(domain_name.to_string(), db_parsed),
            Err(err) => panic!("{err}"),
        };
    }

    let socket = match UdpSocket::bind(format!("0.0.0.0:{port}",)) {
        Ok(socket) => socket,
        Err(_) => panic!("Could not bind socket"),
    };

    let mut buf = [0; 1000];

    loop {
        let (num_of_bytes, src_addr) = match socket.recv_from(&mut buf) {
            Ok(size_and_addr) => size_and_addr,
            Err(_) => panic!("Could not receive on socket"),
        };
        let new_db = database.to_owned();
        let handler =
            thread::spawn(move || client_handler(buf.to_vec(), num_of_bytes, src_addr, new_db));
    }
}

fn client_handler(
    buf: Vec<u8>,
    num_of_bytes: usize,
    src_addr: SocketAddr,
    database: HashMap<String, DomainDatabase>,
) {
    let mut dns_message: DNSMessage = match bincode::deserialize(&buf) {
        Ok(message) => message,
        Err(_) => panic!("Could not deserialize message"),
    };
    let mut queried_domain: String = dns_message.data.query_info.name.to_owned();
    if !queried_domain.ends_with(".") {
        queried_domain = queried_domain.add(".");
    };

    let (domain_name, db) = match database
        .iter()
        .clone()
        .filter(|(domain_name, _domain_db)| {
            ".".to_string()
                .add(&queried_domain)
                .ends_with(&".".to_string().add(domain_name))
        })
        .max_by(|(domain_name1, _domain_db1), (domain_name2, _domain_db2)| {
            domain_name1.cmp(domain_name2)
        }) {
        Some((domain_name, domain_database)) => (domain_name, domain_database),
        None => panic!(
            "My domains can't answer this (need to add feature of resolver if no DD field exists )"
        ),
    };

    if let Some((sub_domain_name, subdomain_ns_list)) = db.get_ns_of(queried_domain.to_owned()) {
        if sub_domain_name == domain_name.to_owned() {
            let query_types = dns_message.data.query_info.type_of_value.clone();

            let mut response_map: HashMap<QueryType, Vec<DNSEntry>> = HashMap::new();

            for query_type in query_types.into_iter() {
                let response = match query_type {
                    QueryType::A => db.get_a_records(),
                    QueryType::NS => match db.get_ns_records() {
                        Some(records) => Some(
                            records
                                .values()
                                .map(|entry| entry.to_owned())
                                .map(|entry| entry.to_owned())
                                .flatten()
                                .filter(|entry| entry.name == queried_domain)
                                .collect(),
                        ),
                        None => None,
                    },
                    QueryType::MX => db.get_mx_records(),
                    QueryType::CNAME => db.get_cname_records(),
                    QueryType::PTR => db.get_ptr_records(),
                };
                println!("Got values from DB");
                let mut response_vec = Vec::new();

                match response {
                    Some(res) => {
                        for entry in res {
                            response_vec.push(entry);
                        }
                        response_map.insert(query_type, response_vec);
                    }
                    None => {
                        println!(
                            "No entries found for requested type {}",
                            query_type.get_string()
                        )
                    }
                }
            }

            dns_message.header.response_code = match response_map.len() {
                0 => Some(2),
                _ => {
                    dns_message.data.response_values = Some(response_map.to_owned());
                    dns_message.header.number_of_values = match response_map
                        .values()
                        .flatten()
                        .collect::<Vec<_>>()
                        .len()
                        .try_into()
                    {
                        Ok(num) => Some(num),
                        Err(err) => panic!("{err}"),
                    };
                    Some(0)
                }
            };
        } else {
            dns_message.header.response_code = Some(1);
        }
        let mut authorities_values = Vec::new();
        for entry in subdomain_ns_list.iter().map(|entry| entry.to_owned()) {
            authorities_values.push(entry)
        }
        dns_message.data.authorities_values = Some(authorities_values.to_owned());
        dns_message.header.number_of_authorities = match authorities_values.len().try_into() {
            Ok(num) => Some(num),
            Err(err) => panic!("{err}"),
        };

        let mut extra_values = Vec::new();
        let a_records = match db.get_a_records() {
            Some(records) => records,
            None => panic!("No A records found, cannot get IP of an NS entry"),
        };

        let non_extra_values = match dns_message.data.response_values {
            Some(ref values) => {
                let mut all_vals = values
                    .values()
                    .map(|val| val.to_owned())
                    .flatten()
                    .collect::<Vec<DNSEntry>>();
                all_vals.append(&mut authorities_values.clone());
                all_vals
            }
            None => authorities_values.clone(),
        };

        for entry in non_extra_values {
            let a_record: DNSEntry;
            if let Some(record) = a_records.iter().find(|a_entry| a_entry.name == entry.value) {
                a_record = record.to_owned();
                extra_values.push(DNSEntry {
                    name: a_record.name,
                    type_of_value: a_record.type_of_value,
                    value: a_record.value,
                    ttl: a_record.ttl,
                    priority: None,
                })
            } else {
                println!("No translate found. need to fix this part of the code");
            };
        }
        dns_message.data.extra_values = Some(extra_values.to_owned());
        dns_message.header.number_of_extra_values = match extra_values.len().try_into() {
            Ok(num) => Some(num),
            Err(err) => panic!("{err}"),
        };
    } else {
        //FAZER RESOLVE DEPENDENDO DOS CAMPOS DD
    }

    let addr = src_addr.ip();
    let port = src_addr.port();
    let send_socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(_) => panic!("Could not bind response socket"),
    };
    let destination = format!("{}:{}", addr, port);
    let _num_sent_bytes = match dns_send::send(dns_message, &send_socket, destination) {
        Ok(num_bytes) => num_bytes,
        Err(err) => panic!("{err}"),
    };
}

fn db_sync_listener(db: HashMap<String, DomainDatabase>) {
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(lst) => lst,
        Err(err) => panic!("Couldn't bind tcp listener"),
    };
 
    for stream in listener.incoming() {
        // falta fazer o check se o ss que se ta a tentar conecatar e realmente ss do dominio
        // make thread for every ss that asks for connection
    }
}

fn db_sync_handler(stream: TcpStream, db: HashMap<String, DomainDatabase>) {
    // ler dominio pedido na stream
    // enviar numero de entries da db desse dominio
    let mut buf = [0u8;1000];
    stream.try_clone().unwrap().read(&mut buf);
    let domain_name_bin = buf.clone().to_vec();
    let domain_name = String::from_utf8(domain_name_bin).unwrap();
    let domain_db = match db.get(&domain_name.to_owned()) {
        Some(ddb) => ddb, 
        None => panic!("Database not found for {}", domain_name.to_owned())
    };

    let mut entries_to_send: Vec<DNSEntry> = Vec::new();
    // get all SOA
    for entry in domain_db.get_config_list().values() {
        entries_to_send.push(entry.to_owned());
    }
    
    // get all ns entries
    for ns_records in domain_db.get_ns_records().unwrap().values() {
        for entry in ns_records{
            entries_to_send.push(entry.to_owned());
        } 
    }
    
    // get all A record
    if let Some(a_records) = domain_db.get_a_records(){
        for entry in a_records {
            entries_to_send.push(entry);
        }
    }
    
    // get all A record
    if let Some(cname_records) = domain_db.get_cname_records(){
        for entry in cname_records {
            entries_to_send.push(entry);
        }
    }
   
    // get all mx records
    if let Some(mx_records) = domain_db.get_mx_records(){
        for entry in mx_records {
            entries_to_send.push(entry);
        }
    }

    // get all ptr records 
    if let Some(ptr_records) = domain_db.get_ptr_records(){
        for entry in ptr_records {
            entries_to_send.push(entry);
        }
    }
    // to string em todas as entries
    // sequence number u16 antes de enviar
    let entry_num: u16 = entries_to_send.len().try_into().unwrap();

    let mut entry_num_bin = [0u8,2];
    entry_num_bin[0] = (entry_num >> 8) as u8;
    entry_num_bin[1] = entry_num as u8;
    stream.try_clone().unwrap().write(&mut entry_num_bin);
    
    stream.try_clone().unwrap().read(&mut entry_num_bin).unwrap();
    
    let _recived_entry_num = (entry_num_bin[0] as u16 * 256) + entry_num_bin[1] as u16;
    let mut seq_number: u16 = 0;
    for entry in entries_to_send {
        let mut ebuf: Vec<u8> = Vec::new();
        ebuf.push((seq_number >> 8) as u8);
        ebuf.push(seq_number as u8);
        ebuf.append(&mut entry.get_string().as_bytes().to_vec());
        stream.try_clone().unwrap().write(ebuf.as_slice()).unwrap();
    }
}
