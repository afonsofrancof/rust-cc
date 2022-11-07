use crate::dns_structs::dns_message::*;
use bincode;
use std::net::{SocketAddr, UdpSocket};

pub fn send(
    dns_message: DNSMessage,
    server_ip: String,
    server_port: u16,
) -> Result<u16, &'static str> {
    //Build DNSMessage structure
    //Socket
    //Create an outgoing socket
    let outgoing_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    //Serialize the DNSMessage to JSON (Temporarily)
    let dns_message_serialized = bincode::serialize(&dns_message).unwrap();
    let dns_message_bytes: &[u8] = &dns_message_serialized;
    //Send DNSMessage to the Dns Server
    match outgoing_socket.send_to(dns_message_bytes, format!("{server_ip}:{server_port}")) {
        Ok(..) => return Ok(outgoing_socket.local_addr().unwrap().port()),
        Err(..) => return Err("Could not send DNS request to the server."),
    };
}
