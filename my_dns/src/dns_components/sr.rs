use std::{
    collections::HashMap,
    hash::Hash,
    ops::Add,
    path::{self, Path},
    sync::mpsc::Receiver,
};

use crate::{
    dns_parse::server_config_parse,
    dns_structs::{
        dns_message::DNSMessage, domain_database_struct::DomainDatabase,
        server_config::{ServerConfig, DomainConfig},
    },
};
use queues::*;

pub fn start_sr(server_config: ServerConfig,db: Option<DomainDatabase>) {}
