use super::dns_message::{DNSEntry, QueryType};
use super::dns_domain_name::Domain;
use crate::dns_parse::domain_database_parse;
use std::{collections::HashMap, ops::Add};

#[derive(Clone)]
pub struct DomainDatabase {
    pub soa_entries: SOA,
    pub ns_records: HashMap<String, Vec<DNSEntry>>,
    pub a_records: Option<Vec<DNSEntry>>,
    pub cname_records: Option<Vec<DNSEntry>>,
    pub mx_records: Option<Vec<DNSEntry>>,
    pub ptr_records: Option<Vec<DNSEntry>>,
}

#[derive(Clone)]
pub struct SOA {
   pub primary_ns: DNSEntry,    // the primary name server for the domain
   pub contact_email: DNSEntry, // the email address of the domain administrator
   pub serial: DNSEntry,        // the serial number of the SOA record
   pub refresh: DNSEntry,       // the time, in seconds, between refreshes of the zone
   pub retry: DNSEntry,         // the time, in seconds, between retries if the refresh fails
   pub expire: DNSEntry,        // the time, in seconds, after which the zone is considered expired
}

impl DomainDatabase {
    pub fn new() -> Self {
        DomainDatabase {
            soa_entries: SOA::new(),
            ns_records: HashMap::new(),
            a_records: None,
            cname_records: None,
            mx_records: None,
            ptr_records: None,
        }
    }

    pub fn get_soa_records(&self) -> SOA {
        self.soa_entries.to_owned()
    }

    pub fn get_ns_of(&self, domain: String) -> Option<(String, Vec<DNSEntry>)> {
        let biggest_match = self
            .ns_records
            .iter()
            .clone()
            .filter(|(domain_name, _domain_ns_vec)| {
                let domain1 = Domain::new(domain_name.to_string());
                let domain2 = Domain::new(domain.to_string());
                domain2.is_subdomain_of(&domain1)
            })
            .max_by(|(dn1, _dnsvec1), (dn2, _dnsvec2)| dn1.len().cmp(&dn2.len()))
            .map(|(dn, dnsvec)| (dn.to_owned(), dnsvec.to_owned()));
        biggest_match
    }

    pub fn get_ns_records(&self) -> HashMap<String, Vec<DNSEntry>> {
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
        match self.ns_records.get_mut(&domain_name) {
            Some(records) => {
                records.push(entry);
            }
            None => {
                self.ns_records.insert(domain_name, vec![entry]);
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

impl SOA {
    pub fn new() -> SOA {
        SOA {
            primary_ns: DNSEntry::new(),
            contact_email: DNSEntry::new(),
            serial: DNSEntry::new(),
            refresh: DNSEntry::new(),
            retry: DNSEntry::new(),
            expire: DNSEntry::new(),
        }
    }
}
