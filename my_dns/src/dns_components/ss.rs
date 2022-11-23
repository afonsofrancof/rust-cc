use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    string::String,
    sync::{Arc, Mutex},
};

use crate::{
    dns_parse::domain_database_parse, dns_structs::domain_database_struct::DomainDatabase,
};

pub fn db_sync(
    domain_name: String,
    sp_addr: SocketAddr,
    db: Arc<Mutex<HashMap<String, DomainDatabase>>>,
) {
    let mut stream = match TcpStream::connect(sp_addr) {
        Ok(stream) => stream,
        Err(err) => {
            panic!("Could't connect to addr {}", sp_addr);
        }
    };

    stream.write(domain_name.as_bytes()).unwrap();

    let mut buf = [0u8; 1000];

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

    let mut locked_db = db.lock().unwrap();
    locked_db.insert(domain_name, domain_db);
}
