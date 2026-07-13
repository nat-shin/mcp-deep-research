use crate::SourceResult;

pub struct Synthesizer;

impl Synthesizer {
    pub fn new() -> Self {
        Self
    }

    pub async fn synthesize(
        &self,
        question: &str,
        sources: &[SourceResult],
    ) -> Result<String, anyhow::Error> {
        let mut output = String::new();

        output.push_str(&format!("# Research Synthesis\n\n"));
        output.push_str(&format!("**Question:** {question}\n\n"));
        output.push_str(&format!("**Sources consulted:** {}\n\n", sources.len()));

        if sources.is_empty() {
            output.push_str("*No sources were found for this query.*\n");
            return Ok(output);
        }

        output.push_str("## Key Findings\n\n");

        for (i, source) in sources.iter().enumerate() {
            let content_preview: String = source
                .content
                .chars()
                .take(500)
                .collect();

            output.push_str(&format!(
                "### {}. {}\n\n",
                i + 1,
                if source.title.is_empty() { &source.url } else { &source.title }
            ));
            output.push_str(&format!("**Source:** [{}]({})\n\n", source.url, source.url));
            output.push_str(&format!("**Relevance:** {:.1}%\n\n", source.relevance_score * 100.0));
            output.push_str(&content_preview);
            output.push_str("\n\n---\n\n");
        }

        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "Collected {} sources across the web. ",
            sources.len()
        ));
        output.push_str(&format!(
            "Total extracted content: {} characters.\n",
            sources.iter().map(|s| s.content.len()).sum::<usize>()
        ));

        Ok(output)
    }
}

impl Default for Synthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_empty_sources() {
        let synth = Synthesizer::new();
        let result = synth.synthesize("test", &[]).await.unwrap();
        assert!(result.contains("No sources were found"));
    }

    #[test]
    fn test_synthesize_with_source() {
        let synth = Synthesizer::new();
        let sources = vec![
            SourceResult {
                url: "https://example.com".into(),
                title: "Example".into(),
                relevance_score: 0.9,
                content: "Content here".into(),
                extracted_at: "".into(),
            },
        ];
        let result = synth.synthesize("test question", &sources).await.unwrap();
        assert!(result.contains("test question"));
        assert!(result.contains("example.com"));
        assert!(result.contains("Example"));
    }
}
