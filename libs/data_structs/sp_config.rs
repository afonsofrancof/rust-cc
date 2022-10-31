pub struct SPConfig{
    domain_name: String,
    entries: Vec<Entry>,
}

pub struct Entry {
    name: String,
    entry_type: EntryType,
    value: String,
}

pub enum EntryType{
    DB,
    SS,
    DD,
    LG,
    ST
}