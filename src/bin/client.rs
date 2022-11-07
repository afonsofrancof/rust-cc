use core::num;
use my_dns::dns_make::dns_send;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
use std::net::UdpSocket;

fn main() {
    let dns_query_info = DNSQueryInfo {
        name: "example.com".to_string(),
        type_of_value: vec![QueryType::A, QueryType::NS],
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

    let my_port = match dns_send::send(dns_message, "127.0.0.1".to_string(), 5454) {
        Err(err) => panic!("{err}"),
        Ok(port) => port,
    };

    let recv_socket = UdpSocket::bind(format!("127.0.0.1:{}", my_port)).unwrap();
    let mut buf = [0; 1000];
    let (_, _) = recv_socket.recv_from(&mut buf).unwrap();
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
