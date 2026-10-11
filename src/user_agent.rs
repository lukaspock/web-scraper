/// Product token used to match `User-agent:` groups in robots.txt (no spaces or contact info).
pub const ROBOTS_TOKEN: &str = "LukasPock-Crawler";

/// Full value sent in the `User-Agent` header of every request.
pub fn header_value() -> String {
    format!(
        "{}/{} (+https://github.com/lukaspock/web-scraper; lukas1.pock@gmail.com)",
        ROBOTS_TOKEN,
        env!("CARGO_PKG_VERSION")
    )
}
