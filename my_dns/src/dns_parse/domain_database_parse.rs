use crate::dns_structs::domain_database_struct::{DomainDatabase, Entry};
use regex::{Match, Regex};
use std::collections::HashMap;
use std::default;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

pub fn get(file_path: String) -> Result<DomainDatabase, &'static str> {
    // Abrir o ficheiro de database para leitura
    let mut file = match File::open(Path::new("etc").join(file_path).as_path()) {
        Err(_err) => return Err("GEY"),
        Ok(file) => file,
    };

    // String em memoria com o ficheiro para dar parse
    let mut read = String::new();

    match file.read_to_string(&mut read) {
        Err(_err) => return Err("Couldn't Read to String"),
        Ok(_) => println!("File great success"),
    };

    let regex_variables =
        Regex::new(r"(?m)^([@A-Za-z.0-9-]+) DEFAULT ([A-Za-z.0-9\\.-]+)").unwrap();

    let regex_soa = Regex::new(
        r"(?m)^([@A-Za-z.0-9-]+) (SOA[A-Z]+) ([A-Za-z.0-9\\.-]+) ([A-Z0-9]+)( [A-Z0-9]+)?",
    )
    .unwrap();

    let regex_entry = Regex::new(
        r"(?m)^([@A-Za-z.0-9-]+) (NS|A|CNAME|MX|PTR) ([A-Za-z.0-9\\.-]+) ([A-Z0-9]+)( [A-Z0-9]+)?",
    )
    .unwrap();

    // Deste modo, os comentario ficam todos ignorados visto que as expressoes capturam apenas as expressoes no inicio da linha

    // HashMaps onde vamos guardar os valores para dar return
    // Mapa com o nome da variavel como key
    let mut variables: HashMap<String, String> = HashMap::new();

    // Mapa que vai conter todas as SOAs entries tendo o tipo de SOA como key (aka SOAADMIN,SOAEXPIRE, etc)
    let mut soa_entries: HashMap<String, Entry> = HashMap::new();

    // Mapa que vai conter todas as entries com o tipo (MX,A,NS,CNAME,PTR) como key e uma lista de entries como valor
    let mut entries: HashMap<String, Vec<Entry>> = HashMap::new();

    // Capturar todas as variaveis primeiro pois vao ser usadas nos outros mapas para substituir os defaults
    for cap in regex_variables.captures_iter(&read) {
        variables.insert(cap[1].to_string(), cap[2].to_string());
    }

    // Capturar todas as SOAs entries
    for cap in regex_soa.captures_iter(&read) {
        // Podemos fazer error check nesta seccao do codigo
        let mut name: String = cap[1].to_string();
        let mut entry_type: String = cap[2].to_string();
        let mut value: String = cap[3].to_string();
        let mut temp_ttl: String = cap[4].to_string();
        let mut priority: Option<u16> = None;

        for (variable, value) in variables.iter() {
            name = name.replace(variable, value);
            temp_ttl = temp_ttl.replace(variable, value).parse().unwrap();
        }

        let ttl: u32 = temp_ttl.parse().unwrap();

        soa_entries.insert(
            cap[2].to_string(),
            Entry {
                name,
                entry_type,
                value,
                ttl,
                priority,
            },
        );
    }

    // Capturar todas as entries
    for cap in regex_entry.captures_iter(&read) {
        // Podemos fazer error check nesta seccao do codigo
        let mut name: String = cap[1].to_string();
        let mut entry_type: String = cap[2].to_string();
        let mut value: String = cap[3].to_string();
        let mut temp_ttl: String = cap[4].to_string();
        let mut priority: Option<u16> = None;

        for (variable, value) in variables.iter() {
            name = name.replace(variable, value);
            temp_ttl = temp_ttl.replace(variable, value).parse().unwrap();
        }

        let ttl: u32 = temp_ttl.parse().unwrap();

        let temp_entry: Entry = Entry {
            name,
            entry_type: entry_type.to_owned(),
            value,
            ttl,
            priority,
        };

        match entries.get_mut(&entry_type) {
            Some(list) => {
                list.push(temp_entry);
            }
            None => {
                entries.insert(temp_entry.entry_type.to_owned(), vec![temp_entry]);
            }
        };
    }

    // // Prints de Debug temporario
    // println!("SOA Entries Registadas");

    // for (key, val) in soa_entries.iter() {
    //     println!("{} {} {}", val.name, key, val.ttl);
    // }

    // println!("Entries Registadas");

    // for (_key, val) in entries.iter() {
    //     for entry in val {
    //         println!(
    //             "{} {} {} {}",
    //             entry.name, entry.entry_type, entry.value, entry.ttl
    //         );
    //     }
    //     println!("-------");
    // }

    Ok(DomainDatabase {
        config_list: soa_entries,
        entry_list: entries,
    })
}

// 1. LER O FILE COM UM BUFFER E DETERMINAR SE A INFO E USEFUL A CADA ITERACAO DO LOOP
// fn read_file_buffer(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
//     const BUFFER_LEN: usize = 512;
//     let mut buffer = [0u8; BUFFER_LEN];
//     let mut file = File::open(filepath)?;
//     loop {
//         let read_count = file.read(&mut buffer)?;
//         do_something(&buffer[..read_count]);
//         if read_count != BUFFER_LEN {
//             break;
//         }
//     }
//     Ok(())
// }
// Nao ha nenhum benificio em ler em buffer, como a informacao relevante esta em cada linha

// 3. LER O FILE LINHA A LINHA E DESCARTAR AS LINHAS QUE NAO SAO PRETENDIDAS
// Optamos por usar esta estrategia
// fn read_file_line_by_line(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let file = File::open(filepath)?;
//     let reader = BufReader::new(file);
//     for line in reader.lines() {
//         println!("{}", line?);
//     }
//     Ok(())
// }

// BAD IDEA!!!!!
/*
    para por isto a funcionar teriamos que fazer dentro das capturas outra regex para verificar o tipo de entrada e afins
    not good at all

*/
// for line in reader.lines() {
//     match line {
//         Result::Ok(line_ok) => {
//             // Guardar o TTL default e o @ example.com (os defaults)

//             let regex_default_capture = regex_default_expression.captures(&line_ok);
//             match regex_default_capture {
//                 Some(default_capture) => {
//                     let filtered_capture: Vec<Match> = default_capture
//                         .iter()
//                         .filter(|x| x.is_some())
//                         .map(|x| x.unwrap())
//                         .collect();

//                     match filtered_capture.len() {
//                         // Captura de Variaveis
//                         4 => {}
//                     }
//                 }

//                 None => {
//                     continue;
//                 }
//             }

//             // match regex_capture[1] == '#'

//             // Vamos capturar de estouro todos os valores necessarios,
//             // descartando os invalidos e alertando dos inputs mal configurados
//         }
//         Result::Err(err) => {
//             panic!("Error Reading Line : {err}")
//         }
//     }
