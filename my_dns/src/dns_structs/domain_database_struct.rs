use std::collections::HashMap;

pub struct DomainDatabase {
    pub config_list: HashMap<String, Entry>,
    pub entry_list: HashMap<String, Vec<Entry>>,
}

pub struct Entry {
    pub name: String,
    pub entry_type: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}
