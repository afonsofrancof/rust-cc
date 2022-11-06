use dnssend;
fn main(){
    let test = dnssend::dnssend("google.com".to_string(), "A".to_string(), "127.0.0.1".to_string());
    match test{
        Err(err) => panic!("{err}"),
        _ => ()
    }
}
