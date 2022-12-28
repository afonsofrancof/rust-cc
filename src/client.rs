#![feature(io_error_more)]
use clap::*;
use core::panic;
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use my_dns::dns_make::{dns_recv, dns_send};
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
use std::ops::Add;
use std::{io, net::UdpSocket, time::Duration};

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
                .num_args(1..=5)
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
                .help("Server IP Address"),
            Arg::new("debug")
                .action(ArgAction::SetTrue)
                .short('b')
                .long("debug")
                .help("Debug Mode"),
        ])
        .get_matches();

    let logging_pattern = PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {h({l})} - {m}{n}");
    // Logging
    let level = log::LevelFilter::Info;
    let file_path = "log/beans.log";

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

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config).unwrap();

    error!("error to stdout and file");
    warn!("to stderr stdout and file");
    info!("to stderr stdout and file");
    debug!("debug! to file");
    trace!("debug! to file");

    let debug_mode: bool = arguments.get_flag("debug");

    let domain_name = match arguments.get_one::<String>("domain") {
        Some(name) => name,
        None => panic!("No domain provided."),
    };

    let input_types = match arguments.get_many::<String>("query_types") {
        Some(in_types) => in_types,
        None => panic!("No query types provided."),
    };

    // Passar de string para a Enum QueryType
    // resultando em erro, e cancelada a execucao
    let mut query_types: Vec<QueryType> = Vec::new();
    for qtype in input_types {
        match QueryType::from_string(qtype.to_string()) {
            Ok(q) => query_types.push(q),
            Err(_e) => {
                panic!("Invalid Query Type Input: {}", qtype);
            }
        };
    }

    // O sistema de flags funciona em binario em que se soma o valor de todas as flags
    // Q  => 1 0 0 = 4
    // R  => 0 1 0 = 2
    // A  => 0 0 1 = 1
    // QR => 1 1 0 = 6
    let flag: u8 = match arguments.get_flag("recursive") {
        true => 6,
        false => 4,
    };

    let server_ip: String = match arguments.get_one::<String>("server_ip") {
        Some(ip) => ip.to_string(),
        None => "127.0.0.1:0".to_string(),
    };
    start_client(domain_name.to_string(), query_types, flag, server_ip);
}

pub fn start_client(
    domain_name: String,
    query_types: Vec<QueryType>,
    flag: u8,
    server_ip: String,
) -> DNSMessage {
    info!("DNS Server IP: {}", server_ip);

    // Construir a mensagem de DNS a ser enviada e serialize
    let mut dns_message = query_builder(domain_name.to_string(), query_types, flag);

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
    socket.set_write_timeout(Some(Duration::new(1, 0))).unwrap();

    let _size_sent = match dns_send::send(dns_message.to_owned(), &socket, server_ip.to_owned()) {
        Err(err) => panic!("{err}"),
        Ok(size_sent) => size_sent,
    };

    let (dns_recv_message, _src_addr) = match dns_recv::recv(&socket) {
        Ok(response) => response,
        Err(_err) => panic!("Receive Error"),
    };
    println!("{}", dns_recv_message.get_string());

    match receive_client(&mut dns_message, dns_recv_message, &socket) {
        Ok(msg) => {
            println!("{}", msg.get_string());
            msg
        }
        Err(err) => panic!("{err}"),
    }
}

fn receive_client(
    dns_message: &mut DNSMessage,
    dns_recv_message: DNSMessage,
    socket: &UdpSocket,
) -> Result<DNSMessage, &'static str> {
    let mut return_message = Ok(DNSMessage::new());
    println!("DNS MESSAGE:\n{}", dns_recv_message.get_string());
    if let Some(response_code) = dns_recv_message.header.response_code {
        match response_code {
            0 => {
                return_message = Ok(dns_recv_message.clone());
            }
            1 => match dns_recv_message.data.authorities_values {
                Some(ref auth_values) => {
                    let mut new_ip;
                    for val in auth_values {
                        if !val.value.chars().all(|c| {
                            vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.'].contains(&c)
                        }) {
                            new_ip =
                                match dns_recv_message.data.extra_values {
                                    Some(ref extra_values) => {
                                        match extra_values.iter().clone().find(|extra| {
                                            extra.name.to_owned() == val.value.to_owned()
                                        }) {
                                            Some(ns) => ns.value.to_owned(),
                                            None => continue,
                                        }
                                    }
                                    None => "No extra values found".to_string(),
                                };
                        } else {
                            new_ip = val.value.to_owned();
                        }
                        let addr_vec = new_ip.split(':').collect::<Vec<_>>();
                        let addr_string_parsed = match addr_vec.len() {
                            1 => addr_vec[0].to_string().add(":").add("5353"),
                            2 => new_ip,
                            _ => panic!("Malformed IP on {}", val.name),
                        };
                        println!(
                            "Received non final query, sending to {}",
                            addr_string_parsed
                        );
                        let _size_sent = match dns_send::send(
                            dns_message.to_owned(),
                            &socket,
                            addr_string_parsed,
                        ) {
                            Err(err) => panic!("{err}"),
                            Ok(size_sent) => size_sent,
                        };
                        let (dns_recv_message_new, _src_addr) = match dns_recv::recv(&socket) {
                            Ok(response) => response,
                            Err(err) => match err {
                                IOError => continue,
                                DeserializeError => panic!("Could not deserialize received message")
                            }
                        };
                        return_message = receive_client(dns_message, dns_recv_message_new, &socket);
                        break;
                    }
                }
                None => {}
            },
            2 => println!("Domain not found"),
            3 => println!("Malformed query"),
            _ => println!("Response code invalid"),
        }
    }
    return_message
}

fn query_builder(domain_name: String, query_types: Vec<QueryType>, flag: u8) -> DNSMessage {
    let dns_query_info = DNSQueryInfo {
        name: domain_name,
        type_of_value: query_types,
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
