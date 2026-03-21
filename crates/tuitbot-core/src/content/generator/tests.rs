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

    // --- improve_draft tests ---

    #[tokio::test]
    async fn improve_draft_success() {
        let provider = MockProvider::single("A sharper version of the draft tweet.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .improve_draft("This is my draft tweet about testing.", None)
            .await
            .expect("improve");
        assert!(!output.text.is_empty());
        assert!(output.text.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn improve_draft_with_tone_cue() {
        let provider = MockProvider::single("A punchy take on testing best practices.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .improve_draft(
                "Testing is important for code quality.",
                Some("Be punchy and bold"),
            )
            .await
            .expect("improve with tone");
        assert!(!output.text.is_empty());
        assert!(output.text.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn improve_draft_with_context_success() {
        let provider = MockProvider::single("An improved tweet grounded in winning patterns.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let rag_block = "Winning patterns:\n1. [tip] (tweet): \"Great advice\"";
        let output = gen
            .improve_draft_with_context("Draft about testing.", Some("Be casual"), Some(rag_block))
            .await
            .expect("improve with context");
        assert!(!output.text.is_empty());
        assert!(output.text.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn improve_draft_with_context_none_matches_base() {
        let provider = MockProvider::single("Improved tweet without context.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .improve_draft_with_context("Draft tweet.", Some("Be concise"), None)
            .await
            .expect("improve with None context");
        assert!(!output.text.is_empty());
    }

    // --- PromptCapturingProvider for system prompt assertions ---

    /// Mock LLM that captures the system prompt for assertion.
    struct PromptCapturingProvider {
        response: String,
        captured_system: Arc<tokio::sync::Mutex<Option<String>>>,
    }

    impl PromptCapturingProvider {
        fn new(response: &str) -> (Self, Arc<tokio::sync::Mutex<Option<String>>>) {
            let captured = Arc::new(tokio::sync::Mutex::new(None));
            (
                Self {
                    response: response.to_string(),
                    captured_system: Arc::clone(&captured),
                },
                captured,
            )
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for PromptCapturingProvider {
        fn name(&self) -> &str {
            "prompt_capturing_mock"
        }

        async fn complete(
            &self,
            system: &str,
            _user_message: &str,
            _params: &GenerationParams,
        ) -> Result<LlmResponse, LlmError> {
            let mut guard = self.captured_system.lock().await;
            *guard = Some(system.to_string());
            Ok(LlmResponse {
                text: self.response.clone(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
            })
        }

        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn improve_draft_with_context_injects_rag_in_prompt() {
        let (provider, captured) = PromptCapturingProvider::new("Improved tweet with RAG context.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let rag_block = "Winning patterns:\n1. [tip] (tweet): \"Great testing advice\"";
        gen.improve_draft_with_context("Draft tweet.", None, Some(rag_block))
            .await
            .expect("improve with RAG");

        let system = captured.lock().await;
        let system = system.as_ref().expect("system prompt captured");
        assert!(
            system.contains("Winning patterns"),
            "RAG block should appear in system prompt"
        );
    }

    #[tokio::test]
    async fn improve_draft_with_context_no_rag_when_none() {
        let (provider, captured) = PromptCapturingProvider::new("Improved tweet without RAG.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        gen.improve_draft_with_context("Draft tweet.", None, None)
            .await
            .expect("improve without RAG");

        let system = captured.lock().await;
        let system = system.as_ref().expect("system prompt captured");
        // When rag_context is None, format_rag_section returns empty string.
        // The system prompt should not contain any RAG-specific content.
        assert!(
            !system.contains("Winning patterns"),
            "No RAG block should appear when context is None"
        );
    }

    // --- GenerationParams tests ---

    #[test]
    fn generation_params_default() {
        let params = GenerationParams::default();
        assert_eq!(params.max_tokens, 512);
        assert!((params.temperature - 0.7).abs() < f32::EPSILON);
        assert!(params.system_prompt.is_none());
    }

    // --- Helper function branch coverage via prompt capture ---

    #[tokio::test]
    async fn voice_section_included_when_brand_voice_set() {
        let mut biz = test_business();
        biz.brand_voice = Some("Friendly, casual, and approachable".to_string());
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Friendly, casual, and approachable"));
    }

    #[tokio::test]
    async fn voice_section_absent_when_none() {
        let mut biz = test_business();
        biz.brand_voice = None;
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(!system.contains("Voice & personality"));
    }

    #[tokio::test]
    async fn voice_section_absent_when_empty_string() {
        let mut biz = test_business();
        biz.brand_voice = Some(String::new());
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(!system.contains("Voice & personality"));
    }

    #[tokio::test]
    async fn audience_section_included_when_non_empty() {
        let biz = test_business(); // target_audience = "developers"
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", true)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Your audience: developers"));
    }

    #[tokio::test]
    async fn audience_section_absent_when_empty() {
        let mut biz = test_business();
        biz.target_audience = String::new();
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", true)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(!system.contains("Your audience"));
    }

    #[tokio::test]
    async fn persona_context_with_opinions_and_experiences() {
        let mut biz = test_business();
        biz.persona_opinions = vec!["Rust is the future".to_string()];
        biz.persona_experiences = vec!["Built CLI tools for 5 years".to_string()];
        biz.content_pillars = vec!["Developer productivity".to_string()];

        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Opinions you hold: Rust is the future"));
        assert!(system.contains("Experiences you can reference: Built CLI tools for 5 years"));
        assert!(system.contains("Content pillars: Developer productivity"));
    }

    #[tokio::test]
    async fn persona_context_empty_when_no_persona() {
        let biz = test_business(); // persona_opinions/experiences/pillars all empty
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(!system.contains("Opinions you hold"));
        assert!(!system.contains("Experiences you can reference"));
        assert!(!system.contains("Content pillars"));
    }

    #[tokio::test]
    async fn reply_style_custom_when_set() {
        let mut biz = test_business();
        biz.reply_style = Some("Be witty and concise".to_string());
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Be witty and concise"));
    }

    #[tokio::test]
    async fn reply_style_default_when_none() {
        let biz = test_business(); // reply_style = None
        let (provider, captured) = PromptCapturingProvider::new("Short reply.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_reply("test", "user", false)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Be conversational and helpful"));
    }

    #[tokio::test]
    async fn content_style_custom_for_tweet() {
        let mut biz = test_business();
        biz.content_style = Some("Sharp and data-driven".to_string());
        let (provider, captured) = PromptCapturingProvider::new("Short tweet.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_tweet("testing").await.expect("tweet");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Sharp and data-driven"));
    }

    #[tokio::test]
    async fn content_style_default_for_tweet() {
        let biz = test_business(); // content_style = None
        let (provider, captured) = PromptCapturingProvider::new("Short tweet.");
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_tweet("testing").await.expect("tweet");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Be informative and engaging"));
    }

    #[test]
    fn generation_output_debug_and_clone() {
        let output = GenerationOutput {
            text: "hello".to_string(),
            usage: TokenUsage::default(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
        };
        let clone = output.clone();
        assert_eq!(clone.text, "hello");
        let debug = format!("{output:?}");
        assert!(debug.contains("hello"));
    }

    #[test]
    fn thread_generation_output_debug_and_clone() {
        let output = ThreadGenerationOutput {
            tweets: vec!["a".to_string(), "b".to_string()],
            usage: TokenUsage::default(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
        };
        let clone = output.clone();
        assert_eq!(clone.tweets.len(), 2);
        let debug = format!("{output:?}");
        assert!(debug.contains("gpt-4"));
    }

    #[test]
    fn business_accessor() {
        let biz = test_business();
        let gen = ContentGenerator::new(Box::new(MockProvider::single("test")), biz);
        assert_eq!(gen.business().product_name, "TestApp");
    }

    #[test]
    fn rag_section_empty_string_returns_empty() {
        let result = ContentGenerator::format_rag_section(Some(""));
        assert_eq!(result, "");
    }

    #[test]
    fn rag_section_none_returns_empty() {
        let result = ContentGenerator::format_rag_section(None);
        assert_eq!(result, "");
    }

    #[test]
    fn rag_section_with_content() {
        let result = ContentGenerator::format_rag_section(Some("context here"));
        assert_eq!(result, "\ncontext here");
    }

    // --- Additional parse_thread edge cases ---

    #[test]
    fn parse_thread_numbered_with_dots() {
        let text =
            "1. First tweet\n2. Second tweet\n3. Third tweet\n4. Fourth tweet\n5. Fifth tweet";
        let tweets = parse_thread(text);
        assert!(
            tweets.len() >= 2,
            "numbered dot format: got {} tweets",
            tweets.len()
        );
    }

    #[test]
    fn parse_thread_numbered_with_parens() {
        let text =
            "1) First tweet\n2) Second tweet\n3) Third tweet\n4) Fourth tweet\n5) Fifth tweet";
        let tweets = parse_thread(text);
        assert!(
            tweets.len() >= 2,
            "numbered paren format: got {} tweets",
            tweets.len()
        );
    }

    #[test]
    fn parse_thread_empty_input() {
        let tweets = parse_thread("");
        assert!(tweets.is_empty());
    }

    #[test]
    fn parse_thread_only_whitespace() {
        let tweets = parse_thread("   \n\n   ");
        assert!(tweets.is_empty());
    }

    #[test]
    fn parse_thread_single_tweet_no_delimiters() {
        let tweets = parse_thread("Just a single tweet with no delimiters");
        assert_eq!(tweets.len(), 1);
        assert_eq!(tweets[0], "Just a single tweet with no delimiters");
    }

    #[test]
    fn parse_thread_mixed_content_with_dashes() {
        let text = "Hook tweet\n---\n  \n---\nMiddle tweet\n---\nFinal tweet";
        let tweets = parse_thread(text);
        // Empty sections should be filtered
        assert_eq!(tweets.len(), 3);
    }

    // --- Additional generator branch coverage ---

    #[tokio::test]
    async fn generate_reply_with_archetype() {
        use crate::content::frameworks::ReplyArchetype;

        let provider = MockProvider::single("I totally agree and would add...");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_reply_with_archetype(
                "Testing is critical",
                "devuser",
                true,
                Some(ReplyArchetype::AgreeAndExpand),
            )
            .await
            .expect("reply with archetype");
        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn generate_tweet_with_format() {
        use crate::content::frameworks::TweetFormat;

        let provider =
            MockProvider::single("1. Test early\n2. Test often\n3. Ship with confidence");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_tweet_with_format("testing", Some(TweetFormat::List))
            .await
            .expect("tweet with format");
        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn generate_thread_with_structure() {
        use crate::content::frameworks::ThreadStructure;

        let thread = "Hook\n---\nStep 1\n---\nStep 2\n---\nStep 3\n---\nStep 4\n---\nSummary";
        let provider = MockProvider::single(thread);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_thread_with_structure("testing", Some(ThreadStructure::Framework))
            .await
            .expect("thread with structure");
        assert!((5..=8).contains(&output.tweets.len()));
    }

    #[tokio::test]
    async fn content_style_custom_for_thread() {
        let mut biz = test_business();
        biz.content_style = Some("Deep and technical".to_string());
        let thread = "Hook\n---\nPoint 1\n---\nPoint 2\n---\nPoint 3\n---\nPoint 4\n---\nSummary";
        let (provider, captured) = PromptCapturingProvider::new(thread);
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_thread("testing").await.expect("thread");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Deep and technical"));
    }

    #[tokio::test]
    async fn content_style_default_for_thread() {
        let biz = test_business(); // content_style = None
        let thread = "Hook\n---\nPoint 1\n---\nPoint 2\n---\nPoint 3\n---\nPoint 4\n---\nSummary";
        let (provider, captured) = PromptCapturingProvider::new(thread);
        let gen = ContentGenerator::new(Box::new(provider), biz);

        gen.generate_thread("testing").await.expect("thread");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("Be informative, not promotional"));
    }

    #[tokio::test]
    async fn improve_draft_empty_tone_cue_ignored() {
        let (provider, captured) = PromptCapturingProvider::new("Improved draft.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        gen.improve_draft("Draft tweet.", Some(""))
            .await
            .expect("improve");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        // Empty tone cue should not add the directive
        assert!(!system.contains("Tone/style directive"));
    }

    #[tokio::test]
    async fn generate_reply_retry_on_too_long_then_succeeds() {
        let long = "a ".repeat(200); // 400 chars
        let short = "Short reply.".to_string();
        let provider = MockProvider::new(vec![long, short]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_reply("Test tweet", "user", false)
            .await
            .expect("reply");
        assert_eq!(output.text, "Short reply.");
    }

    #[tokio::test]
    async fn generate_tweet_retry_on_too_long_then_succeeds() {
        let long = "b ".repeat(200);
        let short = "Concise tweet.".to_string();
        let provider = MockProvider::new(vec![long, short]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen.generate_tweet("testing").await.expect("tweet");
        assert_eq!(output.text, "Concise tweet.");
    }

    #[tokio::test]
    async fn generate_reply_product_url_in_prompt() {
        let (provider, captured) = PromptCapturingProvider::new("Reply mentioning product.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        gen.generate_reply("test", "user", true)
            .await
            .expect("reply");
        let system = captured.lock().await;
        let system = system.as_ref().unwrap();
        assert!(system.contains("https://testapp.com"));
        assert!(system.contains("TestApp"));
    }

    // --- extract_highlights tests ---

    #[tokio::test]
    async fn extract_highlights_parses_dash_bullets() {
        let provider = MockProvider::single("- Insight one\n- Insight two\n- Insight three");
        let gen = ContentGenerator::new(Box::new(provider), test_business());
        let result = gen.extract_highlights("some context").await.unwrap();
        assert_eq!(result, vec!["Insight one", "Insight two", "Insight three"]);
    }

    #[tokio::test]
    async fn extract_highlights_parses_asterisk_bullets() {
        let provider = MockProvider::single("* First point\n* Second point");
        let gen = ContentGenerator::new(Box::new(provider), test_business());
        let result = gen.extract_highlights("some context").await.unwrap();
        assert_eq!(result, vec!["First point", "Second point"]);
    }

    #[tokio::test]
    async fn extract_highlights_parses_numbered_bullets() {
        let provider = MockProvider::single("1. First\n2. Second\n3. Third");
        let gen = ContentGenerator::new(Box::new(provider), test_business());
        let result = gen.extract_highlights("some context").await.unwrap();
        assert_eq!(result, vec!["First", "Second", "Third"]);
    }

    #[tokio::test]
    async fn extract_highlights_filters_empty_lines() {
        let provider = MockProvider::single("- One\n\n- Two\n\n");
        let gen = ContentGenerator::new(Box::new(provider), test_business());
        let result = gen.extract_highlights("some context").await.unwrap();
        assert_eq!(result, vec!["One", "Two"]);
    }

    #[tokio::test]
    async fn extract_highlights_errors_on_empty_response() {
        let provider = MockProvider::single("");
        let gen = ContentGenerator::new(Box::new(provider), test_business());
        let result = gen.extract_highlights("some context").await;
        assert!(result.is_err());
    }

    // --- parse_hooks_response tests ---

    #[test]
    fn parse_hooks_response_empty() {
        let results = super::super::parser::parse_hooks_response("");
        assert!(results.is_empty());
    }

    #[test]
    fn parse_hooks_response_single() {
        let text = "STYLE: question\nHOOK: What if testing was actually fun?";
        let results = super::super::parser::parse_hooks_response(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "question");
        assert_eq!(results[0].1, "What if testing was actually fun?");
    }

    #[test]
    fn parse_hooks_response_multiple() {
        let text = "\
STYLE: question
HOOK: What if testing was actually fun?
---
STYLE: contrarian_take
HOOK: Most devs test too much. Here's why.
---
STYLE: tip
HOOK: One command that saves me 2 hours a week.";
        let results = super::super::parser::parse_hooks_response(text);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, "question");
        assert_eq!(results[1].0, "contrarian_take");
        assert_eq!(results[2].0, "tip");
    }

    #[test]
    fn parse_hooks_response_missing_style_defaults_to_general() {
        let text = "HOOK: A standalone hook without style tag";
        let results = super::super::parser::parse_hooks_response(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "general");
        assert_eq!(results[0].1, "A standalone hook without style tag");
    }

    #[test]
    fn parse_hooks_response_trailing_separator() {
        let text = "STYLE: tip\nHOOK: First hook\n---\nSTYLE: list\nHOOK: Second hook\n---";
        let results = super::super::parser::parse_hooks_response(text);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn parse_hooks_response_empty_hook_skipped() {
        let text = "STYLE: tip\n---\nSTYLE: question\nHOOK: Real hook here";
        let results = super::super::parser::parse_hooks_response(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "question");
    }

    // --- generate_hooks tests ---

    #[tokio::test]
    async fn generate_hooks_returns_hooks() {
        let response = "\
STYLE: question
HOOK: What if your tests could write themselves?
---
STYLE: contrarian_take
HOOK: Most devs test too much. Here's why that hurts your product.
---
STYLE: tip
HOOK: One cargo command saves me 2 hours a week: cargo nextest.
---
STYLE: list
HOOK: 3 testing mistakes costing you time: 1) mocking too much 2) no integration tests 3) ignoring flaky tests
---
STYLE: storytelling
HOOK: Last week I shipped a bug to production. The fix? A 3-line test I should have written first.";

        let provider = MockProvider::single(response);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen
            .generate_hooks("testing best practices", None)
            .await
            .expect("hooks");
        assert!(
            (3..=5).contains(&output.hooks.len()),
            "expected 3-5 hooks, got {}",
            output.hooks.len()
        );
        for hook in &output.hooks {
            assert!(!hook.style.is_empty());
            assert!(!hook.text.is_empty());
            assert!(hook.char_count <= MAX_TWEET_CHARS);
            assert!(hook.confidence == "high" || hook.confidence == "medium");
        }
        assert_eq!(output.provider, "mock");
    }

    #[tokio::test]
    async fn generate_hooks_with_rag_context() {
        let response = "STYLE: question\nHOOK: Short hook here";
        let (provider, captured) = PromptCapturingProvider::new(response);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let rag_block = "Winning patterns:\n1. [tip] (tweet): \"Great advice\"";
        gen.generate_hooks("testing", Some(rag_block))
            .await
            .expect("hooks");

        let system = captured.lock().await;
        let system = system.as_ref().expect("system prompt captured");
        assert!(
            system.contains("Winning patterns"),
            "RAG context should appear in system prompt"
        );
    }

    #[tokio::test]
    async fn generate_hooks_filters_oversized() {
        let long_hook = "x".repeat(300);
        let response =
            format!("STYLE: question\nHOOK: {long_hook}\n---\nSTYLE: tip\nHOOK: Short and sweet.");
        let provider = MockProvider::single(&response);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen.generate_hooks("topic", None).await.expect("hooks");
        assert_eq!(output.hooks.len(), 1, "oversized hook should be filtered");
        assert_eq!(output.hooks[0].text, "Short and sweet.");
    }

    #[tokio::test]
    async fn generate_hooks_retries_when_too_few() {
        let bad = "STYLE: tip\nHOOK: Only one";
        let good = "\
STYLE: question\nHOOK: Hook one?\n---\n\
STYLE: contrarian_take\nHOOK: Hook two.\n---\n\
STYLE: tip\nHOOK: Hook three.";
        let provider = MockProvider::new(vec![bad.into(), good.into()]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let output = gen.generate_hooks("topic", None).await.expect("hooks");
        assert!(
            output.hooks.len() >= 3,
            "expected >= 3 after retry, got {}",
            output.hooks.len()
        );
    }

    #[tokio::test]
    async fn generate_hooks_fails_on_empty() {
        let provider = MockProvider::new(vec!["".into(), "".into()]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let result = gen.generate_hooks("topic", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn generate_hooks_prompt_includes_five_styles() {
        let response = "STYLE: question\nHOOK: Test hook";
        let (provider, captured) = PromptCapturingProvider::new(response);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        // Will likely fail (only 1 hook) but we just need to inspect the prompt
        let _ = gen.generate_hooks("testing", None).await;

        let system = captured.lock().await;
        let system = system.as_ref().expect("system prompt captured");
        assert!(
            system.contains("question"),
            "prompt should include 'question' style"
        );
        assert!(
            system.contains("contrarian_take"),
            "prompt should include 'contrarian_take' style"
        );
        assert!(
            system.contains("Generate exactly 5 hook tweets"),
            "prompt should ask for 5 hooks"
        );
    }

    #[test]
    fn hook_option_serialization() {
        let hook = super::super::HookOption {
            style: "question".to_string(),
            text: "Is testing overrated?".to_string(),
            char_count: 20,
            confidence: "high".to_string(),
        };
        let json = serde_json::to_string(&hook).expect("serialize");
        assert!(json.contains("\"style\":\"question\""));
        assert!(json.contains("\"confidence\":\"high\""));

        let hook2: super::super::HookOption = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(hook2.style, "question");
        assert_eq!(hook2.text, "Is testing overrated?");
    }

    #[test]
    fn hook_option_confidence_heuristic() {
        let short_hooks = super::super::ContentGenerator::build_hook_options(&[(
            "tip".to_string(),
            "Short hook".to_string(),
        )]);
        assert_eq!(short_hooks[0].confidence, "high");

        let long_text = "x".repeat(250);
        let long_hooks =
            super::super::ContentGenerator::build_hook_options(&[("tip".to_string(), long_text)]);
        assert_eq!(long_hooks[0].confidence, "medium");
    }

    #[test]
    fn hook_generation_output_debug_and_clone() {
        let output = super::super::HookGenerationOutput {
            hooks: vec![super::super::HookOption {
                style: "question".to_string(),
                text: "Test?".to_string(),
                char_count: 5,
                confidence: "high".to_string(),
            }],
            usage: TokenUsage::default(),
            model: "gpt-4".to_string(),
            provider: "openai".to_string(),
        };
        let clone = output.clone();
        assert_eq!(clone.hooks.len(), 1);
        let debug = format!("{output:?}");
        assert!(debug.contains("question"));
    }
}
