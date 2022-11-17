use std::{collections::HashMap, fs::File, path::Path, io::Read, net::SocketAddr};
use regex::Regex;
use log::{info,error};

use crate::dns_structs::server_config::ServerConfig;

pub fn get(file_path: String) -> Result<ServerConfig,&'static str>{
    
    info!("Opening file");
    let mut file = match File::open(file_path.to_owned()) {
        Ok(file) => {
            info!("File opened successfully");
            file
        },
        Err(_err) => {
            error!("Failed to open file {}",file_path);
            return Err("Failed to open file") 
        },
    };

    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();

    match file.read_to_string(&mut read) {
        Ok(_) => info!("Conversion to string successful"),
        Err(_err) => {
            error!("Couldn't read to string");
            return Err("Couldn't read to String")
        },
    };
    
    info!("Capturing variables");
    let regex_variables =
        Regex::new(r"(?m)^([a-z.0-9-]+) (DB|SS|DD|LG|ST|SP) (.*)").unwrap();
    
    info!("Starting variable parse");
    let mut server_config = ServerConfig::new();

    for cap in regex_variables.captures_iter(&read){
        match &cap[2]{
            "DB" => {server_config.add_domain_db(cap[1].to_string(), cap[3].to_string())},
            "SS" => {server_config.add_domain_ss(cap[1].to_string(),cap[3].to_string())},
            "DD" => {server_config.add_server_dd(cap[1].to_string(),cap[3].to_string())},
            "LG" => match &cap[1]{
                "all" => {server_config.set_all_log(cap[3].to_string())},
                _ => {server_config.set_domain_log(cap[1].to_string(),cap[3].to_string())}, 
            },
            "ST" => {server_config.set_st_db(cap[3].to_string())},
            _ => ()
        } 
    }

    Ok(server_config)
} 
