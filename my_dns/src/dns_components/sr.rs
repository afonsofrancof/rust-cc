use std::{path::{Path, self}, sync::mpsc::Receiver, hash::Hash, collections::HashMap, ops::Add};

use crate::{dns_structs::{domain_database_struct::DomainDatabase, server_config::ServerConfig, dns_message::{DNSMessage}}, dns_parse::server_config_parse};
use queues::*;

pub fn start_sr(config_dir:String,receiver: Receiver<DNSMessage>){
}

