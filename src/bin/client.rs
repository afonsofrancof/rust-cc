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
    // 1. domain_name
    // 2.type of value (separados virgula)
    // 3. flags (recursivo ou nao)
    // 4. Opcionalmente o ip de um servidor dns
    let args: Vec<String> = env::args().collect();

    let domain_name = args[1].to_owned();
    let input_types = args[2].split(',');

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

    let dns_message = query_builder(domain_name, vec![QueryType::A]);

    let mut recv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    let my_port = match dns_send::send(dns_message, &recv_socket, "127.0.0.1:5454".to_string()) {
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

fn query_builder(domain_name: String, query_types: Vec<QueryType>) -> DNSMessage {
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
        flags: 6,
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
