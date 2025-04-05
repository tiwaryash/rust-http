use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write,Read};


fn respond_body(mut stream: TcpStream, body:&str){
    let content_length=body.len();
    let response=format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        content_length,
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();


}
fn handle_client_req(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();

    if let Ok(_) = reader.read_line(&mut request_line) {
        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let method = parts[0];
            let path = parts[1];

            println!("Received request: {} {}", method, path);

            if method == "GET" && path.starts_with("/echo/") {
                let echo_str = &path[6..];
                respond_body(stream, echo_str);
            }

            else if method == "GET" && path == "/user-agent" {
                let mut user_agent = String::new();

                for line in reader.by_ref().lines() {
                    if let Ok(line) = line {
                        if line == "" {
                            break;
                        }
                        if line.to_lowercase().starts_with("user-agent:") {
                            user_agent = line["User-Agent:".len()..].trim().to_string();
                        }
                    }
                }

                respond_body(stream, &user_agent);
            }

            else if method == "GET" && (path == "/" || path == "/index.html") {
                respond_body(stream, "Welcome to the Rust server!");
            }

            else {
                let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 19\r\n\r\n404 - Page not found";
                stream.write_all(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
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
