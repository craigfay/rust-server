use std::net::TcpStream;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::fs;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}


fn handle_connection(mut stream: TcpStream) {
    println!("connection");
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let html = fs::read_to_string("index.html").unwrap();
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", html);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
