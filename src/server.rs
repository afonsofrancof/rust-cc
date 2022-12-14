use chrono::{DateTime, Utc};
use clap::*;
use log::{debug, error, info, LevelFilter};
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
    dns_components::{sp::db_sync_listener, sr::resolver, ss::db_sync},
    dns_parse::domain_database_parse::parse_root_servers,
    dns_structs::{dns_domain_name::Domain, server_config::DomainConfig},
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
    fs::File,
    io::Write,
    net::{SocketAddr, UdpSocket},
    ops::Add,
    os::linux::fs,
    path::Path,
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
            Arg::new("supports_recursive")
                .short('r')
                .action(ArgAction::SetTrue)
                .help("The flag to define if the server supports recursive queries"),
            Arg::new("debug")
                .short('b')
                .action(ArgAction::SetTrue)
                .long("debug")
                .help("The flag to define if the server will show debug messages on stdout"),
        ])
        .get_matches();

    let debug_mode: bool = arguments.get_flag("debug");
    let mut debug_mode_string = "shy";
    let mut level = LevelFilter::Info;
    if debug_mode {
        level = LevelFilter::Debug;
        debug_mode_string = "debug"
    }

    let handle = create_logger(level);
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

    let supports_recursive = arguments.get_flag("supports_recursive");

    // parsing da config
    let config: ServerConfig = match server_config_parse::get(config_path.to_string()) {
        Ok(config) => {
            info!(
                "ST 127.0.0.1 {} {} {} supports-recursive {}",
                port, timeout, debug_mode_string, supports_recursive
            );
            config
        }
        Err(_err) => panic!("Server config path not found!"),
    };

    let all_log_path = config.get_all_log();
    handle.set_config(create_logger_config(all_log_path, level));

    start_server(config, port, supports_recursive, false);
}

pub fn start_server(config: ServerConfig, port: u16, supports_recursive: bool, once: bool) {
    //Global variables
    let mut database: HashMap<Domain, DomainDatabase>;
    let domain_configs: HashMap<Domain, DomainConfig>;

    database = HashMap::new();

    domain_configs = config.get_domain_configs();

    //Add SP's to DB
    for (domain_name, domain_config) in domain_configs.clone().iter() {
        if let Some(db) = domain_config.get_domain_db() {
            match domain_database_parse::get(db.to_owned()) {
                Ok(db_parsed) => {
                    info!("EV @ db-file-read {}", db);
                    database.insert(Domain::new(domain_name.to_string()), db_parsed);
                }
                Err(_err) => {
                    error!("SP @ db-file-read-fail {}", domain_name.to_string());
                    return;
                }
            };
        }
    }

    //START SP LISTENER
    let config_clone = config.clone();
    let db_clone = database.clone();
    debug!("EV @ initalizing-db-sync-listener");
    thread::spawn(move || db_sync_listener(db_clone, config_clone));

    let mut handle_vec: Vec<JoinHandle<()>> = Vec::new();
    let mutable_db: Arc<Mutex<HashMap<Domain, DomainDatabase>>> = Arc::new(Mutex::new(database));

    //Add SS to DB
    for (domain_name, domain_config) in domain_configs.iter() {
        if let Some(sp_addr) = domain_config.get_domain_sp() {
            let mutable_db_copy = Arc::clone(&mutable_db);
            debug!("EV @ initializing-ss-thread {}", domain_name.to_string());
            let dn = domain_name.clone();
            let handler = thread::spawn(move || db_sync(dn, sp_addr, mutable_db_copy));
            handle_vec.push(handler);
        }
    }
    let socket = match UdpSocket::bind(format!("0.0.0.0:{port}",)) {
        Ok(socket) => socket,
        Err(_) => {
            error!("SP @ udp-listen-socket-fail");
            panic!("Could not bind socket")
        }
    };

    let mut buf = [0; 1000];

    loop {
        let (_, src_addr) = match socket.recv_from(&mut buf) {
            Ok(size_and_addr) => size_and_addr,
            Err(_) => {
                error!("SP @ udp-socket-receive-fail");
                panic!("Could not receive on socket")
            }
        };
        let new_db = mutable_db.clone();
        let config_clone = config.clone();
        let _handler = thread::spawn(move || {
            client_handler(
                buf.to_vec(),
                src_addr,
                config_clone,
                supports_recursive,
                new_db,
            )
        });
        if once {
            break;
        };
    }
}

fn client_handler(
    buf: Vec<u8>,
    src_addr: SocketAddr,
    config: ServerConfig,
    supports_recursive: bool,
    database_mutex: Arc<Mutex<HashMap<Domain, DomainDatabase>>>,
) {
    let mut dns_message: DNSMessage = match bincode::deserialize::<DNSMessage>(&buf) {
        Ok(message) => message,
        Err(_) => {
            error!("ER pdu-deserialize-fail {}", src_addr.ip());
            let mut response = DNSMessage::new();
            response.header.response_code = Some(3);
            // enviar dns message com codigo 3 !!!!!!!!
            send_answer(response, src_addr);
            return;
        }
    };

    let queried_timestamp = format!("{}", Utc::now().format("%Y-%m-%d %H:%M:%S %Z"));
    let queried_message_string = dns_message.get_string();
    let queried_domain: Domain = dns_message.data.query_info.name.to_owned();

    info!("QR {} {}", src_addr.ip(), dns_message.get_string());

    //Get list of root servers
    let root_servers_path: String = config.get_st_db();
    let root_servers: Vec<SocketAddr> = match parse_root_servers(root_servers_path) {
        Ok(roots) => roots,
        Err(err) => panic!("{err}"),
    };

    // Acquire a lock on the database
    let mut database_map = database_mutex.lock().unwrap();

    //Check if there is any parent domain of the queried domain on our database
    let is_parent_cached = database_map
        .iter()
        .any(|(dn, _)| queried_domain.is_subdomain_of(dn));

    if is_parent_cached {
        //Parent Domain is in our database
        debug!(
            "EV @ parent-domain-is-cached {}",
            queried_domain.to_string()
        );
        //Find the database for the highest level parent domain
        let db_clone = database_map.clone();
        let (parent_domain_name, parent_db) = db_clone
            .iter()
            .clone()
            .filter(|(dn, _domain_db)| queried_domain.is_subdomain_of(dn.to_owned()))
            .max_by(|(domain_name1, _domain_db1), (domain_name2, _domain_db2)| {
                domain_name1.to_string().cmp(&domain_name2.to_string())
            })
            .unwrap();
        // Get the type of query being made
        let query_type = dns_message.data.query_info.type_of_value.clone();

        // Check if we have an answer for the queried domain in our parent domain's database
        let response_vec = parent_db.get_domain_query(query_type, queried_domain.to_owned());

        let parent_db_has_answer = response_vec.is_some();

        // Check if we are the parent domain's authority
        let am_parent_authority = parent_db.am_i_authority();

        if parent_db_has_answer {
            debug!("EV @ parent-db-has-answer {}", queried_domain.to_string());
            // We have an answer in our DB, so set the response values and update the number of
            // values
            dns_message.data.response_values = response_vec.to_owned();
            dns_message.header.number_of_values = match response_vec {
                Some(vec) => Some(vec.len().try_into().unwrap()),
                None => panic!("Response purged from database mid query"),
            };

            // Add all authority values for the parent domain
            dns_message.data.authorities_values =
                parent_db.ns_records.get(parent_domain_name).cloned();
            dns_message.header.number_of_authorities = match dns_message.data.authorities_values {
                Some(ref vec) => Some(vec.len().try_into().unwrap()),
                None => None,
            };
            //Set flags and response code
            dns_message.header.response_code = Some(0);
        } else {
            debug!(
                "EV @ parent-db-doesnt-have-answer {}",
                queried_domain.to_string()
            );
            // We don't have an answer in our DB

            //Check if any of the parents domain subdomains can be/know the authority of the queried domain
            //The returned value is the lowest possible parent domain. It can be ourselves.
            let queried_domain_ns = parent_db.get_ns_of(queried_domain.to_owned());

            //Check if the value returned is ourselves
            //If it is ourselves, then the query values doesn't exist.
            let lower_authority_exists = match queried_domain_ns {
                Some(ref ns_vec) => {
                    !ns_vec.is_empty()
                        && ns_vec
                            .iter()
                            .all(|ns| ns.domain_name != parent_domain_name.to_owned())
                }
                None => false,
            };
            //Reply back to user with authorities_values and extra_values
            dns_message.data.authorities_values = queried_domain_ns.to_owned();
            dns_message.header.number_of_authorities = match dns_message.data.authorities_values {
                Some(ref vec) => Some(vec.len().try_into().unwrap()),
                None => None,
            };

            if lower_authority_exists {
                debug!("EV @ lower-authority-exists {}", queried_domain.to_string());
                //This means that we don't know if the entry exists on some other authority lower
                //than us. In this case, we set the response_code to 1.

                dns_message.header.response_code = Some(1);
                //Check if the query received is recursive and if so get the answer with the SR;
                //Call SR with serverlist IP being the auth values.
                if dns_message.header.flags == 6 {
                    //Recursive
                    //Call SR
                    match queried_domain_ns {
                        Some(ns_vec) => {
                            let ip_vec = match DNSMessage::get_authorities_ip(
                                &dns_message,
                                parent_db.get_a_records(),
                                queried_domain,
                                ns_vec,
                            ) {
                                Some(vec) => vec,
                                None => {
                                    panic!("No NS found for the queried domain, cannot get answer")
                                }
                            };
                            let dns_response =
                                match resolver(&mut dns_message, ip_vec, supports_recursive) {
                                    Ok(message) => message,
                                    Err(err) => panic!("{err}"),
                                };
                            match database_map.contains_key(&dns_response.data.query_info.name) {
                                true => database_map
                                    .get_mut(&dns_response.data.query_info.name)
                                    .unwrap()
                                    .add_dns_message(dns_response.clone()),
                                false => {
                                    let mut db = DomainDatabase::new();
                                    db.add_dns_message(dns_response.clone());
                                    database_map
                                        .insert(dns_response.data.query_info.name.clone(), db);
                                }
                            };
                            send_answer(dns_response, src_addr);
                            return;
                            //Return
                        }
                        _ => (),
                    };
                }
            } else {
                debug!(
                    "EV @ lower-authority-doesnt-exist {}",
                    queried_domain.to_string()
                );
                //Here we know that no lower authority exists, and if we don't have the value on
                //our cache, that means that it doesn't exist. We know that because we are the
                //authority.
                if am_parent_authority {
                    dns_message.header.response_code = Some(2);
                } else {
                    let dns_response =
                        match resolver(&mut dns_message, root_servers, supports_recursive) {
                            Ok(message) => message,
                            Err(err) => panic!("{err}"),
                        };
                    match database_map.contains_key(&dns_response.data.query_info.name) {
                        true => database_map
                            .get_mut(&dns_response.data.query_info.name)
                            .unwrap()
                            .add_dns_message(dns_response.clone()),
                        false => {
                            let mut db = DomainDatabase::new();
                            db.add_dns_message(dns_response.clone());
                            database_map.insert(dns_response.data.query_info.name.clone(), db);
                        }
                    };
                    send_answer(dns_response, src_addr);
                    return;
                }
            }
        }

        // If we are the authority, set the authority flag on the response message
        // Otherwise, clear the authority flag
        dns_message.header.flags = if am_parent_authority { 1 } else { 0 };

        //Translate all values to IPs and add it to extra values

        //Get all A records
        let mut extra_values = Vec::new();
        let a_records = match parent_db.get_a_records() {
            Some(records) => records,
            None => panic!("No A records found, cannot get IP of an NS entry"),
        };

        //Get all response values
        let response_vals = match dns_message.data.response_values {
            Some(ref values) => values.to_owned(),
            None => Vec::new(),
        };
        //Get all authorities_values
        let auth_vals = &match dns_message.data.authorities_values {
            Some(ref values) => values.clone(),
            None => Vec::new(),
        };

        //List of all values to translate
        let to_translate = {
            let mut auth_copy = auth_vals.to_owned();
            let mut no_a_records = response_vals
                .into_iter()
                .filter(|entry| entry.type_of_value != "A")
                .to_owned()
                .collect::<Vec<DNSEntry>>();
            no_a_records.append(&mut auth_copy);
            no_a_records
        };

        //Translate all values
        for entry in to_translate {
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
            }
        }
        //Add translated values to extra_values field in response message
        dns_message.data.extra_values = Some(extra_values.to_owned());
        dns_message.header.number_of_extra_values = match extra_values.len().try_into() {
            Ok(num) => Some(num),
            Err(_err) => None,
        };

        if am_parent_authority {
            if let Some(domain_config) = config.get_domain_configs().get(parent_domain_name) {
                let path = &domain_config.get_domain_log().to_string();
                let parent_dir = Path::new(path).parent().unwrap();
                std::fs::create_dir_all(parent_dir).unwrap();
                let mut domain_log_file = File::options()
                    .append(true)
                    .create(true)
                    .open(path)
                    .unwrap();
                let response_timestamp = format!("{}", Utc::now().format("%Y-%m-%d %H:%M:%S %Z"));
                let query_string = format!(
                    "[{}] QR {} {}\n",
                    queried_timestamp, src_addr, queried_message_string
                );
                let response_string = format!(
                    "[{}] RP {} {}\n",
                    response_timestamp, src_addr, queried_message_string
                );
                domain_log_file.write(query_string.as_bytes()).unwrap();
                domain_log_file.write(response_string.as_bytes()).unwrap();
            }
        }
    } else {
        //Answer is not cached
        let dns_response = match resolver(&mut dns_message, root_servers, supports_recursive) {
            Ok(message) => message,
            Err(err) => panic!("{err}"),
        };
        match database_map.contains_key(&dns_response.data.query_info.name) {
            true => database_map
                .get_mut(&dns_response.data.query_info.name)
                .unwrap()
                .add_dns_message(dns_response.clone()),
            false => {
                let mut db = DomainDatabase::new();
                db.add_dns_message(dns_response.clone());
                database_map.insert(dns_response.data.query_info.name.clone(), db);
            }
        };
        dns_message = dns_response;
    };

    send_answer(dns_message, src_addr);
    return;
}

fn send_answer(dns_message: DNSMessage, destination: SocketAddr) {
    let addr = destination.ip();
    let port = destination.port();
    let send_socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(_) => {
            debug!("FL @ client-handler-socket-fail");
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
                debug!("EV @ send-message-fail");
                return;
            }
        };
}

fn create_logger_config(log_path: String, level: LevelFilter) -> log4rs::Config {
    let logging_pattern = PatternEncoder::new(LOG_PATTERN);
    // Logging
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

fn create_logger(level: LevelFilter) -> log4rs::Handle {
    let logging_pattern = PatternEncoder::new(LOG_PATTERN);

    // Logging
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
