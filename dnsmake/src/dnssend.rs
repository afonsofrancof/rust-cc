mod dns_message_struct;
use dns_message_struct::{DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType};
use rand::random;
use std::net::UdpSocket;
use serde_json;

pub fn dnssend(name:String,query_type:String,server_ip:String) -> Result<(),&'static str>{ 
    let qtype = match &query_type[..]{
        "NS" => QueryType::NS,
        "A"  => QueryType::A,
        "CNAME"  => QueryType::CNAME,
        "MX"  => QueryType::MX,
        "PTR"  => QueryType::PTR,
        _ => return Err("Query type does not exist") 
    };  

    //Build DNSMessage structure
    let dns_query_info = DNSQueryInfo{name,type_of_value:vec![qtype]};
    let dns_message_data = DNSMessageData {query_info: dns_query_info,response_values:None,authorities_values:None,extra_values:None};
    let dns_message_header = DNSMessageHeaders{message_id:random(),flags: 6,response_code:None,number_of_values:None,number_of_authorities:None,number_of_extra_values:None};
    let dns_message = DNSMessage {header:dns_message_header,data:dns_message_data};

    //Socket
    //Create an outgoing socket
    let outgoing_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    //Serialize the DNSMessage to JSON (Temporarily)
    let dns_message_serialized = serde_json::to_vec(&dns_message).unwrap();
    let dns_message_bytes: &[u8] = &dns_message_serialized;
    //Send DNSMessage to the Dns Server
    match outgoing_socket.send_to(dns_message_bytes, "127.0.0.1:5353"){
        Ok(..) => return Ok(()),
        Err(..) => return Err("Could not send DNS request to the server.")
    };
} 
