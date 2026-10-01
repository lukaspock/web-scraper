# web-scrapper

A lightweight, custom-built web crawler written in Rust. This project was developed from scratch to explore network programming, focusing on raw TCP connections, manual HTTP request generation, and string parsing without relying on heavy third-party HTTP clients.

## Features

* **Raw TCP Streams:** Uses Rust's standard `std::net::TcpStream` to establish connections directly to port 80.
* **Manual HTTP Protocol:** Constructs and sends raw `GET` requests and manually parses the incoming HTTP headers and body.
* **HTML Parsing:** Scans the response body for anchor tags (`<a href="...">`) to safely extract URLs, handling various quotation formats.
* **Recursive Crawling:** Automatically queues and attempts to connect to newly discovered routes.

## Prerequisites

* [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.

## Getting Started

1. Clone the repository and navigate to the project directory:
   ```bash
   
   cd web-scrapper

## Build and run the Project

* cargo run

* Note: The starting target address is currently hardcoded in main.rs.
* Known Limitations & Educational Context
* This scraper is built as an educational project to understand the underlying mechanics of the web. Because it uses raw HTTP over port 80 and lacks modern browser headers or TLS (HTTPS) support, it will likely be intercepted by Web Application Firewalls (like Cloudflare) or redirected by servers enforcing HTTPS.

* Tech Stack
Language: Rust
Core Libraries: std::net::TcpStream, std::collections::HashMap, std::io::{Read, Write}
