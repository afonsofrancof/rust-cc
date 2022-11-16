use std::{collections::HashMap,hash::Hash,ops::Add,path::{self, Path},str::pattern::Pattern,sync::mpsc::Receiver, net::UdpSocket,};

use crate::{
    dns_parse::{domain_database_parse, server_config_parse},
    dns_structs::{
        dns_message::DNSMessage, domain_database_struct::DomainDatabase,
        server_config::ServerConfig,
    },
};
use queues::*;

pub fn start_sp(config_path: String) {
    let database: DomainDatabase;
    
   // parsing da config 
    let config: ServerConfig = match server_config_parse::get(config_path) {
        Ok(config) => config,
        Err(err) => panic!("Server config path not found!")
    };

    // parse dbs e afins 


    loop {
        
        match queried_domain_sp{
            Some(entry) => {
                let forward_socket = match UdpSocket::bind("127.0.0.1".to_string().add(config.)){
                    Ok(socket) => socket,
                    Err(err) => {println!("Could not open socket to contact subdomain's SP");continue;}
                };   
                   
            },
            None => ()
        }



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
        };
        
    }
}
