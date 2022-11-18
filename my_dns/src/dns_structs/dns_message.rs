use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
    pub response_values: Option<HashMap<QueryType, Vec<DNSSingleResponse>>>,
    pub authorities_values: Option<Vec<DNSSingleResponse>>,
    pub extra_values: Option<Vec<DNSSingleResponse>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSQueryInfo {
    pub name: String,
    pub type_of_value: Vec<QueryType>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSSingleResponse {
    pub name: String,
    pub type_of_value: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum QueryType {
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
}
impl DNSQueryInfo {
    pub fn new() -> Self {
        DNSQueryInfo {
            name: "".to_string(),
            type_of_value: vec![],
        }
    }
}

impl QueryType {
    pub fn to_string(&self) -> &'static str {
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
