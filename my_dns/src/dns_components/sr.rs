use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{self, Read},
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    ops::Add,
    path::{self, Path},
    time::Duration,
};

use log::{error, info};

use crate::{
    dns_make::{dns_recv::{self, RecvError}, dns_send},
    dns_parse::domain_database_parse::parse_root_servers,
    dns_structs::{dns_message::{DNSMessage, self}, server_config::ServerConfig, dns_domain_name::Domain},
};

pub fn start_sr(
    dns_query: &mut DNSMessage,
    server_list: Vec<SocketAddr>,
    supports_recursive: bool
) -> Result<DNSMessage, &'static str> {
    // Inicializar a socket UDP
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
    socket.set_write_timeout(Some(Duration::new(1, 0))).unwrap();
    
    if !supports_recursive {dns_query.header.flags -= 2};
    
    for server_ip in server_list {
        let _size_sent = match dns_send::send(dns_query.to_owned(), &socket, server_ip.to_string())
        {
            Ok(size_sent) => size_sent,
            Err(err) => {
                panic!("{err}");
            }
        };

        let (dns_recv_message, _src_addr) = match dns_recv::recv(&socket) {
            Ok(response) => response,
            Err(err) => match err {
                RecvError::IOError(_io_error) => {
                    error!("TO {} invalid-socket-address", server_ip.to_owned());
                    //When server doesn't respond back, we go to the next loop iteration
                    continue;
                }
                RecvError::DeserializeError(_deserialize_error) => {
                    error!("ER {} could-not-decode", server_ip.to_owned());
                    panic!("Could not decode received DNSMessage");
                }
            },
        };
        info!(
            "RR {} dns-msg-received: {}",
            server_ip.to_owned(),
            dns_recv_message.get_string()
        );

        match eval_and_respond(dns_query, dns_recv_message, &socket) {
            Ok(msg) => {
                info!(
                    "RR {} dns-msg-received: {}",
                    server_ip.to_owned(),
                    msg.get_string()
                );
                info!("SP 127.0.0.1 received-final-answer");
                return Ok(msg);
            }
            Err(err) => {
                error!("SP 127.0.0.1 {}", err);
                panic!("Received Invalid Answer")
            }
        }
    }
    return Err("Empty server list passed to SR");
}

fn eval_and_respond(
    dns_message: &mut DNSMessage,
    dns_recv_message: DNSMessage,
    socket: &UdpSocket,
) -> Result<DNSMessage, &'static str> {
    let mut return_message = Ok(DNSMessage::new());
    if let Some(response_code) = dns_recv_message.header.response_code {
        match response_code {
            // Codigo 0 => Mensagem de resposta valida
            0 => {
                info!("EV @ valid-dns-msg-received");
                return_message = Ok(dns_recv_message.clone());
            }
            // Codigo 1 =>  domínio existe mas não foi obtida a resposta de um servidor de autoridade
            1 => match dns_recv_message.data.authorities_values {
                // Existe pelo menos um servidor de autoridade para o dominio na resposta recebida
                Some(ref auth_values) => {
                    info!("EV @ non-authoritative-msg-received");
                    let mut new_ip;
                    for val in auth_values {
                        // Verificar se o valor do servidor de autoridade é um IP ou um nome
                        if !val.value.chars().all(|c| {
                            vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.'].contains(&c)
                        }) {
                            // Procurar o IP do servidor de autoridade na lista de valores extra
                            new_ip = match dns_recv_message.data.extra_values {
                                // Procurar na lista de valores extra o IP do servidor de autoridade
                                Some(ref extra_values) => {
                                    match extra_values.iter().clone().find(|extra| {
                                        extra.domain_name == Domain::new(val.value.to_string())
                                    }) {
                                        Some(ns) => ns.value.to_owned(),
                                        None => continue,
                                    }
                                }
                                // Nao foi encontrado nenhum valor extra
                                None => "No extra values found".to_string(),
                            };
                        } else {
                            new_ip = val.value.to_owned();
                        }
                        let addr_vec = new_ip.split(':').collect::<Vec<_>>();
                        let new_ip_address = match addr_vec.len() {
                            // Formar novo IP com o IP do servidor de autoridade e a porta 5353
                            1 => addr_vec[0].to_string().add(":").add("5353"),
                            // Formar novo IP com o IP obtido dos extra values e a porta recebida
                            2 => new_ip,
                            // Nao foi encontrado um IP valido
                            _ => {
                                error!(
                                    "SP 127.0.0.1 received-malformed-ip: {}",
                                    val.domain_name.to_string()
                                );
                                panic!("Malformed IP on {}", val.domain_name.to_string());
                            }
                        };

                        // Enviar a query para o novo IP
                        let _size_sent = match dns_send::send(
                            dns_message.to_owned(),
                            &socket,
                            new_ip_address.to_owned(),
                        ) {
                            Ok(size_sent) => {
                                info!("QE {} sent-new-query", new_ip_address.to_owned());
                                size_sent
                            }
                            Err(err) => {
                                error!("TO {} invalid-socket-address", new_ip_address.to_owned());
                                panic!("{err}");
                            }
                        };

                        // Receber a resposta
                        let (dns_recv_message_new, _src_addr) = match dns_recv::recv(&socket) {
                            Ok(response) => {
                                info!(
                                    "RR {} dns-msg-received: {}",
                                    new_ip_address.to_owned(),
                                    response.0.get_string()
                                );
                                response
                            }
                            Err(err) => match err {
                                RecvError::IOError(_io_error) => continue,
                                RecvError::DeserializeError(_deserialize_error) => {
                                    error!("ER {} could-not-decode", new_ip_address.to_owned());
                                    panic!("Could not deserialize received message")
                                }
                            },
                        };
                        // Recomecar o processo de verificar a resposta
                        return_message = eval_and_respond(dns_message, dns_recv_message_new, &socket);
                        break;
                    }
                }
                None => {}
            },
            2 => return Err("domain-not-found"),
            3 => return Err("malformed-query"),
            _ => return Err("response-code-invalid"),
        }
    }
    return_message
}
