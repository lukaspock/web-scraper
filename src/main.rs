use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::macos::raw::stat;
use std::string;
use std::time::Instant;

struct HttpResponse{
    status_code: u16,
    headers: HashMap<String,String>,
    body: String,
}

fn main() {

let address = "info.cern.ch:80";

    crawl(address);

    
}

fn crawl(address: &str){


    match TcpStream::connect(&address) {
            Ok(stream) => {
                println!("Yoho! Successfully conncted to {:?}", address);
                println!("The peer: {}", stream.peer_addr().unwrap());

                let mut response = get_http_response(&stream, &address);

                let http_response_obj = parse_http_response(&response, &stream);

                let additional_routes = get_linked_routes(&http_response_obj);

                println!("additional_routes: {:?}", additional_routes);

                for next_route in additional_routes{
                    let address = next_route.to_string() + ":80";
                    println!("{:?}", address);
                    crawl(&address);
                 }

            }
            Err(err) => {
                println!("Failed to connect: {}", err);
                return;
            }
        }

}

fn get_http_response(mut stream: &TcpStream, address: &str) -> String {

    let buffer = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", address);

    stream.write_all(buffer.as_bytes()).unwrap();

    let mut buf = Vec::<u8>::new();

    stream.read_to_end(&mut buf).unwrap();

    let response = String::from_utf8_lossy(&buf).to_string();

    response
}


fn parse_http_response(repsonse: &str, mut stream: &TcpStream) -> HttpResponse {

    let parts: Vec<&str> = repsonse.splitn(2,"\r\n\r\n").collect();
    let header_section = parts[0];
    let body_section = parts.get(1).unwrap_or(&"");

    let mut lines = header_section.lines();
    let status_line = lines.next().unwrap_or("");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);

    let mut headers = HashMap::new();

    for line in header_section.lines().skip(1) {
        if let Some((key, value)) = line.split_once(": "){
            headers.insert(key.to_string(), value.to_string());
        }

    };

    HttpResponse { 
        status_code, 
        headers, 
        body: body_section.to_string() 
    }

}

fn get_linked_routes(responseObj : &HttpResponse) -> Vec<&str> {

    let body: &str = &responseObj.body;

    let mut links: Vec<&str> = Vec::<&str>::new();

    for line in body.lines() {
        
        if(line.contains("<a")){

            println!("{:?}", line);

            let sectioned: Vec<&str> = line.split("href").filter(|v| v.contains("http")).collect();

            for section in sectioned{
                
                print!("116: ");
                println!("{:?}", section);

                let (_, after_https) =  section
                                .split_once(("https://"))
                                .or_else( || section.split_once("http://"))
                                .unwrap_or(("",""));

                println!("{:?}", after_https);

                let (url, _rest) = after_https.split_once(">").unwrap_or(("",""));

                let (url, _rest) = url.split_once("/").unwrap_or(("",""));

                println!("RESULT: {:?}", url);

                links.push(url);
            }
        }
    }

    links

}
