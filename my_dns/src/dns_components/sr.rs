use std::{path::{Path, self}, sync::mpsc::Receiver, hash::Hash, collections::HashMap, ops::Add};

use crate::{dns_structs::{domain_database_struct::DomainDatabase, server_config::ServerConfig, dns_message::{DNSMessage}}, dns_parse::server_config_parse};
use queues::*;

pub fn start_sr(config_dir:String,receiver: Receiver<DNSMessage>){

    let config: ServerConfig;

    match Path::new(&config_dir).join("resolver".to_string().add(".conf")).to_str(){
        Some(path) => match server_config_parse::get(path.to_string()){
            Ok(config_parsed) => config = config_parsed,
            Err(err) => panic!("{err}")
        },
        None => {panic!("no config file found for the DNS Resolver")}
    };

    loop{
        let dns_message = match receiver.recv(){
            Err(err) => panic!("{err}"),
            Ok(ok) => ok
        };
        println!("SR received query of {}",dns_message.data.query_info.name);
    }
}

