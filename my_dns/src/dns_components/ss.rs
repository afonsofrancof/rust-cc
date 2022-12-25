use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    string::String,
    sync::{Arc, Mutex}, 
    thread,
    time,
};

use crate::{
    dns_parse::domain_database_parse,
    dns_structs::domain_database_struct::{DomainDatabase, SOA},  
};

pub fn db_sync(
    domain_name: String,
    sp_addr: SocketAddr,
    db: Arc<Mutex<HashMap<String, DomainDatabase>>>,
) {
    // initial sync
    let mut domain_db: DomainDatabase = zone_transfer(&domain_name, sp_addr, 0);
    let mut soas: SOA = domain_db.get_soa_records();
    let mut serial: u32;
    let mut refresh: u64;
    let mut retry: u64;
    let mut expire: u64;

    // antes disto tem de existir uma initial sync para poder existir isto
    // SOASERIAL?? nao esquecer de checkar onde isso vai
    loop {
        serial = soas.get_serial_value();
        refresh = soas.get_refresh_value(); 
        thread::sleep(time::Duration::from_secs(refresh));
        
        domain_db = zone_transfer(&domain_name, sp_addr, serial);
        
         
        soas = domain_db.get_soa_records();
        let mut locked_db = db.lock().unwrap();
        locked_db.insert(domain_name.to_owned(), domain_db.clone());
        drop(locked_db)
    }

}

fn zone_transfer(domain_name: &String, sp_addr: SocketAddr, serial: u32) -> DomainDatabase {
    let mut stream = match TcpStream::connect(sp_addr) {
        Ok(stream) => stream,
        Err(err) => {
            panic!("Could't connect to addr {}", sp_addr);
        }
    };

    stream.write(domain_name.as_bytes()).unwrap();
    
    let mut buf = [0u8; 1000];
    // vai receber o numero de serie MUDAR ISTO NO SP
    let mut serial_buf = [0u8; 4];
    stream.read_exact(&mut serial_buf);
    let received_serial = u32::from_ne_bytes(serial_buf);
    if (received_serial == serial) {
        
    }

    // recebe as entries que existem 
    stream.read(&mut buf);

    let entries: u16 = (buf[0].to_owned() as u16 * 256) + buf[1].to_owned() as u16;

    // confirmacao resolver isto ...
    stream.write(&mut buf);
    let mut unparsed_db: Vec<String> = Vec::with_capacity(entries.clone().into());
    // codificao primeiros 2 bytes sao o numero de ordem da entry o resto e do tipo Entry
    println!("Number of entries: {}", entries);
    let mut ebuf = [0u8; 1000];
    for i in 0..entries {
        let num_bytes = match stream.read(&mut ebuf) {
            Ok(bytes) => bytes,
            Err(err) => panic!("{err}"),
        };

        let seq_number: u16 = (ebuf[0] as u16 * 256) + ebuf[1] as u16;

        let line_bin = ebuf[2..num_bytes - 2].to_vec();

        let mut line = String::from_utf8(line_bin).unwrap().to_owned();
        line.push('\n');
        println!("{} - {}", seq_number, line);
        unparsed_db.insert(seq_number.to_owned().into(), line);
    }
    let mut db_txt: String = String::new();

    for line in unparsed_db {
        db_txt.push_str(line.as_str());
    }
    let domain_db: DomainDatabase = match domain_database_parse::parse_from_str(db_txt) {
        Ok(db) => db,
        Err(err) => panic!("Coudn't parse database"),
    };
    domain_db
}
