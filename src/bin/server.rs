use clap::*;
use my_dns::dns_components::{sp, sr, ss};

fn main() {
    // Argumentos de input da CLI para definir quais e quantos servidores inicializar
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("primary")
                .action(ArgAction::SetTrue)
                .long("primary")
                .group("server_type")
                .help("Creates a primary DNS server to a domain"),
            Arg::new("secondary")
                .action(ArgAction::SetTrue)
                .long("secondary")
                .group("server_type")
                .help("Creates a secondary DNS server to a domain"),
            Arg::new("resolver")
                .action(ArgAction::SetTrue)
                .long("resolver")
                .group("server_type")
                .help("Creates a DNS resolver"),
            Arg::new("config_path")
                .short('c')
                .long("config-path")
                .required(true)
                .help("Path to the configuration file for the server"),
            Arg::new("port")
                .short('p')
                .long("port")
                .required(true)
                .help("The port the server will listen on"),
        ])
        .group(
            ArgGroup::new("server_type")
                .args(["primary", "secondary", "resolver"])
                .required(true),
        )
        .get_matches();

    //test if path exists
    let config_path = match arguments.get_one::<String>("config_path") {
        Some(path) => path,
        None => panic!("No config path provided."),
    };

    let port: u16 = match arguments.get_one::<String>("port") {
        Some(port) => match port.parse(){
            Ok(ok_port) => ok_port,
            Err(err) => panic!("{err}")
        },
        None => panic!("No port provided."),
    };

    match arguments.get_one::<Id>("server_type") {
        Some(server_type) => match server_type.as_str() {
            "primary" => sp::start_sp(config_path.to_owned(), port.to_owned()),
            "secondary" => ss::start_ss(config_path.to_owned(), port.to_owned()),
            "resolver" => sr::start_sr(config_path.to_owned(), port.to_owned()),
            _ => panic!("Server type does not exist.")
        },
        None => (),
    };
}
