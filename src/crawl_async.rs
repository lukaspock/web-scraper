
use axum::http::response;
use serde::de::value;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use tokio_native_tls::TlsStream;
use std::collections::HashMap;

use std::error::Error;
use std::fmt::format;

use robotstxt::DefaultMatcher;

use crate::http_response::HttpResponse;

// SWITCHED FORM COMPLETLY FROM SRATCH VIA TCP SCTREAMS TO REQWEST TO SOLVE PORT AND HTTP/HTTPS ISSUES

pub async fn crawl_async(address: &str, path: &str) -> Result<(Vec<String>, Option<HttpResponse>), Box<dyn Error>> {

    let mut further_links = Vec::<String>::new();

    println!("\n\nSuccessfully conncted to {:?}", address);

    let full_url = format!("{}{}", address, path);

   let http_response_obj = get_http_response( &full_url).await?;

    if http_response_obj.status_code == 302 {
        let new_address = match http_response_obj.headers.get("location") {
            Some(loc) => loc,
            None => ""
        };

        further_links.push(new_address.to_string());

        return Ok((further_links,None));
    }

    let additional_routes = get_linked_routes(&http_response_obj, &full_url).await;

    for route in additional_routes {
        further_links.push(route);
    }


    Ok((further_links, Some(http_response_obj)))
}


/* 
async fn connect_to_domain(address: &str) -> Result<tokio_native_tls::TlsStream<TcpStream>, Box<dyn Error>> {
    // Falls das address-Argument keinen Port enthält, hängen wir standardmäßig :443 an
    let full_address = if address.contains(':') {
        address.to_string()
    } else {
        format!("{}:443", address)
    };

    let domain = full_address.split(':').next().unwrap_or(&full_address);

    let stream = TcpStream::connect(&full_address).await?;

    let connector = native_tls::TlsConnector::new()?;
    let cx = TlsConnector::from(connector);

    let tls_stream = cx.connect(domain, stream).await?;

    Ok(tls_stream)
}
*/

async fn get_http_response(url: &str) -> Result<HttpResponse, Box<dyn Error>> {
/* 
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


    stream.write_all(buffer.as_bytes()).await.unwrap();

    let mut buf = Vec::<u8>::new();

    stream.read_to_end(&mut buf).await.unwrap();

    let response = String::from_utf8_lossy(&buf).to_string();
*/
    let response = reqwest::get("https://quotes.toscrape.com").await?;

    let status_code: u16 = response.status().as_u16();
    let mut headers: HashMap<String,String> = HashMap::<String,String>::new();

    for  (name, value) in response.headers() {
        let k = name.to_string();
        let v = value.to_str()?.to_string();
        headers.insert(k,v);
    }

    let body: String = response.text().await?;

    Ok(
        HttpResponse { status_code, headers, body}
    )
    
}

/* 
async fn parse_http_response(repsonse: &str) -> HttpResponse {

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
*/

async fn get_linked_routes(response_obj : &HttpResponse, domain : &str) -> Vec<String> {

    let body: &str = &response_obj.body;

    let mut links: Vec<String> = Vec::<String>::new();

    for line in body.lines() {
        
        if line.contains("<a"){

            let sectioned: Vec<&str> = line.split("href=").filter(|v| v.contains("http")).collect();

            for section in sectioned{

                if section.is_empty() {
                    continue;
                }

                let (url, _rest) = section.split_once(">").unwrap_or(("",""));
                let url = url.trim_matches('"').trim_matches('\'');

                links.push(url.to_string());
            }

            let local_refs: Vec<&str> = line.split("href\"").filter(|v| !v.contains("http")).collect();

            for local_ref in local_refs {


            if let Some(path) = local_ref.split('"').nth(1) {
                if path.starts_with('/') {

                    let new_path = format!("{}{}", domain, path);

                    links.push(new_path)
                }
            }
}
        }
    }

    links

}

pub async fn split_into_domain_and_path(address: &str) -> (&str, &str){
    let (pure_domain, path) = address.split_once('/').unwrap_or_else(|| (address, "/"));

    (pure_domain, path)
}
