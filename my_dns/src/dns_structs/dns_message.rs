use log::{error, debug};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, ops::Add};

use super::{dns_domain_name::Domain, domain_database_struct::DomainDatabase};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DNSMessage {
    pub header: DNSMessageHeaders,
    pub data: DNSMessageData,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct DNSMessageHeaders {
    pub message_id: u16,
    pub flags: u8,
    pub response_code: Option<u8>,
    pub number_of_values: Option<u8>,
    pub number_of_authorities: Option<u8>,
    pub number_of_extra_values: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DNSMessageData {
    pub query_info: DNSQueryInfo,
    pub response_values: Option<Vec<DNSEntry>>,
    pub authorities_values: Option<Vec<DNSEntry>>,
    pub extra_values: Option<Vec<DNSEntry>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSQueryInfo {
    pub name: Domain,
    pub type_of_value: QueryType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSEntry {
    pub domain_name: Domain,
    pub type_of_value: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum QueryType {
    NS,
    A,
    CNAME,
    MX,
    PTR,
}

impl DNSMessage {
    pub fn new() -> Self {
        let header = DNSMessageHeaders::new();
        let data = DNSMessageData::new();
        DNSMessage { header, data }
    }

    pub fn get_string(&self) -> String {
        let mut res = String::new();

        res.push_str(self.header.get_string().as_str());
        res.push_str(self.data.get_string().as_str());
        res
    }

    pub fn get_authorities_ip(&self, entries: Option<Vec<DNSEntry>>,queried_domain:Domain,list_of_authorities:Vec<DNSEntry>) -> Option<Vec<SocketAddr>> {
        let mut ip_vec: Vec<SocketAddr> = Vec::new();
        let mut new_ip;
        for val in list_of_authorities{
            // Verificar se o valor do servidor de autoridade é um IP ou um nome
            if !val
                .value
                .chars()
                .all(|c| vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.'].contains(&c))
            {
                // Procurar o IP do servidor de autoridade na lista de valores extra
                new_ip =
                    match entries {
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
                        None => "No A records found to translate".to_string(),
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
                        "FL @ malformed-ip {} {}",
                        val.domain_name.to_string(),
                        queried_domain.to_string()
                    );
                    panic!("Malformed IP on {}", val.domain_name.to_string());
                }
            };
            debug!(
                "EV @ ns-ip-found {} {}",
                new_ip_address,
                queried_domain.to_string()
            );
            ip_vec.push(new_ip_address.parse().unwrap());
        }
        if ip_vec.is_empty() {return None} else {return Some(ip_vec)}
    }
}

impl DNSMessageHeaders {
    pub fn new() -> Self {
        DNSMessageHeaders {
            message_id: rand::random(),
            flags: 0,
            response_code: None,
            number_of_values: None,
            number_of_authorities: None,
            number_of_extra_values: None,
        }
    }

    // O sistema de flags funciona em binario em que se soma o valor de todas as flags
    // A   => 0 0 1 = 1
    // R   => 0 1 0 = 2
    // Q   => 1 0 0 = 4
    // R+A => 0 1 1 = 3
    // Q+R => 1 1 0 = 6
    pub fn decode_flags(&self) -> Result<&str, &'static str> {
        match self.flags {
            0 => Ok(""),
            1 => Ok("A"),
            2 => Ok("R"),
            4 => Ok("Q"),
            3 => Ok("R+A"),
            6 => Ok("Q+R"),
            _ => Err("Flag value does not match any combination of flags"),
        }
    }

    pub fn get_string(&self) -> String {
        let mut rc: u8 = 0;
        let mut nov: u8 = 0;
        let mut noa: u8 = 0;
        let mut noev: u8 = 0;

        if let Some(i) = self.response_code {
            rc = i
        };
        if let Some(i) = self.number_of_values {
            nov = i
        };
        if let Some(i) = self.number_of_authorities {
            noa = i
        };
        if let Some(i) = self.number_of_extra_values {
            noev = i
        };

        let flags = match self.decode_flags() {
            Ok(flag) => flag,
            Err(err) => panic!("{err}"),
        };

        format!(
            "{},{},{},{},{},{};",
            self.message_id, flags, rc, nov, noa, noev
        )
    }
}

impl DNSMessageData {
    pub fn new() -> Self {
        let dns_query_info = DNSQueryInfo::new();
        DNSMessageData {
            query_info: dns_query_info,
            response_values: None,
            authorities_values: None,
            extra_values: None,
        }
    }

    pub fn get_string(&self) -> String {
        let mut res = String::new();

        res.push_str(self.query_info.get_string().as_str());

        let rv = match &self.response_values {
            Some(hm) => {
                let vec_str: Vec<String> = hm.iter().map(|x| x.get_string()).collect();
                let mut sb: String = String::new();
                for mut entry in vec_str {
                    entry.push_str(",");
                    sb.push_str(entry.as_str());
                }
                if sb.len() > 0 {
                    sb.replace_range(sb.len() - 1..sb.len(), ";")
                };
                sb
            }
            None => String::new(),
        };
        res.push_str(rv.as_str());

        let av = match &self.authorities_values {
            Some(vec) => {
                let vec_str: Vec<String> = vec.iter().map(|x| x.get_string()).collect();
                let mut sb: String = String::new();
                for mut entry in vec_str {
                    entry.push_str(",");
                    sb.push_str(entry.as_str());
                }
                if sb.len() > 0 {
                    sb.replace_range(sb.len() - 1..sb.len(), ";")
                };
                sb
            }
            None => String::new(),
        };
        res.push_str(av.as_str());

        let ev = match &self.extra_values {
            Some(vec) => {
                let vec_str: Vec<String> = vec.iter().map(|x| x.get_string()).collect();
                let mut sb: String = String::new();
                for mut entry in vec_str {
                    entry.push_str(",");
                    sb.push_str(entry.as_str());
                }
                if sb.len() > 0 {
                    sb.replace_range(sb.len() - 1..sb.len(), ";")
                };
                sb
            }
            None => String::new(),
        };
        res.push_str(ev.as_str());
        res
    }
}

impl DNSQueryInfo {
    pub fn new() -> Self {
        DNSQueryInfo {
            name: Domain::new_empty(),
            type_of_value: QueryType::A,
        }
    }

    pub fn get_string(&self) -> String {
        let tov: String = self.type_of_value.get_str().to_string();
        format!("{},{};", self.name.to_string(), tov)
    }
}

impl DNSEntry {
    pub fn new() -> Self {
        DNSEntry {
            domain_name: Domain::new_empty(),
            type_of_value: String::new(),
            value: String::new(),
            ttl: 0,
            priority: None,
        }
    }

    pub fn get_value(&self) -> String {
        self.value.to_owned()
    }

    // falta a priority
    pub fn get_string(&self) -> String {
        if let Some(priority) = self.priority {
            format!(
                "{} {} {} {} {}",
                self.domain_name.to_string(),
                self.type_of_value,
                self.value,
                self.ttl,
                priority
            )
        } else {
            format!(
                "{} {} {} {}",
                self.domain_name.to_string(),
                self.type_of_value,
                self.value,
                self.ttl
            )
        }
    }
}

impl QueryType {
    pub fn get_str(&self) -> &'static str {
        match self {
            QueryType::NS => "NS",
            QueryType::A => "A",
            QueryType::CNAME => "CNAME",
            QueryType::MX => "MX",
            QueryType::PTR => "PTR",
        }
    }

    pub fn from_string(query_type: String) -> Result<QueryType, String> {
        match query_type.as_str() {
            "NS" => Ok(QueryType::NS),
            "A" => Ok(QueryType::A),
            "CNAME" => Ok(QueryType::CNAME),
            "MX" => Ok(QueryType::MX),
            "PTR" => Ok(QueryType::PTR),
            _ => Err(format!("Cannot find QueryType of {}", query_type)),
        }
    }
}
