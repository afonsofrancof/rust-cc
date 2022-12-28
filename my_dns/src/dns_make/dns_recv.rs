use crate::dns_structs::dns_message::DNSMessage;
use bincode;
use std::{net::{SocketAddr, UdpSocket}, io};

pub enum RecvError{
    IOError(io::Error),
    DeserializeError(bincode::Error)
}

pub fn recv(incoming_socket: &UdpSocket) -> Result<(DNSMessage,SocketAddr),RecvError> {
    let mut recv_buf = [0; 1000];
    let (_, src_addr) = match incoming_socket.recv_from(&mut recv_buf) {
        Ok(bytes_and_addr) => bytes_and_addr,
        Err(err) => return Err(RecvError::IOError(err))
    };
    let dns_message: DNSMessage = match bincode::deserialize(&recv_buf){
        Ok(message) => message,
        Err(err) => return Err(RecvError::DeserializeError(err))
    };
    Ok((dns_message,src_addr))
}
