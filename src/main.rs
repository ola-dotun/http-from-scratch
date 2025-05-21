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

const HTTP_404: &str = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";

async fn handle_client_async(mut stream: TcpStream) {
    let data = read_data(&mut stream);

    let (request_line, header_and_body) = data.split_once("\r\n").unwrap();
    let request_header = parse_header(request_line);
    let request_path = request_header.path.as_str();

    let response: &str;
    let formatted;

    match request_path {
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
            let directory = directory_from_args();
            let file_name = request_path.split_once("/files/").unwrap().1;

            match directory {
                Some(directory) => {
                    let http_method = request_header.method.as_str();

                    match http_method {
                        "GET" => {
                            formatted = get_file_from_file_path(directory.as_str(), file_name);
                            response = formatted.as_str();
                        }
                        "POST" => match header_and_body.split_once("\r\n\r\n") {
                            Some((headers, body)) => {
                                let content_length = header_value(headers, "Content-Length");
                                match save_content_to_file_path(
                                    directory.as_str(),
                                    file_name,
                                    body,
                                    content_length.parse().unwrap(),
                                ) {
                                    Ok(_) => response = "HTTP/1.1 201 Created\r\n\r\n",
                                    Err(_) => response = HTTP_404,
                                }
                            }
                            None => response = HTTP_404,
                        },
                        _ => response = HTTP_404,
                    }
                }
                None => response = HTTP_404,
            }
        }
        _ => response = HTTP_404,
    }
    stream.write_all(response.as_bytes()).unwrap();
}

fn directory_from_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let mut iterator = args.iter();

    while let Some(arg) = iterator.next() {
        if arg.as_str().eq("--directory") {
            return Some(iterator.next().unwrap().to_string());
        }
    }

    None
}

fn get_file_from_file_path(directory: &str, file_name: &str) -> String {
    let file_path = format!("{directory}{file_name}");

    match fs::read(file_path) {
        Ok(content) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            String::from_utf8(content).unwrap().trim(),
        ),
        Err(_) => HTTP_404.to_string(),
    }
}

fn save_content_to_file_path(
    directory: &str,
    file_name: &str,
    content: &str,
    content_length: u64,
) -> Result<(), std::io::Error> {
    let file_path = format!("{directory}{file_name}");

    let mut file = fs::File::create(file_path)?;

    match file.write(content.trim().as_bytes()) {
        Ok(_) => {
            file.set_len(content_length)?;
            Ok(())
        }
        Err(_) => panic!("Couldn't save file"),
    }
}

fn user_agent(header_and_body: &str) -> String {
    let value = header_value(header_and_body, "User-Agent");
    let body = format!("{value}");
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn header_value<'a>(headers: &'a str, header: &str) -> &'a str {
    let pairs = headers.trim().split("\r\n");

    let mut dictionary = HashMap::new();
    for pair in pairs {
        match pair.split_once(": ") {
            Some((key, value)) => {
                dictionary.insert(key, value);
            }
            None => {}
        };
    }

    dictionary.get(header).unwrap()
}

fn read_data(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    String::from_utf8_lossy(&buffer[..]).trim().to_string()
}

fn parse_header(header: &str) -> Header {
    let parts = header.split_whitespace().collect::<Vec<&str>>();
    Header {
        method: String::from(parts[0]),
        path: String::from(parts[1]),
        // http_version: String::from(parts[2]),
    }
}

struct Header {
    method: String,
    path: String,
    // http_version: String,
}
