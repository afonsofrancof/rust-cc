use std::{collections::HashMap,hash::Hash,ops::Add,path::{self, Path},str::pattern::Pattern,sync::mpsc::Receiver, net::UdpSocket,};

use crate::{
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::DNSMessage, domain_database_struct::DomainDatabase,
        server_config::ServerConfig,
    },
};
use queues::*;

pub fn start_sp(domain_name: String, config: ServerConfig, receiver: Receiver<DNSMessage>) {
    let config: ServerConfig;
    let database: DomainDatabase;

    database = match domain_database_parse::get(config.domain_db) {
        Ok(db) => db,
        Err(err) => panic!("{err}"),
    };

    loop {
        let dns_message = match receiver.recv() {
            Err(err) => panic!("{err}"),
            Ok(ok) => ok,
        };
        let queried_domain = dns_message.data.query_info.name;
        let queried_domain_sp = match database.entry_list.get("NS") {
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
        }
        
        match queried_domain_sp{
            Some(entry) => {
                let forward_socket = match UdpSocket::bind("127.0.0.1:0"){
                    Ok(socket) => socket,
                    Err(err) => {println!("Could not open socket to contact subdomain's SP");continue;}
                };   
                   
            },
            None => ()
        }
    }
}
