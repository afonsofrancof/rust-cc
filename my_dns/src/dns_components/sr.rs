use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use log::{debug, error, info};


use crate::{
    dns_make::{
        dns_recv::{self, RecvError},
        dns_send,
    },
    dns_structs::dns_message::DNSMessage,
};

pub fn resolver(
    dns_query: &mut DNSMessage,
    server_list: Vec<SocketAddr>,
    supports_recursive: bool,
) -> Result<DNSMessage, &'static str> {
    // Inicializar a socket UDP
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_read_timeout(Some(Duration::new(1, 0))).unwrap();
    socket.set_write_timeout(Some(Duration::new(1, 0))).unwrap();

    if !supports_recursive {
        dns_query.header.flags -= 2
    };

    if server_list.is_empty() {
        return Err("Empty server list provided");
    };

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
                    error!("ER pdu-deserialize-fail {}", server_ip.to_owned());
                    panic!("Could not decode received DNSMessage");
                }
            },
        };

        match eval_and_respond(dns_query, dns_recv_message,supports_recursive) {
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
    return Err("No servers answered your query");
}

fn eval_and_respond(
    dns_message: &mut DNSMessage,
    dns_recv_message: DNSMessage,
    supports_recursive: bool,
) -> Result<DNSMessage, &'static str> {
    let mut return_message = Ok(DNSMessage::new());
    if let Some(response_code) = dns_recv_message.header.response_code {
        match response_code {
            // Codigo 0 => Mensagem de resposta valida
            // Codigo 2 => domínio não existe.
            // Codigo 3 => Malformed message.
            0 | 2 | 3 => {
                return_message = Ok(dns_recv_message.clone());
            }
            // Codigo 1 =>  domínio existe mas não foi obtida a resposta de um servidor de autoridade
            1 => match dns_recv_message.data.authorities_values {
                // Existe pelo menos um servidor de autoridade para o dominio na resposta recebida
                Some(ref auth_values) => {
                    debug!("EV @ non-authoritative-msg-received");

                    let ip_vec = match DNSMessage::get_authorities_ip(
                        &dns_message,
                        dns_recv_message.data.extra_values.to_owned(),
                        dns_message.data.query_info.name.to_owned(),
                        auth_values.to_vec(),
                    ) {
                        Some(vec) => vec,
                        None => {
                            panic!("No NS found for the queried domain, cannot get answer")
                        }
                    };

                    return_message = resolver(dns_message, ip_vec, supports_recursive);
                }
                None => {}
            },
            _ => return Err("response-code-invalid"),
        }
    }
    return_message
}
