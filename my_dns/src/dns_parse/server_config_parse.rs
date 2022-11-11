use std::{collections::HashMap, fs::File, path::Path, io::Read};
use regex::Regex;
use crate::dns_structs::server_config::ServerConfig;
pub fn get(file_path: String) -> Result<ServerConfig,&'static str>{
    let mut file = match File::open(file_path) {
        Err(_err) => return Err("GEY"),
        Ok(file) => file,
    };

    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();

    match file.read_to_string(&mut read) {
        Err(_err) => return Err("Couldn't Read to String"),
        Ok(_) => (),
    };

    let regex_variables =
        Regex::new(r"(?m)^([a-z.0-9-]+) (DB|SS|DD|LG|ST) (.*)").unwrap();

    

    
    let mut domain_name: String = String::new();
    let mut domain_db: String = String::new();
    let mut domain_ss: Vec<String> = Vec::new();
    let mut server_dd: HashMap<String,String> = HashMap::new();
    let mut domain_log: String = String::new();
    let mut all_log: String = String::new();
    let mut st_db: String = String::new();

    for cap in regex_variables.captures_iter(&read){
        match &cap[2]{
            "DB" => domain_db = cap[3].to_string(),
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
