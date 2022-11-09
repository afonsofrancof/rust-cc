use crate::dns_structs::dns_message::*;
use bincode;
use std::net::{SocketAddr, UdpSocket};

pub fn send(
    dns_message: DNSMessage,
    socket : UdpSocket
) -> Result<u16, &'static str> {
    //Serialize the DNSMessage to JSON (Temporarily)
    let dns_message_serialized = bincode::serialize(&dns_message).unwrap();
    let dns_message_bytes: &[u8] = &dns_message_serialized;
    //Send DNSMessage to the Dns Server
    match socket.send_to(dns_message_bytes, format!("{server_ip}:{server_port}")) {
        Ok(..) => return Ok(outgoing_socket.local_addr().unwrap().port()),
        Err(..) => return Err("Could not send DNS request to the server."),
    };
}
