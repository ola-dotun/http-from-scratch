use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::{TcpListener,  TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let data = read_data(&mut _stream);
                
                let (request_line, _) = data.split_once("\r\n").unwrap();
                let request_path = parse_header(request_line).path;
                
                let response: &str;
                
                match request_path.as_str() {
                    "/" => {
                        response = "HTTP/1.1 200 OK\r\n\r\n";
                    },
                    _ => {
                        response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                    }
                }
                println!("Request was {} and response was {}", data, response);
                _stream.write_all(response.as_bytes()).unwrap();
                
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn read_data(stream: &mut TcpStream) -> String {    
    let mut data = String::new();
    
    stream.read_to_string(&mut data).unwrap_or_else(|e| {
        println!("error: {}", e);
        0
    });
    
    data.trim().to_string()
}

fn parse_header(header: &str) -> Header {
    let parts = header.split_whitespace().collect::<Vec<&str>>();
    Header {
        // method: String::from(parts[0]),
        path: String::from(parts[1]),
        // http_version: String::from(parts[2]),
    }
}

struct Header {
    // method: String,
    path: String,
    // http_version: String,
}
