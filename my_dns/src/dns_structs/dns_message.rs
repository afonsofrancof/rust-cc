use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::dns_domain_name::Domain;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DNSMessage {
    pub header: DNSMessageHeaders,
    pub data: DNSMessageData,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct DNSMessageHeaders {
    pub message_id: u16,
    pub flags: u8,
    pub response_code: Option<u8>,
    pub number_of_values: Option<u8>,
    pub number_of_authorities: Option<u8>,
    pub number_of_extra_values: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DNSMessageData {
    pub query_info: DNSQueryInfo,
    pub response_values: Option<HashMap<QueryType, Vec<DNSEntry>>>,
    pub authorities_values: Option<Vec<DNSEntry>>,
    pub extra_values: Option<Vec<DNSEntry>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSQueryInfo {
    pub name: Domain,
    pub type_of_value: Vec<QueryType>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSEntry {
    pub domain_name: Domain,
    pub type_of_value: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum  QueryType {
    NS,
    A,
    CNAME,
    MX,
    PTR,
}

impl DNSMessage {
    pub fn new() -> Self {
        let header = DNSMessageHeaders::new();
        let data = DNSMessageData::new();
        DNSMessage { header, data }
    }

    pub fn get_string(&self) -> String {
        let mut res = String::new();

        res.push_str(self.header.get_string().as_str());
        res.push_str(self.data.get_string().as_str());
        res
    }
}

impl DNSMessageHeaders {
    pub fn new() -> Self {
        DNSMessageHeaders {
            message_id: rand::random(),
            flags: 0,
            response_code: None,
            number_of_values: None,
            number_of_authorities: None,
            number_of_extra_values: None,
        }
    }
   
    // O sistema de flags funciona em binario em que se soma o valor de todas as flags
    // A   => 0 0 1 = 1
    // R   => 0 1 0 = 2
    // Q   => 1 0 0 = 4
    // R+A => 0 1 1 = 3
    // Q+R => 1 1 0 = 6
    pub fn decode_flags(&self) -> Result<&str,&'static str> {
        match self.flags {
            1 => Ok("A"),
            2 => Ok("R"),
            4 => Ok("Q"),
            3 => Ok("R+A"),
            6 => Ok("Q+R"),
            _ => Err("Flag value does not match any combination of flags")
        }
    }

    pub fn get_string(&self) -> String {
        let mut rc: u8 = 0;
        let mut nov: u8 = 0;
        let mut noa: u8 = 0;
        let mut noev: u8 = 0;

        if let Some(i) = self.response_code {
            rc = i
        };
        if let Some(i) = self.number_of_values {
            nov = i
        };
        if let Some(i) = self.number_of_authorities {
            noa = i
        };
        if let Some(i) = self.number_of_extra_values {
            noev = i
        };

        let flags = match self.decode_flags(){
            Ok(flag) => flag,
            Err(err) => panic!("{err}")
        };

        format!("{},{},{},{},{},{};",
                self.message_id,
                flags,
                rc,
                nov,
                noa,
                noev
                )
    }
}

impl DNSMessageData {
    pub fn new() -> Self {
        let dns_query_info = DNSQueryInfo::new();
        DNSMessageData {
            query_info: dns_query_info,
            response_values: None,
            authorities_values: None,
            extra_values: None,
        }
    }

    pub fn get_string(&self) -> String {
        let mut res = String::new();
        
        res.push_str(self.query_info.get_string().as_str());
        res.push_str("\n");

        let rv = match &self.response_values {
            Some(hm) => {
                let vec_str: Vec<String> = hm.values().flatten().map(|x| x.get_string()).collect();
                let mut sb: String = String::new();
                for mut entry in vec_str {
                    entry.push_str(",\n");
                    sb.push_str(entry.as_str());
                }
                sb.insert(sb.len() -2, ';'); 
                sb
            },
            None => String::new()
        };
        res.push_str(rv.as_str());

        let av = match &self.authorities_values {
            Some(vec) => {
                let vec_str: Vec<String> = vec.iter().map(|x| x.get_string()).collect();
                let mut sb: String = String::new();
                for mut entry in vec_str {
                    entry.push_str(",\n");
                    sb.push_str(entry.as_str());
                }
                sb.insert(sb.len() -2, ';'); 
                sb
            },
            None => String::new()
        };
        res.push_str(av.as_str());

        let ev = match &self.extra_values {
            Some(vec) => {
                let vec_str: Vec<String> = vec.iter().map(|x| x.get_string()).collect();
                let mut sb: String = String::new();
                for mut entry in vec_str {
                    entry.push_str(",\n");
                    sb.push_str(entry.as_str());
                }
                sb.insert(sb.len() -2, ';'); 
                sb
            },
            None => String::new()
        };
        res.push_str(ev.as_str());
        res 
    }
}

impl DNSQueryInfo {
    pub fn new() -> Self {
        DNSQueryInfo {
            name: Domain::new_empty(),
            type_of_value: vec![],
        }
    }

    pub fn get_string(&self) -> String {
        let tov: String = self.type_of_value.iter().map(|x| x.get_string()).collect();
<<<<<<< HEAD
        format!("{}, {}",self.name.to_string(), tov)
=======
        format!("{},{};",self.name, tov)
>>>>>>> c1ccd9b972e0e8fe9060dda7eaf5fdf48e19198f
    }
}

impl DNSEntry {
    pub fn new() -> Self {
        DNSEntry {
            domain_name: Domain::new_empty(), 
            type_of_value: String::new(), 
            value: String::new(), 
            ttl: 0,
            priority: None
        }
    }
    
    
    pub fn get_value(&self) -> String {
        self.value.to_owned()
    }

    // falta a priority
    pub fn get_string(&self) -> String {
        if let Some(priority) = self.priority {
            format!("{} {} {} {} {}",self.domain_name.to_string(),self.type_of_value,self.value,self.ttl, priority)
        } else{
            format!("{} {} {} {}",self.domain_name.to_string(),self.type_of_value,self.value,self.ttl)
        }
    }
}

impl QueryType {
    pub fn get_string(&self) -> &'static str {
        match self {
            QueryType::NS => "NS",
            QueryType::A => "A",
            QueryType::CNAME => "CNAME",
            QueryType::MX => "MX",
            QueryType::PTR => "PTR",
        }
    }

    pub fn from_string(query_type: String) -> Result<QueryType, String> {
        match query_type.as_str() {
            "NS" => Ok(QueryType::NS),
            "A" => Ok(QueryType::A),
            "CNAME" => Ok(QueryType::CNAME),
            "MX" => Ok(QueryType::MX),
            "PTR" => Ok(QueryType::PTR),
            _ => Err(format!("Cannot find QueryType of {}", query_type)),
        }
    }
}
