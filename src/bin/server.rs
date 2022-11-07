use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use clap::*;
use my_dns::dns_make::dns_recv;
use my_dns::dns_parse::domain_database_parse;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use my_dns::dns_structs::domain_database_struct::DomainDatabase;

fn main() {
    let server = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .subcommand(
            Command::new("type")
                .short_flag('t')
                .long_flag("type")
                .about("Define the type of server to start")
                .args([
                    Arg::new("primary")
                        .help("Starts the server as the primary DNS server to a domain"),
                    Arg::new("secondary")
                        .help("Starts the server as a secondary DNS server to a domain"),
                    Arg::new("no-domain").help("Starts the server without any owned domain"),
                ]),
        )
        .get_matches();

    let mut database: HashMap<String, DomainDatabase> = HashMap::new();

    let example_db = domain_database_parse::get("example-com.db".to_string()).unwrap();

    database.insert("example.com".to_string(), example_db);

    match dns_recv::recv(5454) {
        Ok(value) => (),
        Err(err) => println!("{}", err.to_string()),
    };
}
