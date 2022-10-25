use std::{
    net::{SocketAddr, UdpSocket},
    thread,
};

fn main() {
    //create a upsocket and bind it to port 8000 and local address
    let socket = UdpSocket::bind("0.0.0.0:8001").unwrap();
    //create a buffer to store the data
    let mut buf = [0; 11];
    //loop forever
    loop {
        //wait for data to be received
        let (num_of_bytes, src_addr) = socket.recv_from(&mut buf).unwrap();
        let recv_struct = (num_of_bytes, buf, src_addr);
        thread::spawn(move || client_handler(recv_struct));
    }
}

fn client_handler((num_of_bytes, data, src_addr): (usize, [u8; 11], SocketAddr)) {
    match data[0] {
        0 => {}
        1 => {}
        _ => {}
    }
}
