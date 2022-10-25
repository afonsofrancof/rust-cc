use std::net::UdpSocket;

#[allow(unused)]
fn main() {
    //send udp data to a socket
    let socket = UdpSocket::bind("10.0.0.13:0").unwrap();
    let data = "hello world";
    socket.send_to(data.as_bytes(), "10.0.0.13:8001").unwrap();
}
