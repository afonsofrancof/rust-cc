use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    ops::Add,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use clap::*;
use my_dns::dns_components::{
    sp::{self, db_sync_listener},
    ss::db_sync,
};
use my_dns::{
    dns_make::dns_send,
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::{DNSEntry, DNSMessage, QueryType},
        domain_database_struct::DomainDatabase,
        server_config::ServerConfig,
    },
};
fn main() {
    // Argumentos de input da CLI para definir quais e quantos servidores inicializar
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("primary")
                .action(ArgAction::SetTrue)
                .long("primary")
                .group("server_type")
                .help("Creates a primary DNS server to a domain"),
            Arg::new("secondary")
                .action(ArgAction::SetTrue)
                .long("secondary")
                .group("server_type")
                .help("Creates a secondary DNS server to a domain"),
            Arg::new("resolver")
                .action(ArgAction::SetTrue)
                .long("resolver")
                .group("server_type")
                .help("Creates a DNS resolver"),
            Arg::new("config_path")
                .short('c')
                .long("config-path")
                .required(true)
                .help("Path to the configuration file for the server"),
            Arg::new("port")
                .short('p')
                .long("port")
                .required(true)
                .help("The port the server will listen on"),
        ])
        .group(
            ArgGroup::new("server_type")
                .args(["primary", "secondary", "resolver"])
                .required(true),
        )
        .get_matches();

    //test if path exists
    let config_path = match arguments.get_one::<String>("config_path") {
        Some(path) => path,
        None => panic!("No config path provided."),
    };

    let port: u16 = match arguments.get_one::<String>("port") {
        Some(port) => match port.parse() {
            Ok(ok_port) => ok_port,
            Err(err) => panic!("{err}"),
        },
        None => panic!("No port provided."),
    };

    start_server(config_path.to_string(), port);
}

pub fn start_server(config_path: String, port: u16) {
    // parsing da config
    let config: ServerConfig = match server_config_parse::get(config_path) {
        Ok(config) => config,
        Err(_err) => panic!("Server config path not found!"),
    };

    let mut database: HashMap<String, DomainDatabase> = HashMap::new();

    let mut handle_vec: Vec<JoinHandle<()>> = Vec::new();
    let mutable_db: Arc<Mutex<HashMap<String, DomainDatabase>>> = Arc::new(Mutex::new(database));

    //Add SP's to DB
    for (domain_name, domain_config) in config.get_domain_configs().iter() {
        if let Some(db) = domain_config.get_domain_db() {
            match domain_database_parse::get(db) {
                Ok(db_parsed) => mutable_db
                    .lock()
                    .unwrap()
                    .insert(domain_name.to_string(), db_parsed),
                Err(err) => panic!("{err}"),
            };
        } else {
            if let Some(sp_addr) = domain_config.get_domain_sp() {
                let dn_copy = domain_name.to_string();
                let mutable_db_copy = Arc::clone(&mutable_db);
                let handler = thread::spawn(move || db_sync(dn_copy, sp_addr, mutable_db_copy));
                handle_vec.push(handler);
            }
        }
    }

    //START SP LISTENER
    let db_clone = mutable_db.clone();
    thread::spawn(move || db_sync_listener(db_clone));

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
        let new_db = mutable_db.clone();
        let handler =
            thread::spawn(move || client_handler(buf.to_vec(), num_of_bytes, src_addr, new_db));
    }
}

fn client_handler(
    buf: Vec<u8>,
    num_of_bytes: usize,
    src_addr: SocketAddr,
    database_mutex: Arc<Mutex<HashMap<String, DomainDatabase>>>,
) {
    let mut dns_message: DNSMessage = match bincode::deserialize(&buf) {
        Ok(message) => message,
        Err(_) => panic!("Could not deserialize message"),
    };
    let mut queried_domain: String = dns_message.data.query_info.name.to_owned();
    if !queried_domain.ends_with(".") {
        queried_domain = queried_domain.add(".");
    };

    let database = database_mutex.lock().unwrap();

    let (domain_name, db) = match database
        .iter()
        .clone()
        .filter(|(domain_name, _domain_db)| {
            let dn = match domain_name.as_str() {
                "." => ".".to_string(),
                _ => ".".to_string().add(domain_name),
            };
            ".".to_string().add(&queried_domain).ends_with(&dn)
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
        println!("SubDomain:{} ,Domain:{}", sub_domain_name, domain_name);
        if sub_domain_name == domain_name.to_owned() {
            let query_types = dns_message.data.query_info.type_of_value.clone();

            let mut response_map: HashMap<QueryType, Vec<DNSEntry>> = HashMap::new();

            for query_type in query_types.into_iter() {
                let response = match query_type {
                    QueryType::A => match db.get_a_records() {
                        Some(records) => Some(
                            records
                                .iter()
                                .filter(|entry| entry.name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
                    QueryType::NS => match db.get_ns_records() {
                        Some(records) => Some(
                            records
                                .values()
                                .map(|entry| entry.to_owned())
                                // .map(|entry| entry.to_owned())
                                .flatten()
                                .collect(),
                        ),
                        None => None,
                    },
                    QueryType::MX => match db.get_mx_records() {
                        Some(records) => Some(
                            records
                                .iter()
                                .filter(|entry| entry.name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
                    QueryType::CNAME => match db.get_cname_records() {
                        Some(records) => Some(
                            records
                                .into_iter()
                                .filter(|entry| entry.name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
                    QueryType::PTR => match db.get_ptr_records() {
                        Some(records) => Some(
                            records
                                .iter()
                                .filter(|entry| entry.name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
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
