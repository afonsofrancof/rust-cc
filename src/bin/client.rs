use dnsmake::dns_message_struct::{DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType};
use rand::random;
use core::num;
use dnsmake::dns_make::dnssend;
fn main(){
    let dns_query_info = DNSQueryInfo{name:"google.com".to_string(),type_of_value:vec![QueryType::A]};
    let dns_message_data = DNSMessageData {query_info: dns_query_info,response_values:None,authorities_values:None,extra_values:None};
    let dns_message_header = DNSMessageHeaders{message_id:random(),flags: 6,response_code:None,number_of_values:None,number_of_authorities:None,number_of_extra_values:None};
    let dns_message = DNSMessage {header:dns_message_header,data:dns_message_data};

    let test = dnssend(dns_message,"127.0.0.1".to_string(),5454);
    match test{
        Err(err) => panic!("{err}"),
        _ => ()
    }
}




