use std::{collections::HashMap, net::SocketAddr, ops::Add};

#[derive(Clone)]
pub struct ServerConfig {
    domain_configs: HashMap<String, DomainConfig>,
    server_dds: Option<HashMap<String, SocketAddr>>,
    all_log: String,
    st_db: String,
}
#[derive(Clone)]
pub struct DomainConfig {
    domain_db: Option<String>,
    domain_sp: Option<SocketAddr>,
    domain_ss: Option<Vec<SocketAddr>>,
    domain_log: String,
}

impl ServerConfig {
    pub fn new() -> Self {
        ServerConfig {
            domain_configs: HashMap::new(),
            server_dds: None,
            all_log: "".to_string(),
            st_db: "".to_string(),
        }
    }
    pub fn add_domain_db(&mut self, domain: String, db_path: String) {
        match  self.domain_configs.get_mut(&domain) {
            Some(domain_config) => {
                domain_config.domain_db = Some(db_path);
            }
            None => {
                let dc = DomainConfig {
                    domain_db: Some(db_path),
                    domain_sp: None,
                    domain_ss: None,
                    domain_log: "".to_string(),
                };
                self.domain_configs.insert(domain, dc);
            }
        }
    }
    pub fn set_domain_sp(&mut self, domain: String, addr_string: String) {
        let addr_vec = addr_string.split(':').collect::<Vec<_>>();
        let addr_string_parsed = match addr_vec.len(){
            1 => addr_vec[0].to_string().add(":").add("5353"),
            2 => addr_vec[0].to_string().add(":").add(&addr_vec[1].to_string()),
            _ => panic!("Malformed IP on {domain}'s SP entry")
        };
        let addr = match SocketAddr::parse_ascii(addr_string_parsed.as_bytes()){
            Ok(addr) => addr,
            Err(_) => panic!("Could not parse {domain} SP's IP") 
        };
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => {
                domain_config.domain_sp = Some(addr);
            }
            None => {
                let dc = DomainConfig {
                    domain_db: None,
                    domain_sp: Some(addr),
                    domain_ss: None,
                    domain_log: "".to_string(),
                };
                self.domain_configs.insert(domain, dc);
            }
        }
    }
    pub fn add_domain_ss(&mut self, domain: String, addr_string: String) {
        let addr_vec = addr_string.split(':').collect::<Vec<_>>();
        let addr_string_parsed = match addr_vec.len(){
            1 => addr_vec[0].to_string().add(":").add("5353"),
            2 => addr_vec[0].to_string().add(":").add(&addr_vec[1].to_string()),
            _ => panic!("Malformed IP on {domain}'s SS entry")
        };
        let addr = match SocketAddr::parse_ascii(addr_string_parsed.as_bytes()){
            Ok(addr) => addr,
            Err(_) => panic!("Could not parse an SP IP from {domain}") 
        };
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => match &mut domain_config.domain_ss {
                Some(vector) => vector.push(addr),
                None => domain_config.domain_ss = Some(vec![addr]),
            },
            None => {
                let dc = DomainConfig {
                    domain_db: None,
                    domain_sp: None,
                    domain_ss: Some(vec![addr]),
                    domain_log: "".to_string(),
                };
                self.domain_configs.insert(domain, dc);
            }
        };
    }

    pub fn add_server_dd(&mut self, domain: String, addr_string: String) {
        let addr_vec = addr_string.split(':').collect::<Vec<_>>();
        let addr_string_parsed = match addr_vec.len(){
            1 => addr_vec[0].to_string().add(":").add("5353"),
            2 => addr_vec[0].to_string().add(":").add(&addr_vec[1].to_string()),
            _ => panic!("Malformed IP on DD entry for {domain}")
        };
        let addr = match SocketAddr::parse_ascii(addr_string_parsed.as_bytes()){
            Ok(addr) => addr,
            Err(_) => panic!("Could not parse one of the Server's DD fields") 
        };
        match &mut self.server_dds {
            Some(server_dds) => {
                server_dds.insert(domain, addr);
            }
            None => {
                let mut hm = HashMap::new();
                hm.insert(domain, addr);
                self.server_dds = Some(hm);
            }
        };
    }
    pub fn set_domain_log(&mut self, domain: String, domain_log: String) {
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => {
                domain_config.domain_log = domain_log;
            }
            None => {
                let dc = DomainConfig {
                    domain_db: None,
                    domain_sp: None,
                    domain_ss: None,
                    domain_log,
                };
                self.domain_configs.insert(domain, dc);
            }
        }
    }
    pub fn set_all_log(&mut self, all_log: String) {
        self.all_log = all_log;
    }
    pub fn set_st_db(&mut self, path: String) {
        self.st_db = path;
    }

    pub fn get_domain_configs(&self) -> HashMap<String, DomainConfig>{
        self.domain_configs.to_owned()
    }
}

impl DomainConfig{
    pub fn get_domain_db(&self) -> Option<String> {
        self.domain_db.to_owned()
    }
}
