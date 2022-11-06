use dnsmake::dns_message_struct::{DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType};
use dnsmake::dns_make::dns_recv;
use clap::*;
fn main(){
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
                                Arg::new("no-domain")
                                    .help("Starts the server without any owned domain")
                            ])
                    )
                    .get_matches();


    match dns_recv(5454){
        Ok(value) => (),
        Err(err) => println!("{}",err.to_string())
    };
}
