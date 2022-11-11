use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Add;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle,Builder, self};
use my_dns::dns_components::sp::start_sp;
use queues::*;
use clap::*;
use my_dns::dns_make::dns_recv;
use my_dns::dns_parse::{domain_database_parse,server_config_parse};
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use my_dns::dns_structs::domain_database_struct::DomainDatabase;
use my_dns::dns_structs::server_config::ServerConfig;

fn main() {
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("primary")
                .short('p')
                .long("primary")
                .help("Creates a primary DNS server to a domain"),
            Arg::new("secondary")
                .short('s')
                .long("secondary")
                .help("Creates a secondary DNS server to a domain"),
            Arg::new("resolver")
                .short('r')
                .long("resolver")
                .help("Creates a DNS resolver"),
            Arg::new("config_dir")
                .long("config_dir")
                .help("Directory where the config files are stored"),
        ])
        .get_matches();
    


    let mut main_queue: HashMap<String,Queue<DNSMessage>> = HashMap::new();
    let main_queue_shared = Arc::new(main_queue);

    let mut thread_list: Vec<_>= Vec::new();


    let mut primary_databases: HashMap<String, DomainDatabase> = HashMap::new();

    let mut configs: HashMap<String,ServerConfig> = HashMap::new();

    let mut config_dir = match arguments.get_one::<String>("config_dir"){
        Some(config_dir_arg) => config_dir_arg,
        None => {panic!("No config directory specified")}
    };

    match arguments.get_many::<String>("primary"){
        Some(domains) => {
            for domain in domains{
                match Path::new(&config_dir).join(domain.clone().replace(".", "-").add(".conf")).to_str(){
                    Some(path) => match server_config_parse::get(path.to_string()){
                                    Ok(config) => configs.insert(domain.to_owned(), config),
                                    Err(err) => panic!("{err}")
                                },
                    None => {panic!("No config file found for the domain {}",domain)}
                };    
                let database_path = &configs.get(&domain.to_owned()).unwrap().domain_db;
                match domain_database_parse::get(database_path.to_owned()){
                    Ok(database) => {primary_databases.insert(domain.to_owned(), database);},
                    Err(err) => panic!("{err}")
                }
               let thread_handle = thread::Builder::new()
                    .name(format!("sp_{}",domain.clone()))
                    .spawn(move || start_sp());
               thread_list.push(thread_handle);
            }
        },
        _ => ()
    }
   
    match dns_recv::recv(5454, &primary_databases) {
        Ok(value) => (),
        Err(err) => println!("{}", err.to_string()),
    };

    for thrd in thread_list{
        thrd.expect("THREAD error? I guess");
    }
}
