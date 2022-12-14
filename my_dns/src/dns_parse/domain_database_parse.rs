use crate::dns_structs::dns_domain_name::Domain;
use crate::dns_structs::dns_message::{DNSEntry, QueryType};
use crate::dns_structs::domain_database_struct::DomainDatabase;
use core::panic;
use std::net::SocketAddr;
use log::{debug, error, info, warn};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Add;

pub fn get(file_path: String) -> Result<DomainDatabase, &'static str> {
    // Abrir o ficheiro de database para leitura
    let mut file = match File::open(file_path.to_owned()) {
        Ok(file) => {
            debug!("EV @ db-file-open {}", file_path);
            file
        }
        Err(_err) => {
            error!("SP @ database-parse-error {}", file_path);
            return Err("Couldn't open file");
        }
    };
    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();

    match file.read_to_string(&mut read) {
        Ok(_) => {}
        Err(_err) => return Err("Couldn't Read to String"),
    };

    let domain_database = match parse_from_str(read) {
        Ok(database) => Ok(database),
        Err(_err) => {
            debug!("FL @ unable-to-read-db-file {}", file_path);
            Err("Error while reading Database")
        }
    };
    domain_database
}

pub fn parse_from_str(read: String) -> Result<DomainDatabase, &'static str> {
    let regex_variables =
        Regex::new(r"(?m)^([@A-Za-z.0-9-]+) +DEFAULT +([A-Za-z.0-9\\.-]+)").unwrap();

    let regex_soa = Regex::new(
        r"(?m)^([@A-Za-z.0-9-]+) +(SOA[A-Z]+) +([A-Za-z.0-9\\.-]+) +([A-Z0-9]+) *([A-Z0-9]+)?",
    )
    .unwrap();

    let regex_entry = Regex::new(
        r"(?m)^([@A-Za-z.0-9-]+) +(NS|A|CNAME|MX|PTR) +([A-Za-z.0-9\\.-]+) +([A-Z0-9]+) *([A-Z0-9]+)?",
    )
    .unwrap();

    // Deste modo, os comentario ficam todos ignorados visto que as expressoes capturam apenas as expressoes no inicio da linha

    // HashMaps onde vamos guardar os valores para dar return
    // Mapa com o nome da variavel como key
    let mut variables: HashMap<String, String> = HashMap::new();

    // Capturar todas as variaveis primeiro pois vao ser usadas nos outros mapas para substituir os defaults
    for cap in regex_variables.captures_iter(&read) {
        variables.insert(cap[1].to_string(), cap[2].to_string());
    }
    debug!("EV @ variable-regex-capture");

    // Mapa que vai conter todas as SOAs entries tendo o tipo de SOA como key (aka SOAADMIN,SOAEXPIRE, etc)
    let mut domain_database = DomainDatabase::new();
    // Capturar todas as SOAs entries
    for cap in regex_soa.captures_iter(&read) {
        // Podemos fazer error check nesta seccao do codigo
        let mut name: String = cap[1].to_string();
        let type_of_value: String = cap[2].to_string();
        let value: String = cap[3].to_string();
        let mut temp_ttl: String = cap[4].to_string();
        let priority: Option<u16> = match cap.get(5) {
            Some(p) => Some(p.as_str().parse::<u16>().unwrap()),
            _ => None,
        };
        for (variable, value) in variables.iter() {
            name = name.replace(variable, value);
            temp_ttl = temp_ttl.replace(variable, value).parse().unwrap();
        }

        let ttl: u32 = temp_ttl.parse().unwrap();
        let entry = DNSEntry {
            domain_name: Domain::new(name),
            type_of_value,
            value,
            ttl,
            priority,
        };
        match &cap[2] {
            "SOASP" => domain_database.soa_entries.primary_ns = entry,
            "SOAADMIN" => domain_database.soa_entries.contact_email = entry,
            "SOASERIAL" => domain_database.soa_entries.serial = entry,
            "SOAREFRESH" => domain_database.soa_entries.refresh = entry,
            "SOARETRY" => domain_database.soa_entries.retry = entry,
            "SOAEXPIRE" => domain_database.soa_entries.expire = entry,
            _ => {
                debug!("FL @ invalid-soa-entry");
                return Err("SOA type does not exist");
            }
        }
    }

    // Capturar todas as entries
    for cap in regex_entry.captures_iter(&read) {
        // Podemos fazer error check nesta seccao do codigo
        let mut name: String = cap[1].to_string();
        let type_of_value: String = cap[2].to_string();
        let value: String = cap[3].to_string();
        let mut temp_ttl: String = cap[4].to_string();
        let priority: Option<u16> = match cap.get(5) {
            Some(p) => match p.as_str().parse::<u16>() {
                Ok(nmbr) => Some(nmbr),
                Err(err) => panic!("{err}"),
            },
            _ => None,
        };
        for (variable, value) in variables.iter() {
            name = name.replace(variable, value);
            temp_ttl = temp_ttl.replace(variable, value).parse().unwrap();
        }

        if !name.ends_with(".") {
            let main_domain = match variables.get("@") {
                Some(value) => value,
                None => {
                    debug!("FL @ incomplete-domain-name {}", name);
                    return Err(
                        "Non complete domain name found in entry and no @ variable defined",
                    );
                }
            };
            name = name.add(".").add(main_domain);
        }

        let ttl: u32 = temp_ttl.parse().unwrap();

        let temp_entry: DNSEntry = DNSEntry {
            domain_name: Domain::new(name.to_owned()),
            type_of_value: type_of_value.to_owned(),
            value,
            ttl,
            priority,
        };

        match type_of_value.as_str() {
            "NS" => domain_database.add_ns_record(Domain::new(name), temp_entry),
            "A" => domain_database.add_a_record(temp_entry),
            "CNAME" => domain_database.add_cname_record(temp_entry),
            "MX" => domain_database.add_mx_record(temp_entry),
            "PTR" => domain_database.add_ptr_record(temp_entry),
            _ => continue,
        }
    }

    domain_database.authority = true; 
    Ok(domain_database)
}


pub fn parse_root_servers(root_servers_path: String) -> Result<Vec<SocketAddr>, std::io::Error> {
    let mut file = match File::open(root_servers_path.to_owned()) {
        Ok(file) => file,
        Err(err) => {
            return Err(err);
        }
    };
    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();
    
    match file.read_to_string(&mut read) {
        Ok(_) => {}
        Err(err) => {
            return Err(err);
        }
    };

    let mut root_vec: Vec<SocketAddr> = Vec::new();

    for root in read.lines() {
        let root_ip: SocketAddr = match root.parse() {
            Ok(ip) => ip,
            Err(_err) => panic!("Malformed root server IP at {}", root_servers_path),
        };
        root_vec.push(root_ip);
    }
    Ok(root_vec)
}
