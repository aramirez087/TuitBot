//! High-level content generation combining LLM providers with business context.
//!
//! Produces replies, tweets, and threads that meet X's format requirements
//! (280 characters per tweet, 5-8 tweets per thread) with retry logic.

use crate::config::BusinessProfile;
use crate::content::frameworks::{ReplyArchetype, ThreadStructure, TweetFormat};
use crate::error::LlmError;
use crate::llm::{GenerationParams, LlmProvider};

/// Maximum characters allowed in a single tweet.
const MAX_TWEET_CHARS: usize = 280;

/// Maximum retries for thread generation.
const MAX_THREAD_RETRIES: u32 = 2;

/// Content generator that combines an LLM provider with business context.
pub struct ContentGenerator {
    provider: Box<dyn LlmProvider>,
    business: BusinessProfile,
}

impl ContentGenerator {
    /// Create a new content generator.
    pub fn new(provider: Box<dyn LlmProvider>, business: BusinessProfile) -> Self {
        Self { provider, business }
    }

    /// Generate a reply to a tweet.
    ///
    /// The reply will be conversational, helpful, and under 280 characters.
    /// When `mention_product` is false, the system prompt explicitly forbids
    /// mentioning the product name.
    /// When `archetype` is provided, the prompt includes archetype-specific
    /// guidance for varied output (e.g., ask a question, share experience).
    /// Retries once with a stricter prompt if the first attempt is too long,
    /// then truncates at a sentence boundary as a last resort.
    pub async fn generate_reply(
        &self,
        tweet_text: &str,
        tweet_author: &str,
        mention_product: bool,
    ) -> Result<String, LlmError> {
        self.generate_reply_with_archetype(tweet_text, tweet_author, mention_product, None)
            .await
    }

    /// Generate a reply using a specific archetype for varied output.
    pub async fn generate_reply_with_archetype(
        &self,
        tweet_text: &str,
        tweet_author: &str,
        mention_product: bool,
        archetype: Option<ReplyArchetype>,
    ) -> Result<String, LlmError> {
        tracing::debug!(
            author = %tweet_author,
            archetype = ?archetype,
            mention_product = mention_product,
            "Generating reply",
        );
        let voice_section = match &self.business.brand_voice {
            Some(v) if !v.is_empty() => format!("\nVoice & personality: {v}"),
            _ => String::new(),
        };
        let reply_section = match &self.business.reply_style {
            Some(s) if !s.is_empty() => format!("\nReply style: {s}"),
            _ => "\nReply style: Be conversational and helpful, not salesy. Sound like a real person, not a bot.".to_string(),
        };

        let archetype_section = match archetype {
            Some(a) => format!("\n{}", a.prompt_fragment()),
            None => String::new(),
        };

        let persona_section = self.format_persona_context();

        let product_rule = if mention_product {
            let product_url = self.business.product_url.as_deref().unwrap_or("");
            format!(
                "You are a helpful community member who uses {} ({}).\n\
                 Your target audience is: {}.\n\
                 Product URL: {}\
                 {voice_section}\
                 {reply_section}\
                 {archetype_section}\
                 {persona_section}\n\n\
                 Rules:\n\
                 - Write a reply to the tweet below.\n\
                 - Maximum 3 sentences.\n\
                 - Only mention {} if it is genuinely relevant to the tweet's topic.\n\
                 - Do not use hashtags.\n\
                 - Do not use emojis excessively.",
                self.business.product_name,
                self.business.product_description,
                self.business.target_audience,
                product_url,
                self.business.product_name,
            )
        } else {
            format!(
                "You are a helpful community member.\n\
                 Your target audience is: {}.\
                 {voice_section}\
                 {reply_section}\
                 {archetype_section}\
                 {persona_section}\n\n\
                 Rules:\n\
                 - Write a reply to the tweet below.\n\
                 - Maximum 3 sentences.\n\
                 - Do NOT mention {} or any product. Just be genuinely helpful.\n\
                 - Do not use hashtags.\n\
                 - Do not use emojis excessively.",
                self.business.target_audience, self.business.product_name,
            )
        };

        let system = product_rule;
        let user_message = format!("Tweet by @{tweet_author}: {tweet_text}");

        let params = GenerationParams {
            max_tokens: 200,
            temperature: 0.7,
            ..Default::default()
        };

        let resp = self
            .provider
            .complete(&system, &user_message, &params)
            .await?;
        let text = resp.text.trim().to_string();

        tracing::debug!(chars = text.len(), "Generated reply");

        if validate_length(&text, MAX_TWEET_CHARS) {
            return Ok(text);
        }

        // Retry with stricter instruction
        let retry_msg = format!(
            "{user_message}\n\nImportant: Your reply MUST be under 280 characters. Be more concise."
        );
        let resp = self.provider.complete(&system, &retry_msg, &params).await?;
        let text = resp.text.trim().to_string();

        if validate_length(&text, MAX_TWEET_CHARS) {
            return Ok(text);
        }

        // Last resort: truncate at sentence boundary
        Ok(truncate_at_sentence(&text, MAX_TWEET_CHARS))
    }

    /// Generate a standalone educational tweet.
    ///
    /// The tweet will be informative, engaging, and under 280 characters.
    pub async fn generate_tweet(&self, topic: &str) -> Result<String, LlmError> {
        self.generate_tweet_with_format(topic, None).await
    }

    /// Generate a tweet using a specific format for varied structure.
    pub async fn generate_tweet_with_format(
        &self,
        topic: &str,
        format: Option<TweetFormat>,
    ) -> Result<String, LlmError> {
        tracing::debug!(
            topic = %topic,
            format = ?format,
            "Generating tweet",
        );
        let voice_section = match &self.business.brand_voice {
            Some(v) if !v.is_empty() => format!("\nVoice & personality: {v}"),
            _ => String::new(),
        };
        let content_section = match &self.business.content_style {
            Some(s) if !s.is_empty() => format!("\nContent style: {s}"),
            _ => "\nContent style: Be informative and engaging.".to_string(),
        };

        let format_section = match format {
            Some(f) => format!("\n{}", f.prompt_fragment()),
            None => String::new(),
        };

        let persona_section = self.format_persona_context();

        let system = format!(
            "You are {}'s social media voice. {}.\n\
             Your audience: {}.\
             {voice_section}\
             {content_section}\
             {format_section}\
             {persona_section}\n\n\
             Rules:\n\
             - Write a single educational tweet about the topic below.\n\
             - Maximum 280 characters.\n\
             - Do not use hashtags.\n\
             - Do not mention {} directly unless it is central to the topic.",
            self.business.product_name,
            self.business.product_description,
            self.business.target_audience,
            self.business.product_name,
        );

        let user_message = format!("Write a tweet about: {topic}");

        let params = GenerationParams {
            max_tokens: 150,
            temperature: 0.8,
            ..Default::default()
        };

        let resp = self
            .provider
            .complete(&system, &user_message, &params)
            .await?;
        let text = resp.text.trim().to_string();

        if validate_length(&text, MAX_TWEET_CHARS) {
            return Ok(text);
        }

        // Retry with stricter instruction
        let retry_msg = format!(
            "{user_message}\n\nImportant: Your tweet MUST be under 280 characters. Be more concise."
        );
        let resp = self.provider.complete(&system, &retry_msg, &params).await?;
        let text = resp.text.trim().to_string();

        if validate_length(&text, MAX_TWEET_CHARS) {
            return Ok(text);
        }

        Ok(truncate_at_sentence(&text, MAX_TWEET_CHARS))
    }

    /// Generate an educational thread of 5-8 tweets.
    ///
    /// Each tweet in the thread will be under 280 characters.
    /// Retries up to 2 times if the LLM produces malformed output.
    pub async fn generate_thread(&self, topic: &str) -> Result<Vec<String>, LlmError> {
        self.generate_thread_with_structure(topic, None).await
    }

    /// Generate a thread using a specific structure for varied content.
    pub async fn generate_thread_with_structure(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
    ) -> Result<Vec<String>, LlmError> {
        tracing::debug!(
            topic = %topic,
            structure = ?structure,
            "Generating thread",
        );
        let voice_section = match &self.business.brand_voice {
            Some(v) if !v.is_empty() => format!("\nVoice & personality: {v}"),
            _ => String::new(),
        };
        let content_section = match &self.business.content_style {
            Some(s) if !s.is_empty() => format!("\nContent style: {s}"),
            _ => "\nContent style: Be informative, not promotional.".to_string(),
        };

        let structure_section = match structure {
            Some(s) => format!("\n{}", s.prompt_fragment()),
            None => String::new(),
        };

        let persona_section = self.format_persona_context();

        let system = format!(
            "You are {}'s social media voice. {}.\n\
             Your audience: {}.\
             {voice_section}\
             {content_section}\
             {structure_section}\
             {persona_section}\n\n\
             Rules:\n\
             - Write an educational thread of 5 to 8 tweets about the topic below.\n\
             - Separate each tweet with a line containing only \"---\".\n\
             - Each tweet must be under 280 characters.\n\
             - The first tweet should hook the reader.\n\
             - The last tweet should include a call to action or summary.\n\
             - Do not use hashtags.",
            self.business.product_name,
            self.business.product_description,
            self.business.target_audience,
        );

        let user_message = format!("Write a thread about: {topic}");

        let params = GenerationParams {
            max_tokens: 1500,
            temperature: 0.7,
            ..Default::default()
        };

        for attempt in 0..=MAX_THREAD_RETRIES {
            let msg = if attempt == 0 {
                user_message.clone()
            } else {
                format!(
                    "{user_message}\n\nIMPORTANT: Write exactly 5-8 tweets, \
                     each under 280 characters, separated by lines containing only \"---\"."
                )
            };

            let resp = self.provider.complete(&system, &msg, &params).await?;
            let tweets = parse_thread(&resp.text);

            if (5..=8).contains(&tweets.len())
                && tweets.iter().all(|t| validate_length(t, MAX_TWEET_CHARS))
            {
                return Ok(tweets);
            }
        }

        Err(LlmError::GenerationFailed(
            "Failed to generate valid thread after retries".to_string(),
        ))
    }

    /// Build a persona context section from opinions and experiences.
    fn format_persona_context(&self) -> String {
        let mut parts = Vec::new();

        if !self.business.persona_opinions.is_empty() {
            let opinions = self.business.persona_opinions.join("; ");
            parts.push(format!("Opinions you hold: {opinions}"));
        }

        if !self.business.persona_experiences.is_empty() {
            let experiences = self.business.persona_experiences.join("; ");
            parts.push(format!("Experiences you can reference: {experiences}"));
        }

        if !self.business.content_pillars.is_empty() {
            let pillars = self.business.content_pillars.join(", ");
            parts.push(format!("Content pillars: {pillars}"));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("\n{}", parts.join("\n"))
        }
    }
}

/// Parse a thread response by splitting on `---` delimiters.
///
/// Also tries numbered patterns (e.g., "1/8", "1.") as a fallback.
fn parse_thread(text: &str) -> Vec<String> {
    // Primary: split on "---" delimiter
    let tweets: Vec<String> = text
        .split("---")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if !tweets.is_empty() && text.contains("---") {
        return tweets;
    }

    // Fallback: try splitting on numbered patterns like "1/8", "2/8" or "1.", "2."
    let lines: Vec<&str> = text.lines().collect();
    let mut tweets = Vec::new();
    let mut current = String::new();

    for line in &lines {
        let trimmed = line.trim();
        let is_numbered = trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
            && (trimmed.contains('/')
                || trimmed.starts_with(|c: char| c.is_ascii_digit())
                    && trimmed.chars().nth(1).is_some_and(|c| c == '.' || c == ')'));

        if is_numbered && !current.is_empty() {
            tweets.push(current.trim().to_string());
            current = String::new();
        }

        if !trimmed.is_empty() {
            if !current.is_empty() {
                current.push(' ');
            }
            // Strip the number prefix if present
            if is_numbered {
                let content = trimmed
                    .find(|c: char| !c.is_ascii_digit() && c != '/' && c != '.' && c != ')')
                    .map(|i| trimmed[i..].trim_start())
                    .unwrap_or(trimmed);
                current.push_str(content);
            } else {
                current.push_str(trimmed);
            }
        }
    }

    if !current.trim().is_empty() {
        tweets.push(current.trim().to_string());
    }

    tweets
}

/// Check if text is within the character limit.
fn validate_length(text: &str, max_chars: usize) -> bool {
    text.len() <= max_chars
}

/// Truncate text at the last sentence boundary that fits within the limit.
///
/// Looks for the last period, exclamation mark, or question mark within the limit.
/// Falls back to truncating at the limit with "..." if no sentence boundary is found.
fn truncate_at_sentence(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let search_area = &text[..max_chars];

    // Find the last sentence-ending punctuation
    let last_sentence_end = search_area
        .rfind('.')
        .max(search_area.rfind('!'))
        .max(search_area.rfind('?'));

    if let Some(pos) = last_sentence_end {
        if pos > 0 {
            return text[..=pos].trim().to_string();
        }
    }

    // No sentence boundary found; hard truncate with ellipsis
    let truncate_at = max_chars.saturating_sub(3);
    // Find a word boundary
    let word_end = search_area[..truncate_at].rfind(' ').unwrap_or(truncate_at);
    format!("{}...", &text[..word_end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{LlmResponse, TokenUsage};
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

    fn test_business() -> BusinessProfile {
        BusinessProfile {
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

    // --- validate_length tests ---

    #[test]
    fn validate_length_under_limit() {
        assert!(validate_length("short text", 280));
    }

    #[test]
    fn validate_length_at_limit() {
        let text = "a".repeat(280);
        assert!(validate_length(&text, 280));
    }

    #[test]
    fn validate_length_over_limit() {
        let text = "a".repeat(281);
        assert!(!validate_length(&text, 280));
    }

    // --- truncate_at_sentence tests ---

    #[test]
    fn truncate_at_sentence_under_limit() {
        let text = "Short sentence.";
        assert_eq!(truncate_at_sentence(text, 280), "Short sentence.");
    }

    #[test]
    fn truncate_at_period() {
        let text = "First sentence. Second sentence. Third sentence is very long and goes over the limit and more and more text.";
        let result = truncate_at_sentence(text, 50);
        assert!(result.len() <= 50);
        assert!(result.ends_with('.'));
    }

    #[test]
    fn truncate_at_question_mark() {
        let text = "Is this working? I hope so because this text is getting very long and will exceed the character limit.";
        let result = truncate_at_sentence(text, 20);
        assert!(result.len() <= 20);
        assert!(result.ends_with('?'));
    }

    #[test]
    fn truncate_no_sentence_boundary() {
        let text =
            "This is a very long sentence without any punctuation that keeps going and going";
        let result = truncate_at_sentence(text, 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("..."));
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

        let reply = gen
            .generate_reply("Testing is important", "devuser", true)
            .await
            .expect("reply");
        assert!(reply.len() <= MAX_TWEET_CHARS);
        assert!(!reply.is_empty());
    }

    #[tokio::test]
    async fn generate_reply_truncates_long_output() {
        let long_text = "a ".repeat(200); // 400 chars
        let provider = MockProvider::new(vec![long_text.clone(), long_text]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let reply = gen
            .generate_reply("test", "user", true)
            .await
            .expect("reply");
        assert!(reply.len() <= MAX_TWEET_CHARS);
    }

    #[tokio::test]
    async fn generate_reply_no_product_mention() {
        let provider = MockProvider::single("That's a great approach for productivity!");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let reply = gen
            .generate_reply("How do you stay productive?", "devuser", false)
            .await
            .expect("reply");
        assert!(reply.len() <= MAX_TWEET_CHARS);
        assert!(!reply.is_empty());
    }

    // --- generate_tweet tests ---

    #[tokio::test]
    async fn generate_tweet_success() {
        let provider =
            MockProvider::single("Testing your code early saves hours of debugging later.");
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let tweet = gen
            .generate_tweet("testing best practices")
            .await
            .expect("tweet");
        assert!(tweet.len() <= MAX_TWEET_CHARS);
        assert!(!tweet.is_empty());
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

        let thread = gen.generate_thread("testing").await.expect("thread");
        assert!(
            (5..=8).contains(&thread.len()),
            "got {} tweets",
            thread.len()
        );
        for tweet in &thread {
            assert!(tweet.len() <= MAX_TWEET_CHARS);
        }
    }

    #[tokio::test]
    async fn generate_thread_retries_on_bad_count() {
        // First attempt: too few tweets. Second: still too few. Third: valid.
        let bad = "Tweet one\n---\nTweet two";
        let good = "One\n---\nTwo\n---\nThree\n---\nFour\n---\nFive";
        let provider = MockProvider::new(vec![bad.into(), bad.into(), good.into()]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let thread = gen.generate_thread("topic").await.expect("thread");
        assert_eq!(thread.len(), 5);
    }

    #[tokio::test]
    async fn generate_thread_fails_after_max_retries() {
        let bad = "Tweet one\n---\nTweet two";
        let provider = MockProvider::new(vec![bad.into(), bad.into(), bad.into()]);
        let gen = ContentGenerator::new(Box::new(provider), test_business());

        let err = gen.generate_thread("topic").await.unwrap_err();
        assert!(matches!(err, LlmError::GenerationFailed(_)));
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
