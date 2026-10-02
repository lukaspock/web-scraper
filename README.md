# web-scrapper

A lightweight, custom-built web crawler written in Rust. This project was developed from scratch to explore network programming, focusing on raw TCP connections, manual HTTP request generation, and string parsing without relying on heavy third-party HTTP clients.

## Features

* **Raw TCP Streams:** Uses Rust's standard `std::net::TcpStream` OR `tokio::net::TcpStream;` (for async) to establish connections directly to HTTPS.
* **Manual HTTP Protocol:** Constructs and sends raw `GET` requests and manually parses the incoming HTTP headers and body.
* **HTML Parsing:** Scans the response body for anchor tags (`<a href="...">`) to safely extract URLs, handling various quotation formats.
* **Looped Crawling:** In main.rs there is a simple VecDeque, used to store all of the remaining urls. In the loop the main function keeps spawning new tasks to process all of the urls

## Prerequisites

* [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.

## Getting Started

1. Clone the repository and navigate to the project directory:
   ```bash
   
   cd web-scrapper

## Build and run the Project

* (Optional) change the target-URL in main.rs
* cargo run

* Note: The starting target address is currently hardcoded in main.rs.
* Known Limitations & Educational Context
* This scraper is built as an educational project to understand the underlying mechanics of the web. The scrapper uses HTTPS BUT it will likely be intercepted by Web Application Firewalls (like Cloudflare) or redirected by servers enforcing HTTP.

* Tech Stack
Language: Rust


