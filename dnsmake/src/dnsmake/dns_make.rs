use crate::dns_message_struct::DNSMessage;
use std::net::{UdpSocket, SocketAddr};
use bincode;
pub fn dnssend(dns_message:DNSMessage,server_ip:String,server_port:i16) -> Result<(),&'static str>{ 
    //Build DNSMessage structure
    //Socket
    //Create an outgoing socket
    let outgoing_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    //Serialize the DNSMessage to JSON (Temporarily)
    let dns_message_serialized = bincode::serialize(&dns_message).unwrap();
    let dns_message_bytes: &[u8] = &dns_message_serialized;
    //Send DNSMessage to the Dns Server
    match outgoing_socket.send_to(dns_message_bytes, format!("{server_ip}:{server_port}")){
        Ok(..) => return Ok(()),
        Err(..) => return Err("Could not send DNS request to the server.")
    };
    
}

pub fn dns_recv(port:i16) -> Result<(DNSMessage,SocketAddr),&'static str>{
    //Create UDP socket
    let incoming_socket = match UdpSocket::bind(format!("127.0.0.1:{port}")){
        Err(..) => return Err("Could not bind incoming socket to specified port"),
        Ok(socket) => socket
    };
    //Recieve data on UDP Socket
    let mut recv_buf = [0;1000];
    let (_,src_addr) = match incoming_socket.recv_from(&mut recv_buf){
        Err(..) => return Err("Could not recieve data on incoming socket."),
        Ok(bytes_and_addr) => bytes_and_addr
    };
    //Falta criar THREAD AQUI!
    //Deserialize data
    let dns_message: DNSMessage = match bincode::deserialize(&mut recv_buf){
        Err(err) => {println!("{}",err.to_string()); return Err("Could not deserialize the recieved DNSMessage.{err}")},
        Ok(dns_message) => dns_message
    };
    Ok((dns_message,src_addr))
}
