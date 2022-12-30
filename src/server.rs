use clap::*;
use log::{debug, error, info, trace, warn, LevelFilter, Log, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use my_dns::{
    dns_components::{
        sp::{self, db_sync_listener},
        ss::db_sync,
    },
    dns_structs::{dns_domain_name::Domain, dns_message},
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
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    ops::Add,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

static DEFAULT_PORT: u16 = 5353;
static DEFAULT_TIMEOUT: u16 = 20000;
static LOG_PATTERN: &str = "[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {m}{n}";

pub fn main() {
    // Argumentos de input da CLI para definir quais e quantos servidores inicializar
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("config_path")
                .short('c')
                .long("config-path")
                .required(true)
                .help("Path to the configuration file for the server"),
            Arg::new("port")
                .short('p')
                .long("port")
                .help("The port the server will listen on"),
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .help("The timeout in milliseconds to wait for query responses"),
            Arg::new("debug")
                .short('b')
                .action(ArgAction::SetTrue)
                .long("debug")
                .help("The flag to define if the server will show debug messages on stdout")
        ])
        .get_matches();

    let handle = create_logger();
    //test if path exists
    let config_path = arguments.get_one::<String>("config_path").unwrap(); 
    let port: u16 = match arguments.get_one::<String>("port") {
        Some(port) => match port.parse() {
            Ok(ok_port) => ok_port,
            Err(err) => panic!("{err}"),
        },
        None => DEFAULT_PORT,
    };
    let timeout: u16 = match arguments.get_one::<String>("timeout") {
        Some(timeout) => match timeout.parse() {
            Ok(ok_timeout) => ok_timeout,
            Err(err) => panic!("{err}"),
        },
        None => DEFAULT_TIMEOUT,
    };
    let debug_mode: bool = arguments.get_flag("debug");
    
    // parsing da config
    let config: ServerConfig = match server_config_parse::get(config_path.to_string()) {
        Ok(config) => {
            info!("ST 127.0.0.1 {} {} {}", port, timeout, debug_mode);
            config
        }
        Err(_err) => panic!("Server config path not found!"),
    };

    let all_log_path = config.get_all_log();
    handle.set_config(create_logger_config(all_log_path,debug_mode));

    start_server(config, port, false);
}

fn create_logger_config(log_path: String, debug: bool) -> log4rs::Config {
    let logging_pattern = PatternEncoder::new(LOG_PATTERN);
    // Logging
    let mut level = LevelFilter::Info;
    if debug {
        level = LevelFilter::Debug
    }
    let file_path = log_path;

    // Build a stderr logger.
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(logging_pattern.to_owned()))
        .target(Target::Stdout)
        .build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        .encoder(Box::new(logging_pattern))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stdout", Box::new(stdout)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stdout")
                .build(LevelFilter::Trace),
        )
        .unwrap();
    config
}

fn create_logger() -> log4rs::Handle {
    let logging_pattern = PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {m}{n}");
    // Logging
    let level = LevelFilter::Debug;

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(logging_pattern.to_owned()))
        .target(Target::Stdout)
        .build();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stdout", Box::new(stdout)),
        )
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let handle = log4rs::init_config(config).unwrap();
    return handle;
}

pub fn start_server(config: ServerConfig, port: u16, once: bool) {
    let mut database: HashMap<Domain, DomainDatabase> = HashMap::new();

    let domain_configs = config.get_domain_configs();

    //Add SP's to DB
    for (domain_name, domain_config) in domain_configs.iter() {
        if let Some(db) = domain_config.get_domain_db() {
            match domain_database_parse::get(db.to_owned()) {
                Ok(db_parsed) => {
                    info!("EV @ db-file-read {}", db);
                    database.insert(Domain::new(domain_name.to_string()), db_parsed);
                }
                Err(err) => {
                    error!("SP @ db-file-read-fail {}", domain_name.to_string());
                    return;
                }
            };
        }
    }

    //START SP LISTENER
    let config_clone = config.clone();
    let db_clone = database.clone();
    thread::spawn(move || db_sync_listener(db_clone, config_clone));

    let mut handle_vec: Vec<JoinHandle<()>> = Vec::new();
    let mutable_db: Arc<Mutex<HashMap<Domain, DomainDatabase>>> = Arc::new(Mutex::new(database));

    //Add SS to DB
    for (domain_name, domain_config) in domain_configs.iter() {
        if let Some(sp_addr) = domain_config.get_domain_sp() {
            let dn = Domain::new(domain_name.to_string());
            let mutable_db_copy = Arc::clone(&mutable_db);
            let handler = thread::spawn(move || db_sync(dn, sp_addr, mutable_db_copy));
            handle_vec.push(handler);
        }
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
        let new_db = mutable_db.clone();
        let config_clone = config.to_owned();
        let _handler = thread::spawn(move || {
            client_handler(buf.to_vec(), num_of_bytes, src_addr, config_clone, new_db)
        });
        if once {
            break;
        };
    }
}

fn client_handler(
    buf: Vec<u8>,
    num_of_bytes: usize,
    src_addr: SocketAddr,
    config: ServerConfig,
    database_mutex: Arc<Mutex<HashMap<Domain, DomainDatabase>>>,
) {
    let mut dns_message: DNSMessage = match bincode::deserialize::<DNSMessage>(&buf) {
        Ok(message) => message,
        Err(_) => {
            error!("ER {} pdu-deserialize-fail", src_addr.ip());
            return;
        }
    };
    let mut write_log: bool = false;
    let mut queried_domain: Domain = dns_message.data.query_info.name.to_owned();
    if let Some(path) = config.get_domain_configs().get(&queried_domain) {
        write_log = true;
    }

    info!("QR {} {}", src_addr.ip(), dns_message.get_string());

    let database_map = database_mutex.lock().unwrap();
    //Go to our database_map and get the domain specific database
    let (domain_name, domain_name_db) = match database_map
        .iter()
        .clone()
        .filter(|(dn, _domain_db)| queried_domain.is_subdomain_of(dn.to_owned()))
        .max_by(|(domain_name1, _domain_db1), (domain_name2, _domain_db2)| {
            domain_name1.to_string().cmp(&domain_name2.to_string())
        }) {
        Some((domain_name, domain_database)) => (domain_name, domain_database),
        None => {
            info!("EV @ response-not-found {}", queried_domain.to_string());
            return;
            //panic!("My domains can't answer this (need to add feature of resolver if no DD field exists )")
        }
    };
    //Get NS (or the closest) of the queried domain
    if let Some((domain_name_authority, authority_ns_list)) =
        domain_name_db.get_ns_of(queried_domain.to_owned())
    {
        //Check if we are that NS
        if domain_name_authority == domain_name.to_owned() {
            let query_types = dns_message.data.query_info.type_of_value.clone();

            let mut response_map: HashMap<QueryType, Vec<DNSEntry>> = HashMap::new();

            for query_type in query_types.into_iter() {
                let response = match query_type {
                    QueryType::A => match domain_name_db.get_a_records() {
                        Some(records) => Some(
                            records
                                .iter()
                                .filter(|entry| entry.domain_name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
                    QueryType::NS => {
                        let records = domain_name_db.get_ns_records();
                        Some(
                            records
                                .values()
                                .map(|entry| entry.to_owned())
                                // .map(|entry| entry.to_owned())
                                .flatten()
                                .collect(),
                        )
                    }
                    QueryType::MX => match domain_name_db.get_mx_records() {
                        Some(records) => Some(
                            records
                                .iter()
                                .filter(|entry| entry.domain_name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
                    QueryType::CNAME => match domain_name_db.get_cname_records() {
                        Some(records) => Some(
                            records
                                .into_iter()
                                .filter(|entry| entry.domain_name == queried_domain)
                                .map(|entry| entry.to_owned())
                                .collect::<Vec<DNSEntry>>(),
                        ),
                        None => None,
                    },
                    QueryType::PTR => match domain_name_db.get_ptr_records() {
                        Some(records) => Some(
                            records
                                .iter()
                                .filter(|entry| entry.domain_name == queried_domain)
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
            dns_message.header.flags = 1;
        } else {
            dns_message.header.response_code = Some(1);
        }
        let mut authorities_values = Vec::new();
        for entry in authority_ns_list.iter().map(|entry| entry.to_owned()) {
            authorities_values.push(entry)
        }
        dns_message.data.authorities_values = Some(authorities_values.to_owned());
        dns_message.header.number_of_authorities = match authorities_values.len().try_into() {
            Ok(num) => Some(num),
            Err(err) => panic!("{err}"),
        };

        let mut extra_values = Vec::new();
        let a_records = match domain_name_db.get_a_records() {
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
            if let Some(record) = a_records
                .iter()
                .find(|a_entry| a_entry.domain_name == Domain::new(entry.value.to_string()))
            {
                a_record = record.to_owned();
                extra_values.push(DNSEntry {
                    domain_name: a_record.domain_name,
                    type_of_value: a_record.type_of_value,
                    value: a_record.value,
                    ttl: a_record.ttl,
                    priority: a_record.priority,
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
        println!("Entered Else");
        //FAZER RESOLVE DEPENDENDO DOS CAMPOS DD
    }

    let addr = src_addr.ip();
    let port = src_addr.port();
    let send_socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(_) => {
            debug!("EV @ client-handler-socket-fail");
            return;
        }
    };
    let destination = format!("{}:{}", addr, port);
    let _num_sent_bytes =
        match dns_send::send(dns_message.to_owned(), &send_socket, destination.to_owned()) {
            Ok(num_bytes) => {
                info!("RP {} {}", destination, dns_message.get_string());
                num_bytes
            }
            Err(_err) => {
                info!("EV @ send-message-fail");
                return;
            }
        };
}
