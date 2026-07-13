#![forbid(unsafe_code)]

use rmcp::model::*;
use rmcp::server::ServerHandler;
use rmcp::serve;
use rmcp::ServiceExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub mod search;
pub mod extract;
pub mod synthesize;

pub use search::SearchEngine;
pub use extract::ContentExtractor;
pub use synthesize::Synthesizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQuery {
    pub question: String,
    pub max_sources: Option<usize>,
    pub depth: Option<SearchDepth>,
    pub include_synthesis: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SearchDepth {
    Quick,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub question: String,
    pub sources: Vec<SourceResult>,
    pub synthesis: Option<String>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult {
    pub url: String,
    pub title: String,
    pub relevance_score: f64,
    pub content: String,
    pub extracted_at: String,
}

pub struct DeepResearchServer {
    search_engine: SearchEngine,
    extractor: ContentExtractor,
    synthesizer: Synthesizer,
    results_cache: Arc<RwLock<HashMap<String, ResearchResult>>>,
}

impl DeepResearchServer {
    pub fn new() -> Self {
        Self {
            search_engine: SearchEngine::new(),
            extractor: ContentExtractor::new(),
            synthesizer: Synthesizer::new(),
            results_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn research(&self, query: ResearchQuery) -> Result<ResearchResult, anyhow::Error> {
        let start = std::time::Instant::now();

        let max_sources = query.max_sources.unwrap_or(5);
        let depth = query.depth.unwrap_or(SearchDepth::Quick);

        // Phase 1: Search
        let search_results = self.search_engine
            .search(&query.question, max_sources * 2, depth)
            .await?;

        // Phase 2: Extract content from top sources
        let top_urls: Vec<&str> = search_results.iter()
            .take(max_sources)
            .map(|r| r.url.as_str())
            .collect();

        let mut sources = self.extractor
            .extract_batch(&top_urls)
            .await?;

        // Merge search metadata with extracted content
        for source in &mut sources {
            if let Some(sr) = search_results.iter().find(|r| r.url == source.url) {
                source.title = sr.title.clone();
                source.relevance_score = sr.relevance_score;
            }
        }

        // Phase 3: Synthesis (optional)
        let synthesis = if query.include_synthesis.unwrap_or(true) {
            let synthesized = self.synthesizer
                .synthesize(&query.question, &sources)
                .await?;
            Some(synthesized)
        } else {
            None
        };

        let result = ResearchResult {
            question: query.question,
            sources,
            synthesis,
            processing_time_ms: start.elapsed().as_millis() as u64,
        };

        self.results_cache.write().await.insert(result.question.clone(), result.clone());

        Ok(result)
    }
}

impl Default for DeepResearchServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

pub async fn run_server() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let server = DeepResearchServer::new();
    let handler = Arc::new(server);

    info!("Starting Deep Research MCP server");

    serve(
        handler,
        rmcp::transport::stdio::StdioTransport,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_query_serde() {
        let q = ResearchQuery {
            question: "What is the latest in Rust?\"".into(),
            max_sources: Some(3),
            depth: Some(SearchDepth::Quick),
            include_synthesis: Some(true),
        };
        let json = serde_json::to_string(&q).unwrap();
        let back: ResearchQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(back.question, q.question);
    }

    #[test]
    fn test_deep_research_server_new() {
        let server = DeepResearchServer::new();
        assert!(server.results_cache.read().unwrap().is_empty());
    }

    #[test]
    fn test_source_result_serde() {
        let sr = SourceResult {
            url: "https://example.com".into(),
            title: "Example".into(),
            relevance_score: 0.95,
            content: "# Example\n\nContent here".into(),
            extracted_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&sr).unwrap();
        let back: SourceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.url, sr.url);
    }

    #[test]
    fn test_search_depth_default() {
        let depth = SearchDepth::Quick;
        match depth {
            SearchDepth::Quick => {}
            SearchDepth::Deep => panic!("wrong variant"),
        }
    }
}
