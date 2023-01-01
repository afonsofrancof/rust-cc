use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    string::String,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use log::debug;

use crate::{
    dns_parse::domain_database_parse,
    dns_structs::{
        dns_domain_name::Domain,
        domain_database_struct::{DomainDatabase, SOA},
    },
};

#[derive(Debug)]
enum ZoneTransferError {
    SERIAL,
    PARSEERR,
    CONERR,
}

pub fn db_sync(
    domain_name: Domain,
    sp_addr: SocketAddr,
    db: Arc<Mutex<HashMap<Domain, DomainDatabase>>>,
) {
    // initial sync
    let mut domain_db: Option<DomainDatabase>;
    let mut soas: SOA;
    let mut serial: u32 = 0;
    let mut refresh: u64 = 0;
    let mut retry: u64 = 3600; // default value
    let mut expire: u64; // no idea para que isto serve
    let mut initial_flag: bool = true;
    // antes disto tem de existir uma initial sync para poder existir isto
    // SOASERIAL?? nao esquecer de checkar onde isso vai
    loop {
        match zone_transfer(&domain_name, sp_addr, serial) {
            Ok(domain_db) => {
                if !initial_flag {
                    soas = domain_db.get_soa_records();
                    serial = soas.get_serial_value();
                    refresh = soas.get_refresh_value();
                    retry = soas.get_retry_value();
                }
                let mut locked_db = db.lock().unwrap();
                locked_db.insert(domain_name.to_owned(), domain_db.clone());
                drop(locked_db);
                initial_flag = false;
                thread::sleep(Duration::from_secs(refresh));
            }
            Err(ZoneTransferError::SERIAL) => {
                thread::sleep(Duration::from_secs(refresh));
            }
            Err(ZoneTransferError::CONERR) => {
                thread::sleep(Duration::from_secs(retry));
            }
            Err(ZoneTransferError::PARSEERR) => {
                thread::sleep(Duration::from_secs(retry));
            }
        }
    }
}

fn zone_transfer(
    domain_name: &Domain,
    sp_addr: SocketAddr,
    serial: u32,
) -> Result<DomainDatabase, ZoneTransferError> {
    let now = Instant::now();

    let mut stream = match TcpStream::connect(sp_addr) {
        Ok(stream) => stream,
        Err(_err) => {
            debug!("EZ {} SS connection-fail",sp_addr);
            return Err(ZoneTransferError::CONERR);
            // panic!("Could't connect to addr {}", sp_addr);
        }
    };
    // enviar o nome do dominio pretendido
    stream.write(domain_name.to_string().as_bytes()).unwrap();
    let mut total_bytes_transfered = 0;
    let mut buf = [0u8; 1000];
    // receber o SERIAL - MUDAR ISTO NO SP
    let mut serial_buf = [0u8; 4];
    total_bytes_transfered += 4;
    stream.read_exact(&mut serial_buf);
    let received_serial = u32::from_ne_bytes(serial_buf);

    if received_serial == serial {
        // se o serial for igual envia um 0 para que o sp feche a ligacao
        stream.write(&[0u8]);
        debug!(
            "ZT {} SS same-SERIAL {} ms {} bytes",
            sp_addr,
            now.elapsed().as_millis(),
            total_bytes_transfered
        );
        return Err(ZoneTransferError::SERIAL);
    }
    // caso o serial seja diferente envia um 1 ao sp para que ele envie a base de dados
    stream.write(&[1u8]);

    // recebe as entries que existem
    total_bytes_transfered += stream.read(&mut buf).unwrap();

    let entries: u16 = (buf[0].to_owned() as u16 * 256) + buf[1].to_owned() as u16;

    // confirmacao resolver isto ...
    stream.write(&mut buf).unwrap();
    let mut unparsed_db: Vec<String> = Vec::with_capacity(entries.clone().into());

    // codificao primeiros 2 bytes sao o numero de ordem da entry o resto e do tipo Entry
    for _i in 0..entries {
        let num_bytes = match stream.read(&mut buf) {
            Ok(bytes) => bytes,
            Err(err) => panic!("{err}"),
        };
        total_bytes_transfered += num_bytes;
        let seq_number: u16 = (buf[0] as u16 * 256) + buf[1] as u16;

        let line_bin = buf[2..num_bytes - 2].to_vec();

        let mut line = String::from_utf8(line_bin).unwrap().to_owned();
        line.push('\n');
        unparsed_db.insert(seq_number.to_owned().into(), line);
    }
    let mut db_txt: String = String::new();

    for line in unparsed_db {
        db_txt.push_str(line.as_str());
    }

    match domain_database_parse::parse_from_str(db_txt) {
        Ok(db) => {
            debug!(
                "ZT {} SS {} ms {} bytes",
                sp_addr,
                now.elapsed().as_millis(),
                total_bytes_transfered
            );
            Ok(db)
        }
        Err(_err) =>{ 
            debug!("EZ {} SS database-parse-fail",sp_addr);
            Err(ZoneTransferError::PARSEERR)
        }
    }
}
