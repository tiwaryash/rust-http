use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};

fn handle_client_req(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    
    if let Ok(_) = reader.read_line(&mut request_line) {
        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];
            
            println!("Received request: {} {}", method, path);
            
            let response = if path == "/" || path == "/index.html" {
                "HTTP/1.1 200 OK\r\n\r\nWelcome to the Rust server!"
            } else {
                "HTTP/1.1 404 Not Found\r\n\r\n404 - Page not found"
            };
            
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn main() {
    println!("Server running on 127.0.0.1:4221");
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client_req(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
