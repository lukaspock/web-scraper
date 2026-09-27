use std::io::{Read, Write};
use std::net::TcpStream;
use std::string;
use std::time::Instant;

fn main() {
    println!("Hello, world!");

    let mut address: String = String::from("example.com:80");

    let start = Instant::now();

    match TcpStream::connect(&address) {
        Ok(stream) => {
            println!("Yoho! Successfully conncted to {:?}", address);
            println!("The peer: {}", stream.peer_addr().unwrap());

            get_http_response(stream, &address);
        }
        Err(err) => {
            println!("Failed to connect: {}", err);
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    
}

fn get_http_response(mut stream: TcpStream, address: &str) -> String {

    let buffer = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", address);

    stream.write_all(buffer.as_bytes()).unwrap();

    let mut buf = Vec::<u8>::new();
    
    stream.read_to_end(&mut buf).unwrap();

    let response = String::from_utf8_lossy(&buf).to_string();

    print!("{:?}", response);

    response
}
