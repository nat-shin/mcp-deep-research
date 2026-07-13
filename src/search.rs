use crate::{SearchDepth, SourceResult};
use std::time::Duration;

pub struct SearchEngine {
    client: reqwest::Client,
}

impl SearchEngine {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("DeepResearch/0.1")
            .build()
            .expect("valid reqwest client");

        Self { client }
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        _depth: SearchDepth,
    ) -> Result<Vec<SourceResult>, anyhow::Error> {
        let encoded = urlencoding(query);
        let url = format!("https://html.duckduckgo.com/html/?q={encoded}");

        let response = self.client
            .get(&url)
            .header("Accept", "text/html")
            .send()
            .await?;

        let html = response.text().await?;
        let results = parse_search_results(&html, limit);

        Ok(results)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn urlencoding(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

fn parse_search_results(html: &str, limit: usize) -> Vec<SourceResult> {
    use scraper::{Html, Selector};

    let document = Html::parse_fragment(html);
    let result_selector = Selector::parse(".result__body").unwrap();
    let link_selector = Selector::parse(".result__a").unwrap();
    let snippet_selector = Selector::parse(".result__snippet").unwrap();

    let mut results = Vec::new();

    for element in document.select(&result_selector) {
        if results.len() >= limit {
            break;
        }

        let title = element
            .select(&link_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        let href = element
            .select(&link_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .map(|h| clean_url(h))
            .unwrap_or_default();

        let snippet = element
            .select(&snippet_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();

        if !href.is_empty() {
            results.push(SourceResult {
                url: href,
                title: title.trim().to_string(),
                relevance_score: 1.0 - (results.len() as f64 * 0.05),
                content: snippet.trim().to_string(),
                extracted_at: chrono_now(),
            });
        }
    }

    results
}

fn clean_url(href: &str) -> String {
    // DuckDuckGo wraps URLs in redirect
    if let Some(start) = href.find("uddg=") {
        let remaining = &href[start + 5..];
        if let Some(end) = remaining.find('&') {
            return url_decode(&remaining[..end]);
        }
        return url_decode(remaining);
    }
    href.to_string()
}

fn url_decode(s: &str) -> String {
    let mut decoded = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                decoded.push(byte as char);
            }
        } else {
            decoded.push(c);
        }
    }
    decoded
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Simple ISO-like timestamp without chrono dep
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    format!("T{:02}:{:02}:{:02}Z", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
        assert_eq!(urlencoding("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_url_decoding() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a%26b%3Dc"), "a&b=c");
    }

    #[test]
    fn test_parse_search_results_empty() {
        let results = parse_search_results("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_engine_new() {
        let engine = SearchEngine::new();
        // just verifies client creation doesn't panic
        assert!(engine.client.deref() == &engine.client);
    }
}
