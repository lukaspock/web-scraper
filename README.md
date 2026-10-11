# web-scraper

A lightweight web crawler written in Rust, controlled through a small HTTP API. It respects `robots.txt`, rate-limits requests per origin and stores results in MongoDB.

> **History:** The crawler was originally built with raw TCP and TLS (hand-written HTTP requests and response parsing). `reqwest` was added later and is now used for `robots.txt`. The original TCP/TLS code is still used for page fetching (`src/crawl_async.rs`) and as a synchronous variant in `src/crawl.rs`.

## Features

* **Axum HTTP API:** Start a crawl with a `POST` request and list previously crawled URLs with a `GET` request (see [API](#api)).
* **Link extraction:** Scans the response body for anchor tags (`<a href="...">`) and handles different quotation formats.
* **robots.txt support:** Fetches and caches `robots.txt` per origin with `reqwest` and checks every URL with the `robotstxt` crate before crawling it (`src/robots.rs`).
* **Rate limiting:** A per-origin rate limiter spaces requests one second apart (`src/rate_limiter.rs`).
* **Concurrent crawling:** A `VecDeque` holds the URL queue. The crawl loop spawns one task per URL in a `tokio::task::JoinSet` and finishes once the queue is empty and all tasks are done.
* **Persistence:** Every crawled page (URL, status code, body) is saved to MongoDB. On startup, previously saved URLs are loaded so they are not crawled again.

## Prerequisites

* [Rust and Cargo](https://www.rust-lang.org/tools/install)
* A MongoDB instance. A ready-made one is provided in `docker-compose.yml` (requires [Docker](https://docs.docker.com/get-docker/)).

## Getting Started

1. Clone the repository and navigate to the project directory:
   ```bash
   cd web-scrapper
   ```
2. Start MongoDB:
   ```bash
   docker compose up -d
   ```
3. Create a `.env` file based on `.env.example`:
   ```
   MONGO_CONNECTION_URL=mongodb://localhost:27017
   MONGO_DB=crawler
   ```
4. Run the server:
   ```bash
   cargo run
   ```
   The API listens on `0.0.0.0:3000`.

## API

### `POST /`: start a crawl

Starts crawling at the given URL and follows the links it finds. The request returns when the crawl has finished.

```bash
curl -X POST http://localhost:3000/ \
  -H "Content-Type: application/json" \
  -d '{"url": "https://quotes.toscrape.com"}'
```

### `GET /`: list crawled URLs

Returns every URL that has been visited so far, including those loaded from MongoDB at startup.

```bash
curl http://localhost:3000/
```

```json
{ "previous_urls": ["https://quotes.toscrape.com", "..."] }
```

## Project Structure

| File | Purpose |
| --- | --- |
| `src/main.rs` | Axum server, crawl loop, MongoDB access |
| `src/crawl_async.rs` | Async TCP/TLS fetching (original approach), HTTP parsing, link extraction |
| `src/crawl.rs` | Synchronous variant of the crawler |
| `src/robots.rs` | `robots.txt` fetching, caching and matching |
| `src/rate_limiter.rs` | Per-origin request spacing |
| `src/http_response.rs`, `src/visited_site.rs` | Data types |

## Known Limitations

* This is an educational project. Because page fetching still speaks HTTP by hand, it will likely be blocked by Web Application Firewalls (such as Cloudflare) or redirected by servers that enforce other protocols or headers.
* The crawl is not limited to a single domain and has no depth or page limit.
* Only one start URL per request is supported, and the request blocks until the crawl is finished.

## Tech Stack

Rust, Tokio, Axum, Reqwest, native-tls, MongoDB
