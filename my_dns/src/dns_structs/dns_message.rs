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
    pub authorities_values: Option<HashMap<QueryType, Vec<DNSSingleResponse>>>,
    pub extra_values: Option<HashMap<QueryType, Vec<DNSSingleResponse>>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSQueryInfo {
    pub name: String,
    pub type_of_value: Vec<QueryType>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum QueryType {
    NS,
    A,
    CNAME,
    MX,
    PTR,
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
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct DNSSingleResponse {
    pub name: String,
    pub type_of_value: String,
    pub value: String,
    pub ttl: u32,
}
