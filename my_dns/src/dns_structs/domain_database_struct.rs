use std::collections::HashMap;
#[derive(Clone)]
pub struct DomainDatabase {
    pub config_list: HashMap<String, Entry>,
    pub entry_list: HashMap<String, Vec<Entry>>,
}
#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub entry_type: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}
