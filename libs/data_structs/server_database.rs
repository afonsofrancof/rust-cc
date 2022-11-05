use std::collections::HashMap;

struct DomainConfig{
    config_list: HashMap<String,Vec<Entry>>,
    entry_list: HashMap<String,Vec<Entry>>
}

struct Entry{
    name: String,
    entry_type: String,
    value: String,
    ttl: u32,
}
