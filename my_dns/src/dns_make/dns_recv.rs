use crate::dns_structs::dns_message::*;
use bincode;
use std::net::{SocketAddr, UdpSocket};


pub fn recv(port: i16) -> Result<(DNSMessage, SocketAddr), &'static str> {
    //Create UDP socket
    let incoming_socket = match UdpSocket::bind(format!("127.0.0.1:{port}")) {
        Err(..) => return Err("Could not bind incoming socket to specified port"),
        Ok(socket) => socket,
    };
    //Recieve data on UDP Socket
    let mut recv_buf = [0; 1000];
    let (_, src_addr) = match incoming_socket.recv_from(&mut recv_buf) {
        Err(..) => return Err("Could not recieve data on incoming socket."),
        Ok(bytes_and_addr) => bytes_and_addr,
    };
    //Falta criar THREAD AQUI!
    //Deserialize data
    let dns_message: DNSMessage = match bincode::deserialize(&mut recv_buf) {
        Err(err) => return Err("Could not deserialize the recieved DNSMessage."),
        Ok(dns_message) => dns_message,
    };
    Ok((dns_message, src_addr))
}
