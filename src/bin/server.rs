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
        .subcommands([
            Command::new("primary")
                .short_flag('p')
                .long_flag("primary")
                .about("Starts the server as the primary DNS server to a domain")
                .args([
                    Arg::new("domain")
                        .help("Domain name to serve"),
                ]),
            Command::new("secondary")
                .short_flag('s')
                .long_flag("secondary")
                .about("Starts the server as the secondary DNS server to a domain")
                .args([
                    Arg::new("domain")
                        .help("Domain name to serve")
                ]),
            Command::new("resolver")
                .short_flag('r')
                .long_flag("resolver")
                .about("Starts the server as a DNS resolver")
        ])
        .get_matches();

    let mut database: HashMap<String, DomainDatabase> = HashMap::new();

    let example_db = domain_database_parse::get("example-com.db".to_string()).unwrap();

    database.insert("example.com".to_string(), example_db);

    match dns_recv::recv(5454, &database) {
        Ok(value) => (),
        Err(err) => println!("{}", err.to_string()),
    };
}
