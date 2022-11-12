use std::{collections::HashMap, fs::File, path::Path, io::Read};
use regex::Regex;
use crate::dns_structs::server_config::ServerConfig;
use log::{info,error};

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
        Regex::new(r"(?m)^([a-z.0-9-]+) (DB|SS|DD|LG|ST) (.*)").unwrap();
    
    let mut domain_name: String = String::new();
    let mut domain_db: String = String::new();
    let mut domain_ss: Vec<String> = Vec::new();
    let mut server_dd: HashMap<String,String> = HashMap::new();
    let mut domain_log: String = String::new();
    let mut all_log: String = String::new();
    let mut st_db: String = String::new();
    
    info!("Starting variable parse");

    for cap in regex_variables.captures_iter(&read){
        match &cap[2]{
            "DB" => {domain_name = cap[1].to_string();domain_db = cap[3].to_string()},
            "SS" => domain_ss.push(cap[3].to_string()),
            "DD" => {server_dd.insert(cap[1].to_string(),cap[3].to_string());},
            "LG" => match &cap[1]{
                "all" => all_log = cap[3].to_string(),
                _ => domain_log = cap[3].to_string(), 
            },
            "ST" => st_db = cap[3].to_string(),
            _ => ()
        } 
    }

    Ok(ServerConfig{domain_name,domain_db,domain_ss,server_dd,domain_log,all_log,st_db})
} 
