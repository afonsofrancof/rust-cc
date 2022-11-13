use std::{
    collections::HashMap,
    hash::Hash,
    ops::Add,
    path::{self, Path},
    sync::mpsc::Receiver,
};

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

        match config.server_dd.get(dns_message.data.query_info.name){
            Some(ip) => match ip.as_str(){
                "127.0.0.1" =>  
            }
            None =>  
        }

    }
}
