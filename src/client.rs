use clap::{parser::ValuesRef, *};
use core::panic;
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
use my_dns::dns_structs::dns_domain_name::Domain;
use my_dns::{
    dns_components::sr::start_sr,
    dns_structs::dns_message::{
        DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
    },
};
use rand::random;
use std::net::SocketAddr;
use std::ops::Add;

pub fn main() {
    // Argumentos do CLI
    let arguments = Command::new("client")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("domain")
                .short('d')
                .long("domain")
                .required(true)
                .help("Name of the domain to query"),
            Arg::new("query_types")
                .short('t')
                .long("types")
                .required(true)
                .help("Types of Entries needed"),
            Arg::new("recursive")
                .action(ArgAction::SetTrue)
                .short('r')
                .long("recursive")
                .help("Creates a DNS resolver"),
            Arg::new("server_ip")
                .short('s')
                .long("server")
                .num_args(1..)
                .help("Server IP Address"),
            Arg::new("debug")
                .action(ArgAction::SetTrue)
                .short('b')
                .long("debug")
                .help("Debug Mode"),
        ])
        .get_matches();

    // Logging
    // Caso o modo debug esteja ativo, o log é escrito para o terminal
    let debug_mode: bool = arguments.get_flag("debug");
    let level_filter;
    let level;
    match debug_mode {
        true => {
            level = log::LevelFilter::Debug;
            level_filter = "debug";
        }
        false => {
            level = log::LevelFilter::Info;
            level_filter = "shy";
        }
    };

    let logging_pattern = PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {m}{n}");

    let log_file_path = "logs/client.log";

    // Construir o logger para o stdout.
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(logging_pattern.to_owned()))
        .target(Target::Stdout)
        .build();

    // Construir o logger para o ficheiro.
    let logfile = FileAppender::builder()
        .encoder(Box::new(logging_pattern))
        .build(log_file_path)
        .unwrap();

    // Construir o config para o log4rs.
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

    // Inicializar o logger.
    let _handle = log4rs::init_config(config).unwrap();
    info!("ST 127.0.0.1 53 TTL {level_filter}");
    debug!("EV @ log-file-create {}", log_file_path);

    let domain_name = arguments.get_one::<String>("domain").unwrap();

    let query_type_string = arguments.get_one::<String>("query_types").unwrap();

    // Passar de string para a Enum QueryType
    // resultando em erro, e cancelada a execucao
    let query_type: QueryType = match QueryType::from_string(query_type_string.to_string()) {
        Ok(q) => q,
        Err(_e) => {
            error!("SP 127.0.0.1 invalid-user-input {query_type_string}");
            return;
        }
    };

    // O sistema de flags funciona em binario em que se soma o valor de todas as flags
    // Q  => 1 0 0 = 4
    // R  => 0 1 0 = 2
    // A  => 0 0 1 = 1
    // QR => 1 1 0 = 6
    let flag: u8 = match arguments.get_flag("recursive") {
        true => 6,
        false => 4,
    };

    let server_ips_input: ValuesRef<String> = match arguments.get_many::<String>("server_ip") {
        Some(ips) => ips.to_owned(),
        None => panic!("No IP provided"),
    };

    let mut server_ips_vec: Vec<SocketAddr> = Vec::new();

    for server_ip in server_ips_input.into_iter() {
        let addr_vec = server_ip.split(':').collect::<Vec<_>>();
        let new_ip_address = if addr_vec.len() == 1 {
            addr_vec[0].to_string().add(":").add("5353")
        } else {
            server_ip.to_string()
        };
        let server_ip_socket_addr = match new_ip_address.parse() {
            Ok(ip) => ip,
            Err(_err) => panic!("Malformed server ip {}", server_ip.to_string()),
        };
        server_ips_vec.push(server_ip_socket_addr)
    }
    // Construir a mensagem de DNS a ser enviada e dar serialize
    let mut dns_message = query_builder(Domain::new(domain_name.to_string()), query_type, flag);
    info!("EV @ dns-msg-created");

    let answer = start_sr(&mut dns_message, server_ips_vec, true);

    let check_answer = match answer {
        Ok(acceptable) => {
            info!("EV @ dns-msg-received");
            acceptable
        }
        Err(err) => {
            error!("EV @ {}", err);
            error!("SP 127.0.0.1 dns-msg-receive-error");
            panic!("Error receiving answer");
        }
    };

    print_dns_message(check_answer);
}

pub fn query_builder(domain_name: Domain, query_type: QueryType, flag: u8) -> DNSMessage {
    let dns_query_info = DNSQueryInfo {
        name: domain_name,
        type_of_value: query_type,
    };
    let dns_message_data = DNSMessageData {
        query_info: dns_query_info,
        response_values: None,
        authorities_values: None,
        extra_values: None,
    };
    let dns_message_header = DNSMessageHeaders {
        message_id: random(),
        flags: flag,
        response_code: None,
        number_of_values: None,
        number_of_authorities: None,
        number_of_extra_values: None,
    };
    let dns_message = DNSMessage {
        header: dns_message_header,
        data: dns_message_data,
    };

    return dns_message;
}

fn print_dns_message(message: DNSMessage) {
    println!("DNS Message:");
    println!(
        "{}, {}, {}, {}, {}, {}",
        message.header.message_id,
        message.header.flags.decode_flags().unwrap(),
        message.header.response_code.unwrap(),
        message.header.number_of_values.unwrap(),
        message.header.number_of_authorities.unwrap(),
        message.header.number_of_extra_values.unwrap()
    );

    println!("  Response Values:");
    if let Some(response_values) = message.data.response_values {
        for value in response_values {
            println!(
                "{}, {}, {}, {}, {}",
                value.domain_name.to_string(),
                value.type_of_value,
                value.value,
                value.ttl,
                value.priority.unwrap()
            );
        }
    }
    println!("  Authorities Values:");
    if let Some(authorities_values) = message.data.authorities_values {
        for value in authorities_values {
            println!(
                "{}, {}, {}, {}, {}",
                value.domain_name.to_string(),
                value.type_of_value,
                value.value,
                value.ttl,
                value.priority.unwrap()
            );
        }
    }
    println!("  Extra Values:");
    if let Some(extra_values) = message.data.extra_values {
        for value in extra_values {
            println!(
                "{}, {}, {}, {}, {}",
                value.domain_name.to_string(),
                value.type_of_value,
                value.value,
                value.ttl,
                value.priority.unwrap()
            );
        }
    }
}
