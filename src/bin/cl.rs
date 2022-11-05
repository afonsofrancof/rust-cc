use data_structs::dns_message::{DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType, self};
use std::{env, array, io::Read};
use rand::random;
use std::net::UdpSocket;
use serde;
fn main(){
    env::args().next();
    let name = env::args().next().unwrap();
    let query_type = env::args().next().unwrap();
        let query_type_str = query_type.as_str();
    let server_ip = env::args().next().unwrap();
    let qtype = match query_type_str{
        "NS" => QueryType::NS,
        "A"  => QueryType::A,
        "CNAME"  => QueryType::CNAME,
        "MX"  => QueryType::MX,
        "PTR"  =>QueryType::PTR,
        _ => QueryType::A
    };
    let dns_query_info = DNSQueryInfo{name,type_of_value:vec![qtype]};
    let dns_message_data = DNSMessageData {query_info: dns_query_info,response_values:None,authorities_values:None,extra_values:None};
    let dns_message_header = DNSMessageHeaders{message_id:random(),flags: 6,response_code:None,number_of_values:None,number_of_authorities:None,number_of_extra_values:None};
    let dns_message = DNSMessage {header:dns_message_header,data:dns_message_data};
    
    //Socket

    let outgoing_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let dns_message_serialized = serde_json::to_vec(&dns_message).unwrap();
    let dns_message_bytes: &[u8] = &dns_message_serialized;
    println!("{server_ip}:5353");
    outgoing_socket.send_to(dns_message_bytes, "127.0.0.1:5353").expect("Could not send data");
    let mut buf = [0;1000];
    let (number_of_bytes,src_addr) = outgoing_socket.recv_from(&mut buf).unwrap();
    let dns_message_recv : DNSMessage = serde_json::from_slice(&buf).unwrap(); 
    for entry in dns_message_recv.data.response_values.unwrap().into_iter(){
        println!("{} {} {}",entry.name,entry.type_of_value,entry.value);
    }
}



