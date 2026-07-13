use crate::SourceResult;
use std::time::Duration;

pub struct ContentExtractor {
    client: reqwest::Client,
}

impl ContentExtractor {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("DeepResearch/0.1")
            .build()
            .expect("valid reqwest client");

        Self { client }
    }

    pub async fn extract_batch(&self, urls: &[&str]) -> Result<Vec<SourceResult>, anyhow::Error> {
        let mut results = Vec::with_capacity(urls.len());

        for &url in urls {
            match self.extract_single(url).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!(url = %url, error = %e, "Failed to extract content");
                    results.push(SourceResult {
                        url: url.to_string(),
                        title: String::new(),
                        relevance_score: 0.0,
                        content: format!("Error extracting: {e}"),
                        extracted_at: String::new(),
                    });
                }
            }
        }

        Ok(results)
    }

    pub async fn extract_single(&self, url: &str) -> Result<SourceResult, anyhow::Error> {
        let response = self.client
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {status} for {url}");
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let html = response.text().await?;
        let markdown = html_to_markdown(&html);
        let title = extract_title(&html);

        Ok(SourceResult {
            url: url.to_string(),
            title,
            relevance_score: 0.0,
            content: markdown,
            extracted_at: chrono_now(),
        })
    }
}

impl Default for ContentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

fn html_to_markdown(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let body_selector = Selector::parse("body").unwrap_or_else(|_| Selector::parse("*").unwrap());

    let mut markdown = String::new();
    let body = document.select(&body_selector).next();

    if let Some(body_elem) = body {
        extract_text(&body_elem, &mut markdown, 0);
    }

    markdown
}

fn extract_text(element: &scraper::ElementRef, output: &mut String, depth: usize) {
    let tag = element.value().name();

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = tag[1..].parse::<usize>().unwrap_or(1);
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                output.push_str(&"\n".repeat(2));
                output.push_str(&"#".repeat(level));
                output.push(' ');
                output.push_str(&text);
                output.push('\n');
            }
        }
        "p" | "li" => {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                output.push('\n');
                output.push_str(&text);
                output.push('\n');
            }
        }
        "a" => {
            let href = element.value().attr("href").unwrap_or("");
            let text = element.text().collect::<String>();
            if !text.is_empty() && !href.is_empty() {
                output.push_str(&format!("[{text}]({href})"));
            }
        }
        "img" => {
            let alt = element.value().attr("alt").unwrap_or("");
            let src = element.value().attr("src").unwrap_or("");
            output.push_str(&format!("![{alt}]({src})"));
        }
        "pre" | "code" => {
            let text = element.text().collect::<String>();
            output.push_str(&format!("\n```\n{text}\n```\n"));
        }
        _ => {}
    }

    for child in element.children() {
        if let Some(child_elem) = scraper::ElementRef::wrap(child) {
            extract_text(&child_elem, output, depth + 1);
        }
    }
}

fn extract_title(html: &str) -> String {
    use scraper::{Html, Selector};
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").unwrap();
    document
        .select(&selector)
        .next()
        .map(|e| e.text().collect::<String>().trim().to_string())
        .unwrap_or_default()
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
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
    fn test_html_to_markdown_simple() {
        let html = "<h1>Hello</h1><p>World</p>";
        let md = html_to_markdown(html);
        assert!(md.contains("Hello"));
        assert!(md.contains("World"));
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        let title = extract_title(html);
        assert_eq!(title, "Test Page");
    }

    #[test]
    fn test_content_extractor_new() {
        let extractor = ContentExtractor::new();
        assert!(extractor.client.deref() == &extractor.client);
    }
}
