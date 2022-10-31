use regex::Regex;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

pub fn main(){
    test_configReader();
}
// Como argumentos maybe recebe o domain, o tipo de coiso (MX,A,Ns...)?
pub fn test_configReader() {
    let request_type = "MX";

    let file_path = Path::new("etc").join("example-com.db");

    let mut file = File::open(&file_path).unwrap();
    let reader = BufReader::new(file);

    let mut default_address = String::new();
    let mut default_ttl: u16;

    let regex_default_expression = Regex::new(r"(@|[A-Z]+) ([A-Z]+) ([a-zA-Z.0-9]+)").unwrap();

    for line in reader.lines() {
        match line {
            Result::Ok(line_ok) => {
                // Guardar o TTL default e o @ example.com (os defaults)

                let regex_default_capture = regex_default_expression.captures(&line_ok);
                match regex_default_capture {
                    Some(default_capture) => {
                        if default_capture.len() == 4 {
                            println!("Captura feita : {}", &default_capture[0]);
                        }
                    }

                    None => continue,
                }

                // match regex_capture[1] == '#'

                // Vamos capturar de estouro todos os valores necessarios,
                // descartando os invalidos e alertando dos inputs mal configurados
            }
            Result::Err(err) => {
                panic!("Error Reading Line : {err}")
            }
        }
    }
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
