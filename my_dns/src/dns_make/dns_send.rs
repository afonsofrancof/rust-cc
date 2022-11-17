use crate::dns_structs::dns_message::*;
use bincode;
use std::net::{UdpSocket};

pub fn send(
    dns_message: DNSMessage,
    socket :& UdpSocket,
    remote_addr_and_port : String,
) -> Result<usize,std::io::Error> {
    //Serialize the DNSMessage to JSON (Temporarily)
    let dns_message_serialized = bincode::serialize(&dns_message).unwrap();
    let dns_message_bytes: &[u8] = &dns_message_serialized;
    //Send DNSMessage to the Dns Server
    let bytes_sent = socket.send_to(dns_message_bytes,remote_addr_and_port);
    bytes_sent
}
