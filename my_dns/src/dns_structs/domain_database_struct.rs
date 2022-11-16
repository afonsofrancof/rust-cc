use std::collections::HashMap;
#[derive(Clone)]
pub struct DomainDatabase {
    pub config_list: HashMap<String, Entry>,
    pub  domain_name_servers: Option<Vec<Entry>>,
    pub  subdomain_name_servers: Option<Vec<Entry>>,
    pub  a_records: Option<Vec<Entry>>,
    pub  cname_records: Option<Vec<Entry>>,
    pub  mx_records: Option<Vec<Entry>>,
    pub  ptr_records: Option<Vec<Entry>>,
}
#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub entry_type: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}
