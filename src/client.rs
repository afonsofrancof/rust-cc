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
use my_dns::{dns_make::{dns_recv, dns_send}, dns_structs::dns_domain_name::Domain};
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
use std::fs;
use std::{io, net::UdpSocket, time::Duration};
use std::{ops::Add, thread};

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



    // Remover o ficheiro de log anterior, caso exista
    let _rm = fs::remove_file("logs/client.log");

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
            level = log::LevelFilter::Error;
            level_filter = "shy";
        }
    };

    let logging_pattern = PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S %Z)(utc)}] {m}{n}");

    let file_path = "logs/client.log";

    // Construir o logger para o stdout.
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(logging_pattern.to_owned()))
        .target(Target::Stdout)
        .build();

    // Construir o logger para o ficheiro.
    let logfile = FileAppender::builder()
        .encoder(Box::new(logging_pattern))
        .build(file_path)
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
    info!("EV @ log-file-create /logs/client.log");

    let domain_name = arguments.get_one::<String>("domain").unwrap();

    let input_types = arguments.get_many::<String>("query_types").unwrap();

    // Passar de string para a Enum QueryType
    // resultando em erro, e cancelada a execucao
    let mut query_types: Vec<QueryType> = Vec::new();
    for qtype in input_types {
        match QueryType::from_string(qtype.to_string()) {
            Ok(q) => query_types.push(q),
            Err(_e) => {
                error!("SP 127.0.0.1 invalid-user-input {qtype}");
                return;
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

    start_client(Domain::new(domain_name.to_string()), query_types, flag, server_ip);


}

pub fn start_client(
    domain_name: Domain,
    query_types: Vec<QueryType>,
    flag: u8,
    server_ip: String,
) -> DNSMessage {

    // Construir a mensagem de DNS a ser enviada e dar serialize
    let mut dns_message = query_builder(domain_name, query_types, flag);
    info!("EV @ dns-msg-created");

    // Inicializar a socket UDP
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
    socket.set_write_timeout(Some(Duration::new(1, 0))).unwrap();

    let _size_sent = match dns_send::send(dns_message.to_owned(), &socket, server_ip.to_owned()) {
        Ok(size_sent) => {
            info!(
                "QE {} dns-msg-sent: {}",
                server_ip.to_owned(),
                dns_message.to_owned().get_string()
            );
            size_sent
        }
        Err(err) => {
            error!("TO {} invalid-socket-address", server_ip.to_owned());
            panic!("{err}");
        }
    };

    let (dns_recv_message, _src_addr) = match dns_recv::recv(&socket) {
        Ok(response) => response,
        Err(err) => match err {
            IOError => {
                error!("TO {} invalid-socket-address", server_ip.to_owned());
                panic!("Receiving Query Answer");
            }
            DeserializeError => {
                error!("ER {} could-not-decode", server_ip.to_owned());
                panic!("Receiving Query Answer");
            }
        },
    };
    info!(
        "RR {} dns-msg-received: {}",
        server_ip.to_owned(),
        dns_recv_message.get_string()
    );

    match receive_client(&mut dns_message, dns_recv_message, &socket) {
        Ok(msg) => {
            info!(
                "RR {} dns-msg-received: {}",
                server_ip.to_owned(),
                msg.get_string()
            );
            info!("SP 127.0.0.1 received-final-answer");
            msg
        }
        Err(err) => {
            error!("SP 127.0.0.1 {}", err);
            panic!("Received Invalid Answer")
        }
    }
}

fn receive_client(
    dns_message: &mut DNSMessage,
    dns_recv_message: DNSMessage,
    socket: &UdpSocket,
) -> Result<DNSMessage, &'static str> {
    let mut return_message = Ok(DNSMessage::new());
    if let Some(response_code) = dns_recv_message.header.response_code {
        match response_code {
            // Codigo 0 => Mensagem de resposta valida
            0 => {
                info!("EV @ valid-dns-msg-received");
                return_message = Ok(dns_recv_message.clone());
            }
            // Codigo 1 =>  domínio existe mas não foi obtida a resposta de um servidor de autoridade
            1 => match dns_recv_message.data.authorities_values {
                // Existe pelo menos um servidor de autoridade para o dominio na resposta recebida
                Some(ref auth_values) => {
                    info!("EV @ non-authoritative-msg-received");
                    let mut new_ip;
                    for val in auth_values {
                        // Verificar se o valor do servidor de autoridade é um IP ou um nome
                        if !val.value.chars().all(|c| {
                            vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.'].contains(&c)
                        }) {
                            // Procurar o IP do servidor de autoridade na lista de valores extra
                            new_ip =
                                match dns_recv_message.data.extra_values {
                                    // Procurar na lista de valores extra o IP do servidor de autoridade
                                    Some(ref extra_values) => {
                                        match extra_values.iter().clone().find(|extra| {
                                            extra.domain_name == Domain::new(val.value.to_string())
                                        }) {
                                            Some(ns) => ns.value.to_owned(),
                                            None => continue,
                                        }
                                    }
                                    // Nao foi encontrado nenhum valor extra
                                    None => "No extra values found".to_string(),
                                };
                        } else {
                            new_ip = val.value.to_owned();
                        }
                        let addr_vec = new_ip.split(':').collect::<Vec<_>>();
                        let new_ip_address = match addr_vec.len() {
                            // Formar novo IP com o IP do servidor de autoridade e a porta 5353
                            1 => addr_vec[0].to_string().add(":").add("5353"),
                            // Formar novo IP com o IP obtido dos extra values e a porta recebida
                            2 => new_ip,
                            // Nao foi encontrado um IP valido
                            _ => {
                                error!("SP 127.0.0.1 received-malformed-ip: {}", val.domain_name.to_string());
                                panic!("Malformed IP on {}", val.domain_name.to_string());
                            }
                        };
                        
                        // Enviar a query para o novo IP 
                        let _size_sent = match dns_send::send(dns_message.to_owned(), &socket, new_ip_address.to_owned(),) {
                            Ok(size_sent) => {
                                info!("QE {} sent-new-query", new_ip_address.to_owned());
                                size_sent
                            }
                            Err(err) => {
                                error!("TO {} invalid-socket-address", new_ip_address.to_owned());
                                panic!("{err}");
                            }
                        };
                        
                        // Receber a resposta
                        let (dns_recv_message_new, _src_addr) = match dns_recv::recv(&socket) {
                            Ok(response) => {
                                info!("RR {} dns-msg-received: {}",new_ip_address.to_owned(),response.0.get_string());
                                response
                            }
                            Err(err) => match err {
                                IOError => continue,
                                DeserializeError => {
                                    error!("ER {} could-not-decode", new_ip_address.to_owned());
                                    panic!("Could not deserialize received message")
                                }
                            },
                        };
                        // Recomecar o processo de verificar a resposta
                        return_message = receive_client(dns_message, dns_recv_message_new, &socket);
                        break;
                    }
                }
                None => {}
            },
            2 => return Err("domain-not-found"),
            3 => return Err("malformed-query"),
            _ => return Err("response-code-invalid"),
        }
    }
    return_message
}

fn query_builder(domain_name: Domain, query_types: Vec<QueryType>, flag: u8) -> DNSMessage {
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
