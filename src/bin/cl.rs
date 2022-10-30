use data_structs::dns_packet::{DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo};

fn main(){
    test_query();
}
pub fn test_query(){
    let query_info = DNSQueryInfo{name:"example.com".to_string(),type_of_value: 8};
    let query_data = DNSMessageData{query_info: query_info,authorities_values:None,response_values:None,extra_values:None};
    let query_headers = DNSMessageHeaders{message_id: 34224,flags: 6, number_of_values:None,response_code:None,number_of_authorities:None,number_of_extra_values:None};
    let dns_request = DNSMessage{header: query_headers,data: query_data};
    let query_type: u16 = dns_request.get_message_id();
    println!("{}\n",query_type);
}
