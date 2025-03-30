#[allow(unused_imports)]
use std::net::{TcpListener,TcpStream};
use std::io::{Read,Write};

//Handling an incoming client connection
fn handle_client_req(mut stream: TcpStream){

    let mut buffer=[0;1024];

    if let Ok(_) = stream.read(&mut buffer){
        let response ="HTTP/1.1 200 OK\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client_req(stream)
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
