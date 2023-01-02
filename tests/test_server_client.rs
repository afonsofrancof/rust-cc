extern crate rustcc;

use my_dns::{
    dns_components::sr::resolver,
    dns_parse::server_config_parse,
    dns_structs::{
        dns_domain_name::Domain,
        dns_message::{
            DNSEntry, DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
        },
    },
};
use rustcc::{client, server};
use std::{net::SocketAddr, thread, time::Duration, str::FromStr};

#[test]
fn test_client_server() {
    let server = thread::spawn(move || {
        let config = server_config_parse::get("etc/test-example-com.conf".to_string()).unwrap();
        server::start_server(config, 5454, true, true)
    });
    thread::sleep(Duration::new(1, 0));
    let client = thread::spawn(move || {
        let server_list :Vec<SocketAddr> = vec![SocketAddr::from_str(&"0.0.0.0:5454".to_string()).unwrap()];
        let mut query =
            client::query_builder(Domain::new("www.example.com".to_string()), QueryType::A, 4);
        resolver(
            &mut query,
            server_list,
            true,
        )
    });

    match client.join() {
        Ok(msg) => {
            match msg {
                Ok(response) => {
                    let dns_query_info = DNSQueryInfo {
                        name: Domain::new("www.example.com".to_string()),
                        type_of_value: QueryType::A,
                    };
                    //Fill in response_values
                    let response_values = vec![DNSEntry {
                        domain_name: Domain::new("www.example.com.".to_string()),
                        type_of_value: "A".to_string(),
                        value: "10.3.3.1".to_string(),
                        ttl: 86400,
                        priority: Some(200),
                    }];
                    //Fill in authorities_values
                    let authorities_values = vec![DNSEntry {
                        domain_name: Domain::new("example.com.".to_string()),
                        type_of_value: "NS".to_string(),
                        value: "ns1.example.com.".to_string(),
                        ttl: 86400,
                        priority: None,
                    }];
                    //Fill in extra_values
                    let extra_values = vec![DNSEntry {
                        domain_name: Domain::new("ns1.example.com.".to_string()),
                        type_of_value: "A".to_string(),
                        value: "10.2.2.2".to_string(),
                        ttl: 86400,
                        priority: None,
                    }];
                    //Create dns_message_data
                    let dns_message_data = DNSMessageData {
                        query_info: dns_query_info,
                        response_values: Some(response_values),
                        authorities_values: Some(authorities_values),
                        extra_values: Some(extra_values),
                    };
                    //Create dns_message_header
                    let dns_message_header = DNSMessageHeaders {
                        message_id: response.header.message_id,
                        flags: 1,
                        response_code: Some(0),
                        number_of_values: Some(1),
                        number_of_authorities: Some(1),
                        number_of_extra_values: Some(1),
                    };

                    let dns_message = DNSMessage {
                        header: dns_message_header,
                        data: dns_message_data,
                    };
                    assert_eq!(response, dns_message);
                }
                Err(err) => panic!("{err}"),
            }
        },
        Err(_err) => {
            panic!("Client join failed on test")
        }
    };
    server.join().unwrap();
}
