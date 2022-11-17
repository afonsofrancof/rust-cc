use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    ops::Add,
    thread,
};

use crate::{
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::{DNSMessage, DNSSingleResponse, QueryType},
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
    let dns_message: DNSMessage = match bincode::deserialize(&buf) {
        Ok(message) => message,
        Err(_) => panic!("Could not deserialize message"),
    };
    let queried_domain = dns_message.data.query_info.name;

    let (domain,db) = match database
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
        Some((domain_name, domain_database)) => (domain_name,domain_database),
        None => panic!(
            "My domains can't answer this (need to add feature of resolver if no DD field exists )"
        ),
    };

    let subdomain_ns_list = match db.get_ns_records() {
        Some(ns_records) => match ns_records.get(&queried_domain) {
            Some(ns_list) => Some(ns_list.to_owned()),
            None => {
                println!("No NS found for {}", queried_domain);
                None
            }
        },
        None => None,
    };
}

//CONTINUE
// let query_types = dns_message.data.query_info.type_of_value.clone();

// let mut response_map: HashMap<QueryType, Vec<DNSSingleResponse>> = HashMap::new();

// for query_type in query_types.into_iter() {
//     let response = match query_type{
//         QueryType::A => queried_domain_db.a_records
//         QueryType::NS => queried_domain_db.domain_name_servers
//         QueryType::MX => queried_domain_db.mx_records
//         QueryType::CNAME => queried_domain_db.cname_records
//         QueryType::PTR => queried_domain_db.ptr_records
//     };
//     let mut response_vec = Vec::new();

//     for entry in response {
//         response_vec.push(DNSSingleResponse {
//             name: entry.name.to_owned(),
//             type_of_value: entry.entry_type.clone(),
//             value: entry.value.to_owned(),
//             ttl: entry.ttl,
//         });
//     }
//     response_map.insert(query_type, response_vec);
// }

// dns_message.data.response_values = Some(response_map);
// let addr = src_addr.ip();
// let port = src_addr.port();
// let send_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
// let destination = format!("{}:{}",addr,port);
// let _port = dns_send::send(dns_message, &send_socket,destination);
