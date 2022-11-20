use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    ops::Add,
    thread,
};

use crate::{
    dns_make::dns_send,
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::{DNSMessage, DNSSingleResponse, QueryType},
        domain_database_struct::{DomainDatabase, Entry},
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

    let socket = match UdpSocket::bind(format!("127.0.0.1:{port}",)) {
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

    if let Some((sub_domain_name, subdomain_ns_list)) = db.get_ns_of(queried_domain) {
        if sub_domain_name == domain_name.to_owned() {
            let query_types = dns_message.data.query_info.type_of_value.clone();

            let mut response_map: HashMap<QueryType, Vec<DNSSingleResponse>> = HashMap::new();

            for query_type in query_types.into_iter() {
                let response = match query_type {
                    QueryType::A => db.get_a_records(),
                    QueryType::NS => match db.get_ns_records() {
                        Some(records) => Some(
                            records
                                .values()
                                .map(|entry| entry.to_owned())
                                .flatten()
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
                            response_vec.push(DNSSingleResponse {
                                name: entry.name.to_owned(),
                                type_of_value: entry.entry_type.clone(),
                                value: entry.value.to_owned(),
                                ttl: entry.ttl,
                            });
                        }
                        response_map.insert(query_type, response_vec);
                    }
                    None => {
                        println!(
                            "No entries found for requested type {}",
                            query_type.to_string()
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
            authorities_values.push(DNSSingleResponse {
                name: entry.name,
                type_of_value: entry.entry_type,
                value: entry.value,
                ttl: entry.ttl,
            })
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
                    .collect::<Vec<DNSSingleResponse>>();
                all_vals.append(&mut authorities_values.clone());
                all_vals
            }
            None => authorities_values.clone(),
        };

        for entry in non_extra_values {
            let a_record: Entry = match a_records.iter().find(|entry| entry.name == entry.value) {
                Some(record) => record.to_owned(),
                None => panic!("No A record found for entry value {}", entry.value),
            };
            extra_values.push(DNSSingleResponse {
                name: a_record.name,
                type_of_value: a_record.entry_type,
                value: a_record.value,
                ttl: a_record.ttl,
            })
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
    let send_socket = match UdpSocket::bind("127.0.0.1:0") {
        Ok(socket) => socket,
        Err(_) => panic!("Could not bind response socket"),
    };
    let destination = format!("{}:{}", addr, port);
    let _num_sent_bytes = match dns_send::send(dns_message, &send_socket, destination) {
        Ok(num_bytes) => num_bytes,
        Err(err) => panic!("{err}"),
    };
}
