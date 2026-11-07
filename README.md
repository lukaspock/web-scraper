# web-scraper

A lightweight web crawler written in Rust with full-text search, controlled through a small HTTP API and a React frontend ("Ruggle"). It respects `robots.txt`, rate-limits requests per origin and stores results in MongoDB.

> **History:** The crawler was originally built with raw TCP and TLS (hand-written HTTP requests and response parsing). `reqwest` was added later and is now used for `robots.txt`. The original TCP/TLS code is still used for page fetching (`src/crawl_async.rs`) and as a synchronous variant in `src/crawl.rs`.

## Features

- **Axum HTTP API:** Start a crawl, list crawled URLs and search them (see [API](#api)). CORS is permissive so the frontend can call it.
- **Link extraction:** Scans the response body for anchor tags (`<a href="...">`) and handles different quotation formats.
- **robots.txt support:** Fetches and caches `robots.txt` per origin with `reqwest` and checks every URL with the `robotstxt` crate before crawling it (`src/robots.rs`).
- **Rate limiting:** A per-origin rate limiter spaces requests one second apart (`src/rate_limiter.rs`).
- **Concurrent crawling:** A `VecDeque` holds the URL queue. The crawl loop spawns one task per URL in a `tokio::task::JoinSet` and finishes once the queue is empty and all tasks are done.
- **Full-text search:** Besides the raw body, a `filtered_body` (visible text without HTML tags, `<style>` and `<script>`) is stored and indexed with a MongoDB text index.
- **Frontend:** A small React + TypeScript + Vite app (`src/crawler-frontend`) with a search box that queries `/search`.
- **Persistence:** Every crawled page (URL, status code, body, filtered body) is saved to MongoDB. On startup, previously saved URLs are loaded so they are not crawled again.

## Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- A MongoDB instance. A ready-made one is provided in `docker-compose.yml` (requires [Docker](https://docs.docker.com/get-docker/)).

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
5. (Optional) Start the frontend in a second terminal:
    ```bash
    cd src/crawler-frontend
    npm install
    npm run dev
    ```
    Vite prints the local URL (usually `http://localhost:5173`).

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

### `GET /search?text=...`: full-text search

Searches the `filtered_body` of all crawled pages and returns up to 20 matches.

```bash
curl "http://localhost:3000/search?text=love"
```

```json
[{ "url": "https://quotes.toscrape.com", "title": null }]
```

## Project Structure

| File                                          | Purpose                                                                   |
| --------------------------------------------- | ------------------------------------------------------------------------- |
| `src/main.rs`                                 | Axum server, crawl loop, MongoDB access                                   |
| `src/crawl_async.rs`                          | Async TCP/TLS fetching (original approach), HTTP parsing, link extraction |
| `src/crawl.rs`                                | Synchronous variant of the crawler                                        |
| `src/robots.rs`                               | `robots.txt` fetching, caching and matching                               |
| `src/rate_limiter.rs`                         | Per-origin request spacing                                                |
| `src/http_response.rs`, `src/visited_site.rs` | Data types                                                                |
| `src/crawler-frontend/`                       | React search frontend (Vite)                                              |

## Tech Stack

Rust, Tokio, Axum, Reqwest, MongoDB, React, TypeScript, Vite, TanStack Query
