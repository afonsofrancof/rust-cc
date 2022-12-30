use log::{debug, error};
use regex::Regex;
use std::{fs::File, io::Read, ops::Add};

use crate::dns_structs::{dns_domain_name::Domain, server_config::ServerConfig};

pub fn get(file_path: String) -> Result<ServerConfig, &'static str> {
    let mut file = match File::open(file_path.to_owned()) {
        Ok(file) => {
            debug!("EV @ config-file-opened {}", file_path);
            file
        }
        Err(_err) => {
            error!("SP @ incorrect-config-file-path {}", file_path);
            return Err("Failed to open file");
        }
    };

    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();

    match file.read_to_string(&mut read) {
        Ok(_) => debug!("EV @ read-config-file"),
        Err(_err) => {
            error!("SP @ unable-to-read-config-file");
            return Err("Couldn't read to String");
        }
    };

    debug!("EV @ capturing-regex-variables");
    let regex_variables = Regex::new(r"(?m)^([a-z.0-9-]+) +(DB|SS|DD|LG|ST|SP) +(.*)").unwrap();

    let mut server_config = ServerConfig::new();

    for cap in regex_variables.captures_iter(&read) {
        let name: String;
        if !cap[1].ends_with(".") && &cap[1] != "all" && &cap[1] != "root" {
            name = cap[1].to_string().add(".")
        } else {
            name = cap[1].to_string();
        }
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
    debug!("EV @ config-file-parsed");
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
        server_config.add_domain_db(
            Domain::new("example.com.".to_owned()),
            "etc/example-com.db".to_string(),
        );
        server_config.add_domain_ss(
            Domain::new("example.com.".to_owned()),
            "193.123.5.189".to_owned(),
        );
        server_config.add_domain_ss(
            Domain::new("example.com.".to_owned()),
            "193.123.5.190:5353".to_owned(),
        );
        server_config.add_server_dd(
            Domain::new("example.com.".to_owned()),
            "127.0.0.1".to_owned(),
        );
        server_config.set_domain_log(
            Domain::new("example.com.".to_owned()),
            "logs/example-com.log".to_owned(),
        );
        server_config.set_all_log("logs/all.log".to_owned());
        server_config.set_st_db("etc/rootservers.db".to_owned());

        assert!(parsed_config == server_config);
    }
}
