use core::{num, panic};
use my_dns::dns_make::dns_send;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
use std::env;
use std::net::UdpSocket;
use std::str::FromStr;

fn main() {
    // Ordem de input de argumentos:
    // 1. Domain_name                               Obrigatorio
    // 2. Types of values (separados por virgula)   Obrigatorio
    // 3. Flag Recursiva Ou Nao                     Obrigatorio
    // 4. Ip do servidor a quem enviar pedido      Opcional
    let args: Vec<String> = env::args().collect();
    let n_args = args.len();

    let domain_name = args[1].to_owned();
    let input_types = args[2].split(',');

    // Passar de string para a Enum QueryType
    // resultando em erro, e cancelada a execucao
    let mut query_types: Vec<QueryType> = Vec::new();
    for qtype in input_types {
        println!("{}", qtype);
        match QueryType::from_str(qtype) {
            Ok(q) => query_types.push(q),
            Err(e) => {
                panic!("Input invalido: {}", qtype);
            }
        };
    }

    // O sistema de flags funciona em binario em que se soma o valor de todas as flags
    // Q  => 1 0 0 = 4
    // R  => 0 1 0 = 2
    // A  => 0 0 1 = 1
    // QR => 1 1 0 = 6
    let mut flag: u8 = 4;
    let mut server_ip: String = "void".to_string();
    match args[3] {
        'R' => flag = 6,
        _ => server_ip = args[3],
    }

    match server_ip {
        "void" => server_ip = "127.0.0.1:5454",
    }

    // Construir a mensagem de DNS a ser enviada e serialize
    let dns_message = query_builder(domain_name, vec![QueryType::A], flag);

    let mut recv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    let my_port = match dns_send::send(dns_message, &recv_socket, server_ip.to_string()) {
        Err(err) => panic!("{err}"),
        Ok(port) => port,
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
