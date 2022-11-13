#![feature(pattern)]
use clap::*;
use core::panic;
use my_dns::dns_components::sp::start_sp;
use my_dns::dns_components::sr::start_sr;
use my_dns::dns_components::ss::start_ss;
use my_dns::dns_parse::server_config_parse;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::hash::Hash;
use std::net::UdpSocket;
use std::ops::Add;
use std::str::pattern::Pattern;
use std::sync::mpsc::{channel, Sender};
use std::thread::{Builder, JoinHandle};

fn main() {
    // Argumentos de input da CLI para definir quais e quantos servidores inicializar
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("primary")
                .action(ArgAction::Append)
                .short('p')
                .long("primary")
                .help("Creates a primary DNS server to a domain"),
            Arg::new("secondary")
                .action(ArgAction::Append)
                .short('s')
                .long("secondary")
                .help("Creates a secondary DNS server to a domain"),
            Arg::new("resolver")
                .action(ArgAction::Count)
                .short('r')
                .long("resolver")
                .help("Creates a DNS resolver"),
            Arg::new("config_dir")
                .long("config_dir")
                .help("Directory where the config files are stored"),
        ])
        .get_matches();

    // Os maps onde vao estar guardadas os handlers das threads assim como o channel Sender
    // utilizado para enviar as DNSMessage para a thread recorrendo a key que
    //neste caso corresponde ao domain
    struct Servers {
        sp_threads: HashMap<String, (JoinHandle<()>, Sender<DNSMessage>)>,
        sp_configs: HashMap<String, ServerConfig>,
        ss_threads: HashMap<String, (JoinHandle<()>, Sender<DNSMessage>)>,
        ss_configs: HashMap<String, ServerConfig>,
        sr_threads: HashMap<u8, (JoinHandle<()>, Sender<DNSMessage>)>,
    }

    let mut sp_threads: HashMap<String, (JoinHandle<()>, Sender<DNSMessage>)> = HashMap::new();
    let mut ss_threads: HashMap<String, (JoinHandle<()>, Sender<DNSMessage>)> = HashMap::new();
    let mut sr_threads: HashMap<u8, (JoinHandle<()>, Sender<DNSMessage>)> = HashMap::new();
    let mut sp_configs: HashMap<String, ServerConfig> = HashMap::new();
    let mut ss_configs: HashMap<String, ServerConfig> = HashMap::new();

    let mut server_threads = Servers {
        sp_threads,
        sp_configs,
        ss_threads,
        ss_configs,
        sr_threads,
    };

    // Dos argumentos de input, guardar a diretoria onde se encontram as configs e databases
    let config_dir = match arguments.get_one::<String>("config_dir") {
        Some(config_dir_arg) => config_dir_arg,
        None => {
            panic!("No config directory specified")
        }
    };

    // Inicializar as threads por cada servidor primario que foi dado como input
    match arguments.get_many::<String>("primary") {
        Some(domains) => {
            for domain in domains {
                match Path::new(&config_dir)
                    .join(domain.clone().replace(".", "-").add(".conf"))
                    .to_str()
                {
                    Some(path) => match server_config_parse::get(path.to_string()) {
                        Ok(config_parsed) => {
                            let config = config_parsed;
                            sp_configs.insert(domain.clone(), config_parsed);
                            let (sender, receiver) = channel::<DNSMessage>();
                            let domain_name_cloned = domain.to_owned();
                            let thread_builder = Builder::new().name(format!("SP_{}", domain));
                            let thread_handle = thread_builder
                                .spawn(move || start_sp(domain_name_cloned, config, receiver))
                                .unwrap();
                            server_threads
                                .sp_threads
                                .insert(domain.to_owned(), (thread_handle, sender));
                        }
                        Err(err) => panic!("{err}"),
                    },
                    None => {
                        println!("No config file found for the SP of {}", domain_name);
                        continue;
                    }
                };
            }
        }
        None => println!("No primary domains received"),
    };

    // Inicializar as threads por cada servidor secundario que foi dado como input
    match arguments.get_many::<String>("secondary") {
        Some(domains) => {
            for domain in domains {
                match Path::new(&config_dir)
                    .join(domain.clone().replace(".", "-").add(".conf"))
                    .to_str()
                {
                    Some(path) => match server_config_parse::get(path.to_string()) {
                        Ok(config_parsed) => {
                            let config = config_parsed;
                            ss_configs.insert(domain.clone(), config_parsed);
                            let (sender, receiver) = channel::<DNSMessage>();
                            let domain_name_cloned = domain.to_owned();
                            let thread_builder = Builder::new().name(format!("SS_{}", domain));
                            let thread_handle = thread_builder
                                .spawn(move || start_ss(domain_name_cloned, config, receiver))
                                .unwrap();
                            server_threads
                                .ss_threads
                                .insert(domain.to_owned(), (thread_handle, sender));
                        }
                        Err(err) => panic!("{err}"),
                    },
                    None => {
                        println!("No config file found for the SS of {}", domain_name);
                        continue;
                    }
                };
            }
        }
        None => println!("No secondary domains received"),
    };

    // Inicializar as threads por cada resolver server que foi dado como input
    let num_of_resolvers = arguments.get_count("resolver");
    for resolver in 1..num_of_resolvers {
        let (sender, receiver) = channel::<DNSMessage>();
        let config_dir_cloned = config_dir.to_owned();
        let thread_builder = Builder::new().name(format!("SR_{}", resolver));
        let thread_handle = thread_builder
            .spawn(move || start_sr(config_dir_cloned, receiver))
            .unwrap();
        server_threads
            .sr_threads
            .insert(resolver, (thread_handle, sender));
    }

    // A socket de entrada de queries do "main server"
    let main_socket = match UdpSocket::bind("127.0.0.1:5454") {
        Ok(socket) => socket,
        Err(err) => panic!("{err}"),
    };

    // E nesta seccao que vao ser recebidas as queries, listener na socket
    // (loop infinito ate forcado a ser finalizado)
    loop {
        // TODO - Alterar o tamanho do buffer para o valor do pacote recebido com o peek(UDPSocket::peek)
        // Para o read do pacote recebido vamos usar o peek para consultar o tamanho do pacote
        // podendo deste modo criar um buffer com o tamanho correto
        let mut main_buffer = [0; 1000];

        // Receber o pacote pela socket, dando unwrap para dar handle nos erros
        // Ao recv_from falhar, prosseguimos para a recepcao do proximo pacote
        let (num_of_bytes, src_addr) = match main_socket.recv_from(&mut main_buffer) {
            Ok(nob_sa) => nob_sa,
            Err(_) => continue,
        };

        // Deserialize do pacote recebido e caso nao esteja correctamente formatado, ou
        // for invalido, vai ser descartado, acabando esta iteracao do ciclo de listen
        // e inicializando o seguinte
        let incoming_dns_query: DNSMessage = match bincode::deserialize(&main_buffer.to_vec()) {
            Ok(dns_message) => dns_message,
            Err(err) => {
                println!("Malformed query received");
                continue;
            }
        };

        // Valor usado para selecionar um resolver server randomly
        // TODO (nao faz muito sentido ter mais do que 1 SR, verificar isto mais late)
        let mut rng = rand::thread_rng();

        //
        // Confirmar a existencia do domain ou subdomain nesta "maquina/componente" dando
        // filter out as keys que nao correspondam a query que esta a ser feita
        //
        // Servidores primarios correspondentes a query
        let incoming_matches_sp = server_threads.sp_threads.keys().filter(|key| {
            ".".to_string()
                .add(incoming_dns_query.data.query_info.name.as_str())
                .ends_with(&format!(".{}", key))
        });

        // Servidores secundarios correspondentes a query
        let incoming_match_ss = server_threads.ss_threads.keys().filter(|key| {
            ".".to_string()
                .add(incoming_dns_query.data.query_info.name.as_str())
                .ends_with(&format!(".{}", key))
        });

        // Decidir qual thread (servidor) vai conseguir responder ao pedido
        // Primeiro confirmamos se existe algum servidor primario para dar handle ao pedido
        let (thread_handle, thread_sender) = match incoming_matches_sp.max() {
            Some(domain_name) => server_threads.sp_threads.get(domain_name).unwrap(),
            // Caso nao exista procuramos um servidor secundario
            None => match incoming_match_ss.max() {
                Some(domain_name) => server_threads.ss_threads.get(domain_name).unwrap(),
                // Caso nao exista nenhum primario ou secundario recorremos aos resolvers
                // Escolhendo um aleatoriamente
                None => match server_threads.sr_threads.values().choose(&mut rng) {
                    Some(handle_and_sender) => handle_and_sender,
                    None => {
                        println!("No component can answer your query");
                        continue;
                    }
                },
            },
        };

        // Enviamos a DNS Query para o servidor que ficou responsavel
        match thread_sender.send(incoming_dns_query) {
            Ok(_) => continue,
            Err(_err) => println!("Thread has closed it's receiver end of the channel"),
        };
    }
}
