use std::collections::HashMap;

pub struct ServerConfig{
    pub domain_name: String,
    pub domain_db: String,
    pub domain_ss: Vec<String>,
    pub server_dd: HashMap<String,String>,
    pub domain_log: String,
    pub all_log: String,
    pub st_db: String
}


