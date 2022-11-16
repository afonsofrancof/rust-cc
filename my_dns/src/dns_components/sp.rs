use std::{collections::HashMap,hash::Hash,ops::Add,path::{self, Path},str::pattern::Pattern,sync::mpsc::Receiver, net::{UdpSocket, SocketAddr}, thread,};

use crate::{
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::DNSMessage, domain_database_struct::DomainDatabase,
        server_config::ServerConfig,
    },
};
use queues::*;

pub fn start_sp(config_path: String,port: u16) {
    let database: HashMap<String,DomainDatabase>;
    
   // parsing da config 
    let config: ServerConfig = match server_config_parse::get(config_path) {
        Ok(config) => config,
        Err(err) => panic!("Server config path not found!")
    };

    let socket = match UdpSocket::bind(format!("127.0.0.1:{port}",)){
        Ok(socket) => socket,
        Err(_) => panic!("Could not bind socket")
    };

        let buf = [0;1000];

    loop {
        
        let (num_of_bytes,src_addr) = match socket.recv_from(&mut buf){
            Ok(size_and_addr) => size_and_addr,
            Err(_) => panic!("Could not receive on socket")
        };
       
        let builder = thread::spawn(move || client_handler(buf.to_vec(),num_of_bytes,src_addr,database));
        
    }
}

fn client_handler(buf: Vec<u8>,num_of_bytes:usize, src_addr:SocketAddr,database: HashMap<String,DomainDatabase>){

    let dns_message: DNSMessage = match bincode::deserialize(&buf){
        Ok(message) => message,
        Err(_) => panic!("Could not deserialize message")
    };
        let queried_domain = dns_message.data.query_info.name;
        let queried_domain_db = match database.get("queried_domain"){
           Some(domain_database) => domain_database,
            None => panic!("Domain database not found")
        };
        match queried_domain_db.subdomain_name_servers {
            Some(ns_list) => {
                match ns_list
                    .iter()
                    .filter(|ns| {".".to_string().add(&ns.name).is_suffix_of(&".".to_string().add(&queried_domain))})
                    .max_by(|ns1,ns2| ns1.name.len().cmp(&ns2.name.len())){
                        Some(sp) => Some(sp),
                        None => None
                    }
            },
            None => {println!("No NS found for {}",queried_domain);None}
        };

}
