use std::collections::HashMap;
use crate::dns_parse::domain_database_parse;
use super::dns_message::QueryType;

#[derive(Clone)]
pub struct DomainDatabase {
    pub config_list: HashMap<String, Entry>,
    pub ns_records: Option<HashMap<String,Vec<Entry>>>,
    pub a_records: Option<Vec<Entry>>,
    pub cname_records: Option<Vec<Entry>>,
    pub mx_records: Option<Vec<Entry>>,
    pub ptr_records: Option<Vec<Entry>>,
}
#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub entry_type: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}

impl DomainDatabase {
    pub fn get_ns_records(&self) -> Option<HashMap<String,Vec<Entry>>> {self.ns_records.to_owned()}
    pub fn get_a_records(&self) -> Option<Vec<Entry>> {self.a_records.to_owned()}
    pub fn get_cname_records(&self) -> Option<Vec<Entry>> {self.cname_records.to_owned()}
    pub fn get_mx_records(&self) -> Option<Vec<Entry>> {self.mx_records.to_owned()}
    pub fn get_ptr_records(&self) -> Option<Vec<Entry>> {self.ptr_records.to_owned()}

    pub fn add_ns_record(&mut self, domain_name: String, entry: Entry){
        match &mut self.ns_records {
            Some(domain) => match domain.get_mut(&domain_name) {
                Some(records) => {records.push(entry);}
                None => {domain.insert(domain_name,vec![entry]);}
            },
            None => {
                let mut ns_records = HashMap::new();
                ns_records.insert(domain_name,vec![entry]);
                self.ns_records = Some(ns_records);     
            }
        }
    }
    pub fn add_a_record(&mut self,entry:Entry) {
        match &mut self.a_records {
            Some(records) => {records.push(entry);},
            None => {self.a_records = Some(vec![entry]);}
        };
    }
    pub fn add_cname_record(&mut self,entry:Entry) { 
        match &mut self.cname_records {
            Some(records) => {records.push(entry);},
            None => {self.cname_records = Some(vec![entry]);}
        };
    }
    pub fn add_mx_record(&mut self,entry:Entry) {
        match &mut self.mx_records {
            Some(records) => {records.push(entry);},
            None => {self.mx_records = Some(vec![entry]);}
        }
    }
    pub fn add_ptr_record(&mut self,entry:Entry) {
        match &mut self.ptr_records {
            Some(records) => {records.push(entry);},
            None => {self.ptr_records = Some(vec![entry]);}
        }
    }
}

