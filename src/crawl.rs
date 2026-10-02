
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::error::Error;

use crate::HttpResponse::HttpResponse;


pub fn start(address: &str) {

    let mut already_visited : HashSet<String> = HashSet::<String>::new();

    let (address, path) = split_into_domain_and_path(address);

    crawl(address, path, &mut already_visited);
}

fn connect_to_domain(address: &str) -> Result<TlsStream<TcpStream>, Box<dyn Error>> {

    let domain = address.split(':').next().unwrap_or(address);

    let stream = TcpStream::connect(address)?;

    let connector = TlsConnector::new()?;

    let tls_stream = connector.connect(domain, stream)?;

    Ok(tls_stream)
}


pub fn crawl(address: &str, path: &str, visited_sites: &mut HashSet<String>){

    if !address.starts_with("quotes.toscrape.com") {
        return;
    }

    let address_with_path = address.to_string() + "/" + path;

    if visited_sites.contains(&address_with_path){
        return
    };

    let stream = match connect_to_domain(address){
        Ok(s) => s,
        Err(err) => {
            println!("ERROR {:?}", err);
            return;
        }
    };

    println!("\n\nSuccessfully conncted to {:?}", address);

    let _ = visited_sites.insert(address_with_path);

    /* 
    print!("\n\nALREADY VISITED:");
    for site in visited_sites.clone() {
        print!("{}\n", site);
    }
    */
    
    print!("\n");


    let response = get_http_response(stream, &address, &path);
    let http_response_obj = parse_http_response(&response);

    println!("\n");
    println!("STATUS: {:?}", http_response_obj.status_code);
    println!("BODY: \n{:?}", http_response_obj.body);

    if http_response_obj.status_code == 302 {

        let new_address = match http_response_obj.headers.get("location") {
            Some(loc) => loc,
            None => {
                return;     
            }
        };

        println!("NEW LOCATION: {:?}", new_address);

        let (new_address, new_path) = split_into_domain_and_path(new_address);

        crawl(new_address, new_path, visited_sites);
    }

    let additional_routes = get_linked_routes(&http_response_obj, &address);

    println!("further routes: {:?}", &additional_routes);

    for next_route in additional_routes{

        let (pure_domain, path) = split_into_domain_and_path(&next_route);

        println!("{}{}", pure_domain, path);

        let address = format!("{}:443", pure_domain);

        crawl(&address, &path, visited_sites);
    }
}

fn get_http_response(mut stream: TlsStream<TcpStream>, address: &str, path: &str) -> String {

    let safe_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };


    let domain = address.split(':').next().unwrap_or(address);

    let buffer = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\r\nConnection: close\r\n\r\n", 
        safe_path,
        domain
    );

    stream.write_all(buffer.as_bytes()).unwrap();

    let mut buf = Vec::<u8>::new();

    stream.read_to_end(&mut buf).unwrap();

    let response = String::from_utf8_lossy(&buf).to_string();

    response
}


fn parse_http_response(repsonse: &str) -> HttpResponse {

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

fn get_linked_routes(response_obj : &HttpResponse, domain : &str) -> Vec<String> {

    let body: &str = &response_obj.body;

    let mut links: Vec<String> = Vec::<String>::new();

    for line in body.lines() {
        
        if line.contains("<a"){

            let sectioned: Vec<&str> = line.split("href").filter(|v| v.contains("http")).collect();

            for section in sectioned{

                let (_, after_url) =  section
                                .split_once("https://")
                                .or_else( || section.split_once("http://"))
                                .unwrap_or(("",""));

                if after_url.is_empty() {
                    continue;
                }

                let (url, _rest) = after_url.split_once(">").unwrap_or(("",""));
                let url = url.trim_matches('"').trim_matches('\'');

                links.push(url.to_string());
            }

            let local_refs: Vec<&str> = line.split("href").filter(|v| !v.contains("http")).collect();

            for local_ref in local_refs {


            if let Some(path) = local_ref.split('"').nth(1) {
                if path.starts_with('/') {
                    
                    let (domain, _) = domain.split_once(":").unwrap();

                    let new_path = domain.to_string() + path;

                    links.push(new_path)
                }
            }
}
        }
    }

    links

}

fn split_into_domain_and_path(address: &str) -> (&str, &str){
    let (pure_domain, path) = address.split_once('/').unwrap_or_else(|| (address, "/"));

    (pure_domain, path)
}
