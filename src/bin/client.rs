use core::num;
use my_dns::dns_make::dns_send;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::random;
fn main() {
    let dns_query_info = DNSQueryInfo {
        name: "google.com".to_string(),
        type_of_value: vec![QueryType::A],
    };
    let dns_message_data = DNSMessageData {
        query_info: dns_query_info,
        response_values: None,
        authorities_values: None,
        extra_values: None,
    };
    let dns_message_header = DNSMessageHeaders {
        message_id: random(),
        flags: 6,
        response_code: None,
        number_of_values: None,
        number_of_authorities: None,
        number_of_extra_values: None,
    };
    let dns_message = DNSMessage {
        header: dns_message_header,
        data: dns_message_data,
    };

    let test = dns_send::send(dns_message, "127.0.0.1".to_string(), 5454);
    match test {
        Err(err) => panic!("{err}"),
        _ => (),
    }
}
