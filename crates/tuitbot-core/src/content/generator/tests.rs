#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::content::generator::parser::parse_thread;
    use crate::content::length::MAX_TWEET_CHARS;
    use crate::error::LlmError;
    use crate::llm::{GenerationParams, LlmProvider, LlmResponse, TokenUsage};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Mock LLM provider that returns canned responses.
    struct MockProvider {
        responses: Vec<String>,
        call_count: Arc<AtomicUsize>,
    }

    impl MockProvider {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn single(response: &str) -> Self {
            Self::new(vec![response.to_string()])
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }

        async fn complete(
            &self,
            _system: &str,
            _user_message: &str,
            _params: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            let text = self
                .responses
                .get(idx)
                .cloned()
                .unwrap_or_else(|| self.responses.last().cloned().unwrap_or_default());

            Ok(LlmResponse {
                text,
                usage: TokenUsage::default(),
                model: "mock".to_string(),
            })
        }

        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    fn test_business() -> crate::config::BusinessProfile {
        crate::config::BusinessProfile {
            product_name: "TestApp".to_string(),
            product_description: "A test application".to_string(),
            product_url: Some("https://testapp.com".to_string()),
            target_audience: "developers".to_string(),
            product_keywords: vec!["test".to_string()],
            competitor_keywords: vec![],
            industry_topics: vec!["testing".to_string()],
            brand_voice: None,
            reply_style: None,
            content_style: None,
            persona_opinions: vec![],
            persona_experiences: vec![],
            content_pillars: vec![],
        }
    }

    // --- parse_thread tests ---

    #[test]
    fn parse_thread_with_dashes() {
        let text = "Tweet one\n---\nTweet two\n---\nTweet three";
        let tweets = parse_thread(text);
        assert_eq!(tweets.len(), 3);
        assert_eq!(tweets[0], "Tweet one");
        assert_eq!(tweets[1], "Tweet two");
        assert_eq!(tweets[2], "Tweet three");
    }

    #[test]
    fn parse_thread_with_extra_whitespace() {
        let text = "  Tweet one  \n---\n  Tweet two  \n---\n";
        let tweets = parse_thread(text);
        assert_eq!(tweets.len(), 2);
        assert_eq!(tweets[0], "Tweet one");
        assert_eq!(tweets[1], "Tweet two");
    }

    #[test]
    fn parse_thread_single_block_falls_back_to_numbered() {
        let text =
            "1/5 First tweet\n2/5 Second tweet\n3/5 Third tweet\n4/5 Fourth tweet\n5/5 Fifth tweet";
        let tweets = parse_thread(text);
        assert!(
            tweets.len() >= 2,
            "got {} tweets: {:?}",
            tweets.len(),
            tweets
        );
    }

    #[test]
    fn parse_thread_empty_sections_filtered() {
        let text = "---\n---\nActual tweet\n---\n---";
        let tweets = parse_thread(text);
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0], "Actual tweet");
    }

    // --- generate_reply tests ---

    #[tokio::test]
    async fn generate_reply_success() {
        let provider =
            MockProvider::single("Great point about testing! I've found similar results.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_reply("Testing is important", "devuser", true)
            .await
            .expect("reply");
        assert!(output.text.len() <= MAX_TWEET_CHARS);
        assert!(!output.text.is_empty());
        assert_eq!(output.provider, "mock");
    }

    #[tokio::test]
    async fn generate_reply_truncates_long_output() {
        let long_text = "a ".repeat(200); // 400 chars
        let provider = MockProvider::new(vec![long_text.clone(), long_text]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_reply("test", "user", true)
            .await
            .expect("reply");
        assert!(output.text.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn generate_reply_no_product_mention() {
        let provider = MockProvider::single("That's a great approach for productivity!");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_reply("How do you stay productive?", "devuser", false)
            .await
            .expect("reply");
        assert!(output.text.len() <= MAX_TWEET_CHARS);
        assert!(!output.text.is_empty());
    }

    // --- generate_tweet tests ---

    #[tokio::test]
    async fn generate_tweet_success() {
        let provider =
            MockProvider::single("Testing your code early saves hours of debugging later.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_tweet("testing best practices")
            .await
            .expect("tweet");
        assert!(output.text.len() <= MAX_TWEET_CHARS);
        assert!(!output.text.is_empty());
    }

    // --- generate_thread tests ---

    #[tokio::test]
    async fn generate_thread_success() {
        let thread_text = vec![
            "Hook tweet here",
            "---",
            "Second point about testing",
            "---",
            "Third point about quality",
            "---",
            "Fourth point about CI/CD",
            "---",
            "Fifth point about automation",
            "---",
            "Summary and call to action",
        ]
        .join("\n");

        let provider = MockProvider::single(&thread_text);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen.generate_thread("testing").await.expect("thread");
        assert!(
            (5..=8).contains(&output.tweets.len()),
            "got {} tweets",
            output.tweets.len()
        );
        for tweet in &output.tweets {
            assert!(tweet.len() <= MAX_TWEET_CHARS);
        }
    }

    #[tokio::test]
    async fn generate_thread_retries_on_bad_count() {
        let bad = "Tweet one\n---\nTweet two";
        let good = "One\n---\nTwo\n---\nThree\n---\nFour\n---\nFive";
        let provider = MockProvider::new(vec![bad.into(), bad.into(), good.into()]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen.generate_thread("topic").await.expect("thread");
        assert_eq!(output.tweets.len(), 5);
    }

    #[tokio::test]
    async fn generate_thread_fails_after_max_retries() {
        let bad = "Tweet one\n---\nTweet two";
        let provider = MockProvider::new(vec![bad.into(), bad.into(), bad.into()]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let err = gen.generate_thread("topic").await.unwrap_err();
        assert!(matches!(err, LlmError::GenerationFailed(_)));
    }

    // --- generate_reply_with_context tests ---

    #[tokio::test]
    async fn generate_reply_with_context_injects_rag() {
        let provider = MockProvider::single("Great insight about testing patterns!");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let rag_block = "Winning patterns:\n1. [tip] (tweet): \"Great advice\"";
        let output = gen
            .generate_reply_with_context("Test tweet", "user", false, None, Some(rag_block))
            .await
            .expect("reply");

        assert!(!output.text.is_empty());
        assert!(output.text.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn generate_reply_with_context_none_matches_archetype() {
        let provider = MockProvider::single("Agreed, great point!");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_reply_with_context("Test tweet", "user", false, None, None)
            .await
            .expect("reply");
        assert!(!output.text.is_empty());
    }

    // --- generate_tweet_with_context tests ---

    #[tokio::test]
    async fn generate_tweet_with_context_injects_rag() {
        let provider = MockProvider::single("Testing early saves debugging time.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let rag_block = "Winning patterns:\n1. [tip] (tweet): \"Great advice\"";
        let output = gen
            .generate_tweet_with_context("testing", None, Some(rag_block))
            .await
            .expect("tweet");
        assert!(!output.text.is_empty());
        assert!(output.text.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn generate_tweet_with_context_none_matches_base() {
        let provider = MockProvider::single("Testing matters for quality.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_tweet_with_context("testing", None, None)
            .await
            .expect("tweet");
        assert!(!output.text.is_empty());
    }

    // --- generate_thread_with_context tests ---

    #[tokio::test]
    async fn generate_thread_with_context_injects_rag() {
        let thread_text =
            "Hook\n---\nPoint 1\n---\nPoint 2\n---\nPoint 3\n---\nPoint 4\n---\nSummary";
        let provider = MockProvider::single(thread_text);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let rag_block = "Winning patterns:\n1. [tip] (tweet): \"Great advice\"";
        let output = gen
            .generate_thread_with_context("testing", None, Some(rag_block))
            .await
            .expect("thread");
        assert!((5..=8).contains(&output.tweets.len()));
    }

    // --- GenerationParams tests ---

    #[test]
    fn generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.max_tokens, 512);
        assert!((params.temperature - 0.7).abs() < f32::EPSILON);
        assert!(params.system_prompt.is_none());
    }
}
