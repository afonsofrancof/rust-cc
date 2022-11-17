use clap::*;
use core::panic;
use my_dns::dns_make::dns_send;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
use std::net::UdpSocket;

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

    let input_types = match arguments.get_one::<String>("query_types") {
        Some(name) => name,
        None => panic!("No query types provided."),
    }
    .split(',');

    // Passar de string para a Enum QueryType
    // resultando em erro, e cancelada a execucao
    let mut query_types: Vec<QueryType> = Vec::new();
    for qtype in input_types {
        println!("{}", qtype);
        match QueryType::from_string(qtype.to_string()) {
            Ok(q) => query_types.push(q),
            Err(e) => {
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

    // Construir a mensagem de DNS a ser enviada e serialize
    let dns_message = query_builder(domain_name.to_string(), vec![QueryType::A], flag);

    let recv_socket = UdpSocket::bind(server_ip.to_string()).unwrap();

    let size_sent = match dns_send::send(dns_message, &recv_socket, "10.0.0.14".to_string(), 5353) {
        Err(err) => panic!("{err}"),
        Ok(size_sent) => size_sent,
    };

    let mut buf = [0; 1000];
    let _ = recv_socket.recv_from(&mut buf).unwrap();
    let dns_recv_message: DNSMessage = bincode::deserialize(&mut buf).unwrap();

    for (entry_type, entry_vector) in dns_recv_message.data.response_values.unwrap().iter() {
        println!("{} Responses:", entry_type.to_string());
        for entry in entry_vector {
            println!(
                "{} {} {} {}",
                entry.name, entry.type_of_value, entry.value, entry.ttl
            );
        }
        println!("\n---------");
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
