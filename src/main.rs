use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut packet = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();
    loop {
        let mut temp_buffer: Vec<u8> = Vec::new();
        packet.read_until(b'\n', &mut temp_buffer).unwrap();
        if temp_buffer.is_empty() {
            // No more data to read
            break;
        }
        buffer.extend_from_slice(&temp_buffer);
        if temp_buffer.len() == 2 && temp_buffer == b"\r\n" {
            // End of headers
            // TODO: read content
            break;
        }
        println!("Request: {:?}", temp_buffer.clone());
        println!("Request: {}", String::from_utf8(buffer.clone()).unwrap());
    }
    println!("Request: {}", String::from_utf8(buffer).unwrap());
    let response_body = "Hello, world!";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
