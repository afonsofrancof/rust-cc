use log::{error, info};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    net::{IpAddr, SocketAddr},
    ops::Add,
    path::Path,
    str::FromStr,
};

use crate::dns_structs::{server_config::ServerConfig, dns_domain_name::Domain};

pub fn get(file_path: String) -> Result<ServerConfig, &'static str> {
    info!("Opening file");
    let mut file = match File::open(file_path.to_owned()) {
        Ok(file) => {
            info!("File opened successfully");
            file
        }
        Err(_err) => {
            error!("Failed to open file {}", file_path);
            return Err("Failed to open file");
        }
    };

    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();

    match file.read_to_string(&mut read) {
        Ok(_) => info!("Conversion to string successful"),
        Err(_err) => {
            error!("Couldn't read to string");
            return Err("Couldn't read to String");
        }
    };

    info!("Capturing variables");
    let regex_variables = Regex::new(r"(?m)^([a-z.0-9-]+) +(DB|SS|DD|LG|ST|SP) +(.*)").unwrap();

    info!("Starting variable parse");
    let mut server_config = ServerConfig::new();

    for cap in regex_variables.captures_iter(&read) {
        let name: String;
        if !cap[1].ends_with(".") && &cap[1]!="all" && &cap[1]!="root" {

            name = cap[1].to_string().add(".")
        } else {
            name = cap[1].to_string();
        }
        println!(
            "{} {} {}",
            name,
            cap[2].to_string(),
            cap[3].to_string()
        );
        match &cap[2] {
            "DB" => server_config.add_domain_db(Domain::new(name), cap[3].to_string()),
            "SS" => server_config.add_domain_ss(Domain::new(name), cap[3].to_string()),
            "SP" => server_config.set_domain_sp(Domain::new(name), cap[3].to_string()),
            "DD" => server_config.add_server_dd(Domain::new(name), cap[3].to_string()),
            "LG" => match &cap[1] {
                "all" => server_config.set_all_log(cap[3].to_string()),
                _ => server_config.set_domain_log(Domain::new(name), cap[3].to_string()),
            },
            "ST" => server_config.set_st_db(cap[3].to_string()),
            _ => (),
        }
    }

    Ok(server_config)
}

#[cfg(test)]
mod tests {
    use crate::dns_structs::dns_domain_name::Domain;

    #[test]
    fn test_config_parse() {
        let get_config = super::get("etc/test-config.conf".to_string());

        let parsed_config = match get_config {
            Ok(config) => config,
            Err(err) => panic!("{err}"),
        };

        let mut server_config = super::ServerConfig::new();
        server_config.add_domain_db(Domain::new("example.com.".to_owned()), "etc/example-com.db".to_string());
        server_config.add_domain_ss(Domain::new("example.com.".to_owned()), "193.123.5.189".to_owned());
        server_config.add_domain_ss(Domain::new("example.com.".to_owned()), "193.123.5.190:5353".to_owned());
        server_config.add_server_dd(Domain::new("example.com.".to_owned()), "127.0.0.1".to_owned());
        server_config.set_domain_log(Domain::new("example.com.".to_owned()), "logs/example-com.log".to_owned());
        server_config.set_all_log("logs/all.log".to_owned());
        server_config.set_st_db("etc/rootservers.db".to_owned());
        
        assert!(parsed_config==server_config);
    }
}
