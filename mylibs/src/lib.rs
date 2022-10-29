    pub struct DNSMessage {
         pub header: DNSMessageHeaders,
         pub data: DNSMessageData,
    }
    
        pub struct DNSMessageHeaders {
         pub message_id: u16,
         pub flags: u8,
         pub response_code: Option<u8>,
         pub number_of_values: Option<u8>,
         pub number_of_authorities: Option<u8>,
         pub number_of_extra_values: Option<u8>,

    }
                
     pub struct DNSMessageData {
         pub query_info: DNSQueryInfo,
         pub response_values: Option<Vec<DNSSingleResponse>>,
         pub authorities_values: Option<Vec<DNSSingleResponse>>,
         pub extra_values: Option<Vec<DNSSingleResponse>>,
    }
        
     pub struct DNSQueryInfo {
         pub name: String,
         pub type_of_value: u16,
    }

     pub struct DNSSingleResponse {
        pub name: String,
        pub type_of_value: u16,
        pub value: String,
        pub ttl: u32,
    }
    
    impl DNSMessage {
        pub fn get_message_id(&self) -> u16{
            self.header.message_id
        }
    }
