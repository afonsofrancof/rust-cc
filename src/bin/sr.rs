use dnsmake::dns_message_struct::{DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType};
use dnsmake::dns_make::dns_recv;
fn main(){
    match dns_recv(5454){
        Ok(value) => (),
        Err(err) => println!("{}",err.to_string())
    };
}
