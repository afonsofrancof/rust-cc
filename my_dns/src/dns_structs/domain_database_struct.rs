use super::dns_message::{DNSEntry, QueryType};
use crate::dns_parse::domain_database_parse;
use std::{collections::HashMap, ops::Add};

#[derive(Clone)]
pub struct DomainDatabase {
    pub config_list: HashMap<String, DNSEntry>,
    pub ns_records: Option<HashMap<String, Vec<DNSEntry>>>,
    pub a_records: Option<Vec<DNSEntry>>,
    pub cname_records: Option<Vec<DNSEntry>>,
    pub mx_records: Option<Vec<DNSEntry>>,
    pub ptr_records: Option<Vec<DNSEntry>>,
}

impl DomainDatabase {
    pub fn new() -> Self {
        DomainDatabase {
            config_list: HashMap::new(),
            ns_records: None,
            a_records: None,
            cname_records: None,
            mx_records: None,
            ptr_records: None,
        }
    }

    pub fn get_config_list(&self) -> HashMap<String, DNSEntry> {
        self.config_list.to_owned()
    }

    pub fn get_ns_of(&self, domain: String) -> Option<(String, Vec<DNSEntry>)> {
        match &self.ns_records {
            Some(hm) => {
                let biggest_match = hm
                    .iter()
                    .clone()
                    .filter(|(domain_name, _domain_ns_vec)| {
                        println!("subdomain: {domain}, domain: {domain_name}");
                        let dn = match domain_name.as_str() {
                            "." => ".".to_string(),
                            _ => ".".to_string().add(domain_name),
                        };
                        ".".to_string().add(&domain).ends_with(&dn)
                    })
                    .max_by(|(dn1, _dnsvec1), (dn2, _dnsvec2)| dn1.len().cmp(&dn2.len()))
                    .map(|(dn, dnsvec)| (dn.to_owned(), dnsvec.to_owned()));
                biggest_match
            }
            None => None,
        }
    }
    pub fn get_ns_records(&self) -> Option<HashMap<String, Vec<DNSEntry>>> {
        self.ns_records.to_owned()
    }
    pub fn get_a_records(&self) -> Option<Vec<DNSEntry>> {
        self.a_records.to_owned()
    }
    pub fn get_cname_records(&self) -> Option<Vec<DNSEntry>> {
        self.cname_records.to_owned()
    }
    pub fn get_mx_records(&self) -> Option<Vec<DNSEntry>> {
        self.mx_records.to_owned()
    }
    pub fn get_ptr_records(&self) -> Option<Vec<DNSEntry>> {
        self.ptr_records.to_owned()
    }

    pub fn add_ns_record(&mut self, domain_name: String, entry: DNSEntry) {
        match &mut self.ns_records {
            Some(hm) => match hm.get_mut(&domain_name) {
                Some(records) => {
                    records.push(entry);
                }
                None => {
                    hm.insert(domain_name, vec![entry]);
                }
            },
            None => {
                let mut ns_records = HashMap::new();
                ns_records.insert(domain_name, vec![entry]);
                self.ns_records = Some(ns_records);
            }
        }
    }
    pub fn add_a_record(&mut self, entry: DNSEntry) {
        match &mut self.a_records {
            Some(records) => {
                records.push(entry);
            }
            None => {
                self.a_records = Some(vec![entry]);
            }
        };
    }
    pub fn add_cname_record(&mut self, entry: DNSEntry) {
        match &mut self.cname_records {
            Some(records) => {
                records.push(entry);
            }
            None => {
                self.cname_records = Some(vec![entry]);
            }
        };
    }
    pub fn add_mx_record(&mut self, entry: DNSEntry) {
        match &mut self.mx_records {
            Some(records) => {
                records.push(entry);
            }
            None => {
                self.mx_records = Some(vec![entry]);
            }
        }
    }
    pub fn add_ptr_record(&mut self, entry: DNSEntry) {
        match &mut self.ptr_records {
            Some(records) => {
                records.push(entry);
            }
            None => {
                self.ptr_records = Some(vec![entry]);
            }
        }
    }
}
