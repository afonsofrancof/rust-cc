use std::{collections::HashMap, net::SocketAddr, ops::Add};

use super::dns_domain_name::Domain;

#[derive(Clone,PartialEq)]
pub struct ServerConfig {
    domain_configs: HashMap<Domain, DomainConfig>,
    server_dds: Option<HashMap<Domain, SocketAddr>>,
    all_log: String,
    st_db: String,
}
#[derive(Clone,PartialEq)]
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
    pub fn add_domain_db(&mut self, domain: Domain, db_path: String) {
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => {
                domain_config.set_domain_db(db_path);
            }
            None => {
                let mut dc = DomainConfig::new();
                dc.set_domain_db(db_path);
                self.domain_configs.insert(domain, dc);
            }
        }
    }
    pub fn set_domain_sp(&mut self, domain: Domain, addr_string: String) {
        let addr_vec = addr_string.split(':').collect::<Vec<_>>();
        let addr_string_parsed = match addr_vec.len() {
            1 => addr_vec[0].to_string().add(":").add("8000"),
            2 => addr_string,
            _ => panic!("Malformed IP on {}'s SP entry",domain.to_string()),
        };
        println!("Address string parsed: {}", addr_string_parsed);
        let addr = match SocketAddr::parse_ascii(addr_string_parsed.as_bytes()) {
            Ok(addr) => addr,
            Err(_) => panic!("Could not parse {} SP's IP",domain.to_string()),
        };
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => {
                domain_config.set_domain_sp(addr);
            }
            None => {
                let mut dc = DomainConfig::new();
                dc.set_domain_sp(addr);
                self.domain_configs.insert(domain, dc);
            }
        };
    }
    pub fn add_domain_ss(&mut self, domain: Domain, addr_string: String) {
        let addr_vec = addr_string.split(':').collect::<Vec<_>>();
        let addr_string_parsed = match addr_vec.len() {
            1 => addr_vec[0].to_string().add(":").add("5353"),
            2 => addr_string,
            _ => panic!("Malformed IP on {}'s SS entry",domain.to_string()),
        };
        let addr = match SocketAddr::parse_ascii(addr_string_parsed.as_bytes()) {
            Ok(addr) => addr,
            Err(_) => panic!("Could not parse an SP IP from {}",domain.to_string()),
        };
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => domain_config.add_domain_ss(addr),
            None => {
                let mut dc = DomainConfig::new();
                dc.add_domain_ss(addr);
                self.domain_configs.insert(domain, dc);
            }
        };
    }
    pub fn set_domain_log(&mut self, domain: Domain, domain_log: String) {
        match self.domain_configs.get_mut(&domain) {
            Some(domain_config) => {
                domain_config.set_domain_log(domain_log);
            }
            None => {
                let mut dc = DomainConfig::new();
                dc.set_domain_log(domain_log);
                self.domain_configs.insert(domain, dc);
            }
        }
    }
    pub fn add_server_dd(&mut self, domain: Domain, addr_string: String) {
        let addr_vec = addr_string.split(':').collect::<Vec<_>>();
        let addr_string_parsed = match addr_vec.len() {
            1 => addr_vec[0].to_string().add(":").add("5353"),
            2 => addr_string,
            _ => panic!("Malformed IP on DD entry for {}",domain.to_string()),
        };
        let addr = match SocketAddr::parse_ascii(addr_string_parsed.as_bytes()) {
            Ok(addr) => addr,
            Err(_) => panic!("Could not parse one of the Server's DD fields"),
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
    
    pub fn set_all_log(&mut self, all_log: String) {
        self.all_log = all_log;
    }
    pub fn set_st_db(&mut self, path: String) {
        self.st_db = path;
    }
    pub fn get_domain_configs(&self) -> HashMap<Domain, DomainConfig> {
        self.domain_configs.to_owned()
    }
    pub fn get_all_ss(&self) -> Vec<SocketAddr> {
        let mut all_ss: Vec<SocketAddr> = Vec::new();
        for domain_config in self.domain_configs.values() {
            match domain_config.get_domain_ss() {
                Some(ss_vec) => {
                    let mut local_ss = ss_vec.clone();
                    all_ss.append(&mut local_ss);
                },
                None => continue
            }
        }
        return all_ss;
    }
    pub fn get_all_log(&self) -> String{
        self.all_log.to_owned()
    }
    pub fn get_st_db(&self) -> String {
        self.st_db.to_owned()
    }

}

impl DomainConfig {
    pub fn new() -> Self {
        DomainConfig {
            domain_db: None,
            domain_sp: None,
            domain_ss: None,
            domain_log: "".to_string(),
        }
    }
    pub fn get_domain_db(&self) -> Option<String> {
        self.domain_db.to_owned()
    }
    pub fn get_domain_sp(&self) -> Option<SocketAddr> {
        self.domain_sp.to_owned()
    }
    pub fn get_domain_ss(&self) -> Option<Vec<SocketAddr>> {
        self.domain_ss.to_owned()
    }
    pub fn get_domain_log(&self) -> String {
        self.domain_log.to_owned()
    }

    pub fn set_domain_db(&mut self, db_path: String) {
        self.domain_db = Some(db_path);
    }
    pub fn set_domain_sp(&mut self, sp_addr: SocketAddr) {
        self.domain_sp = Some(sp_addr);
    }
    pub fn add_domain_ss(&mut self, ss_addr: SocketAddr) {
        match &mut self.domain_ss {
            Some(servers) => servers.push(ss_addr),
            None => self.domain_ss = Some(vec![ss_addr]),
        }
    }
    pub fn set_domain_log(&mut self, log_path: String) {
        self.domain_log = log_path;
    }
}
