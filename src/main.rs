use regex::Regex;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{env, fs};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    loop {
        let (stream, _) = listener.accept()?;

        tokio::spawn(async move {
            handle_client_async(stream).await;
        });
    }
}

async fn handle_client_async(mut stream: TcpStream) {
    let data = read_data(&mut stream);

    let (request_line, header_and_body) = data.split_once("\r\n").unwrap();
    let request_path = parse_header(request_line).path;

    let response: &str;
    let formatted;

    match request_path.as_str() {
        "/" => {
            response = "HTTP/1.1 200 OK\r\n\r\n";
        }
        path if {
            let pattern = Regex::new(r"/echo/.+").unwrap();
            pattern.is_match(path.trim())
        } =>
        {
            let path = request_path.split_once("/echo/").unwrap().1;
            formatted = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                path.len(),
                path
            );
            response = formatted.trim();
        }
        "/user-agent" => {
            formatted = user_agent(header_and_body);
            response = formatted.trim();
        }
        file_path
            if {
                let pattern = Regex::new(r"/files/.*").unwrap();
                pattern.is_match(file_path.trim())
            } =>
        {
            formatted = handle_file_path(request_path);
            response = formatted.trim();
        }
        _ => {
            response = "HTTP/1.1 404 Not Found\r\n\r\n";
        }
    }
    println!("Request was {} and response was {}", data, response);
    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_file_path(request_path: String) -> String {
    let args: Vec<String> = env::args().collect();
    let directory = args
        .iter()
        .find(|&arg| arg.starts_with("--directory="))
        .unwrap()
        .split("=")
        .collect::<Vec<&str>>()[1];
    let file = request_path.split_once("/files/").unwrap().1;

    match fs::read(format!("{directory}{file}")) {
        Ok(content) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            String::from_utf8(content).unwrap().trim(),
        ),
        Err(_) => "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string(),
    }
}

fn user_agent(header_and_body: &str) -> String {
    let pairs = header_and_body.trim().split("\r\n");
    let mut dictionary = HashMap::new();
    for pair in pairs {
        match pair.split_once(": ") {
            Some((key, value)) => {
                dictionary.insert(key, value);
            }
            None => {}
        };
    }
    let key = "User-Agent";
    let value = dictionary.get(key).unwrap();
    let body = format!("{value}");
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn read_data(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    String::from_utf8_lossy(&buffer[..]).trim().to_string()
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
