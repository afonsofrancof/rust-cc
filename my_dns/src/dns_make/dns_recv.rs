use crate::dns_structs::dns_message::*;
use crate::dns_structs::domain_database_struct::DomainDatabase;
use bincode;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::thread;

use super::dns_send;

pub fn recv(port: i16, database: &HashMap<String, DomainDatabase>) -> Result<bool, &'static str> {
    //Create UDP socket
    let incoming_socket = match UdpSocket::bind(format!("127.0.0.1:{port}")) {
        Err(..) => return Err("Could not bind incoming socket to specified port"),
        Ok(socket) => socket,
    };

    let mut client_threads: Vec<Result<_, _>> = Vec::new();

    loop {
        //Recieve data on UDP Socket
        let mut recv_buf = [0; 1000];
        let (_, src_addr) = match incoming_socket.recv_from(&mut recv_buf) {
            Err(..) => return Err("Could not recieve data on incoming socket."),
            Ok(bytes_and_addr) => bytes_and_addr,
        };

        let temp_database = database.to_owned();

        client_threads.push(
            thread::Builder::new()
                .name("thread1".to_string())
                .spawn(move || client_handler(recv_buf.to_vec(), src_addr, temp_database)),
        );
    }
    //Deserialize data
    return Ok(true);
}

fn client_handler(
    mut recv_buf: Vec<u8>,
    src_addr: SocketAddr,
    database: HashMap<String, DomainDatabase>,
) -> Result<bool, &'static str> {
    let mut dns_message: DNSMessage = match bincode::deserialize(&mut recv_buf) {
        Err(err) => return Err("Could not deserialize the recieved DNSMessage."),
        Ok(dns_message) => dns_message,
    };

    let domain = dns_message.data.query_info.name.clone();
    let query_types = dns_message.data.query_info.type_of_value.clone();

    let domain_database = match database.get(&domain){
       Some(domain_db) => domain_db, 
        None => panic!("{}'s database not found in memory",domain)
    };

    let mut response_map: HashMap<QueryType, Vec<DNSSingleResponse>> = HashMap::new();

    for query_type in query_types.into_iter() {
        let response = domain_database
            .entry_list
            .get(query_type.to_string())
            .unwrap();

        let mut response_vec = Vec::new();

        for entry in response {
            response_vec.push(DNSSingleResponse {
                name: entry.name.to_owned(),
                type_of_value: entry.entry_type.clone(),
                value: entry.value.to_owned(),
                ttl: entry.ttl,
            });
        }
        response_map.insert(query_type, response_vec);
    }

    dns_message.data.response_values = Some(response_map);
    let addr = src_addr.ip();
    let port = src_addr.port();
    let send_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let destination = format!("{}:{}",addr,port);
    let _port = dns_send::send(dns_message, &send_socket,destination);

    Ok(true)
}
