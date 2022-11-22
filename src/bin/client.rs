use clap::*;
use core::panic;
use my_dns::dns_make::{dns_recv, dns_send};
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
use std::net::UdpSocket;
use std::ops::Add;

fn main() {
    // Argumentos de Input para fazer queries
    // Ordem de input de argumentos:
    // 1. Domain_name                               Obrigatorio
    // 2. Types of values (separados por virgula)   Obrigatorio
    // 3. Flag Recursiva                            Opcional
    // 4. Ip do servidor a quem enviar pedido       Opcional
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
        ])
        .get_matches();

    let domain_name = match arguments.get_one::<String>("domain") {
        Some(name) => name,
        None => panic!("No domain provided."),
    };

    let input_types = match arguments.get_many::<String>("query_types") {
        Some(name) => name,
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

    println!("DNS Server IP: {}", server_ip);

    // Construir a mensagem de DNS a ser enviada e serialize
    let mut dns_message = query_builder(domain_name.to_string(), query_types, flag);

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let _size_sent = match dns_send::send(dns_message.to_owned(), &socket, server_ip.to_owned()) {
        Err(err) => panic!("{err}"),
        Ok(size_sent) => size_sent,
    };

    let (dns_recv_message, _src_addr) = match dns_recv::recv(&socket) {
        Ok(response) => response,
        Err(err) => panic!("{err}"),
    };

    receive_client(&mut dns_message, dns_recv_message, &socket);
    // match dns_recv_message.data.response_values {
    //     Some(response_vec) => {
    //         for (entry_type, entry_vector) in response_vec.iter() {
    //             println!("{} Responses:", entry_type.to_string());
    //             for entry in entry_vector {
    //                 println!(
    //                     "{} {} {} {}",
    //                     entry.name, entry.type_of_value, entry.value, entry.ttl
    //                 );
    //             }
    //             println!("\n---------");
    //         }
    //     }
    //     None => { println!("No response values received");//NEED TO CHECK NS
    //     }
    // }
}

fn receive_client(dns_message: &mut DNSMessage, dns_recv_message: DNSMessage, socket: &UdpSocket) {
    if let Some(response_code) = dns_recv_message.header.response_code {
        match response_code {
            0 => println!("{}", dns_recv_message.get_string()),
            1 => match dns_recv_message.data.authorities_values {
                Some(ref auth_values) => {
                    let new_ip;
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
                            Err(_err) => panic!("No response received"),
                        };
                        receive_client(dns_message, dns_recv_message_new, &socket);
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
