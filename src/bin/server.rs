use clap::*;


fn main() {
    // Argumentos de input da CLI para definir quais e quantos servidores inicializar
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .group(
            ArgGroup::new("server_type")
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
            ])
            .required(true))
        .args([
            Arg::new("config_path")
                .long("config-path")
                .required(true)
                .help("Path to the configuration file for the server"),
            Arg::new("port")
                .short('p')
                .long("port")
                .required(true)
                .help("The port the server will listen on")
        ])
        .get_matches();
         
        
    match arguments.get_one::<String>("server_type"){
        Some(server_type) => println!("{}",server_type),
            None => ()
    }    
    
}
