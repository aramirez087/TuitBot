//! High-level content generation combining LLM providers with business context.
//!
//! Produces replies, tweets, and threads that meet X's format requirements
//! (280 characters per tweet, 5-8 tweets per thread) with retry logic.

pub(crate) mod parser;

#[cfg(test)]
mod tests;

use crate::config::BusinessProfile;
use crate::content::frameworks::{ReplyArchetype, ThreadStructure, TweetFormat};
use crate::content::length::{truncate_at_sentence, validate_tweet_length, MAX_TWEET_CHARS};
use crate::error::LlmError;
use crate::llm::{GenerationParams, LlmProvider, TokenUsage};

use parser::parse_thread;

/// Output from a single-text generation (reply or tweet).
#[derive(Debug, Clone)]
pub struct GenerationOutput {
    /// The generated text.
    pub text: String,
    /// Accumulated token usage across all attempts (including retries).
    pub usage: TokenUsage,
    /// The model that produced the final response.
    pub model: String,
    /// The provider name (e.g., "openai", "anthropic", "ollama").
    pub provider: String,
}

/// Output from thread generation.
#[derive(Debug, Clone)]
pub struct ThreadGenerationOutput {
    /// The generated tweets in thread order.
    pub tweets: Vec<String>,
    /// Accumulated token usage across all attempts (including retries).
    pub usage: TokenUsage,
    /// The model that produced the final response.
    pub model: String,
    /// The provider name.
    pub provider: String,
}

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

    // -----------------------------------------------------------------
    // Reply generation
    // -----------------------------------------------------------------

    /// Generate a reply to a tweet.
    pub async fn generate_reply(
        &self,
        tweet_text: &str,
        tweet_author: &str,
        mention_product: bool,
    ) -> Result<GenerationOutput, LlmError> {
        self.generate_reply_inner(tweet_text, tweet_author, mention_product, None, None)
            .await
    }

    /// Generate a reply using a specific archetype for varied output.
    pub async fn generate_reply_with_archetype(
        &self,
        tweet_text: &str,
        tweet_author: &str,
        mention_product: bool,
        archetype: Option<ReplyArchetype>,
    ) -> Result<GenerationOutput, LlmError> {
        self.generate_reply_inner(tweet_text, tweet_author, mention_product, archetype, None)
            .await
    }

    /// Generate a reply with optional RAG context injected into the prompt.
    pub async fn generate_reply_with_context(
        &self,
        tweet_text: &str,
        tweet_author: &str,
        mention_product: bool,
        archetype: Option<ReplyArchetype>,
        rag_context: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        self.generate_reply_inner(
            tweet_text,
            tweet_author,
            mention_product,
            archetype,
            rag_context,
        )
        .await
    }

    /// Internal reply generation with optional archetype and RAG context.
    async fn generate_reply_inner(
        &self,
        tweet_text: &str,
        tweet_author: &str,
        mention_product: bool,
        archetype: Option<ReplyArchetype>,
        rag_context: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        tracing::debug!(
            author = %tweet_author,
            archetype = ?archetype,
            mention_product = mention_product,
            has_rag_context = rag_context.is_some(),
            "Generating reply",
        );

        let voice_section = self.format_voice_section();
        let reply_section = match &self.business.reply_style {
            Some(s) if !s.is_empty() => format!("\nReply style: {s}"),
            _ => "\nReply style: Be conversational and helpful, not salesy. Sound like a real person, not a bot.".to_string(),
        };
        let archetype_section = match archetype {
            Some(a) => format!("\n{}", a.prompt_fragment()),
            None => String::new(),
        };
        let persona_section = self.format_persona_context();
        let rag_section = Self::format_rag_section(rag_context);
        let audience_section = self.format_audience_section();

        let system = if mention_product {
            let product_url = self.business.product_url.as_deref().unwrap_or("");
            format!(
                "You are a helpful community member who uses {} ({}).\
                 {audience_section}\n\
                 Product URL: {}\
                 {voice_section}\
                 {reply_section}\
                 {archetype_section}\
                 {persona_section}\
                 {rag_section}\n\n\
                 Rules:\n\
                 - Write a reply to the tweet below.\n\
                 - Maximum 3 sentences.\n\
                 - Only mention {} if it is genuinely relevant to the tweet's topic.\n\
                 - Do not use hashtags.\n\
                 - Do not use emojis excessively.",
                self.business.product_name,
                self.business.product_description,
                product_url,
                self.business.product_name,
            )
        } else {
            format!(
                "You are a helpful community member.\
                 {audience_section}\
                 {voice_section}\
                 {reply_section}\
                 {archetype_section}\
                 {persona_section}\
                 {rag_section}\n\n\
                 Rules:\n\
                 - Write a reply to the tweet below.\n\
                 - Maximum 3 sentences.\n\
                 - Do NOT mention {} or any product. Just be genuinely helpful.\n\
                 - Do not use hashtags.\n\
                 - Do not use emojis excessively.",
                self.business.product_name,
            )
        };

        let user_message = format!("Tweet by @{tweet_author}: {tweet_text}");
        let params = GenerationParams {
            max_tokens: 200,
            temperature: 0.7,
            ..Default::default()
        };

        self.generate_single(&system, &user_message, &params).await
    }

    // -----------------------------------------------------------------
    // Tweet generation
    // -----------------------------------------------------------------

    /// Generate a standalone educational tweet.
    pub async fn generate_tweet(&self, topic: &str) -> Result<GenerationOutput, LlmError> {
        self.generate_tweet_inner(topic, None, None).await
    }

    /// Generate a tweet using a specific format for varied structure.
    pub async fn generate_tweet_with_format(
        &self,
        topic: &str,
        format: Option<TweetFormat>,
    ) -> Result<GenerationOutput, LlmError> {
        self.generate_tweet_inner(topic, format, None).await
    }

    /// Generate a tweet with optional RAG context injected into the prompt.
    pub async fn generate_tweet_with_context(
        &self,
        topic: &str,
        format: Option<TweetFormat>,
        rag_context: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        self.generate_tweet_inner(topic, format, rag_context).await
    }

    /// Internal tweet generation with optional format and RAG context.
    async fn generate_tweet_inner(
        &self,
        topic: &str,
        format: Option<TweetFormat>,
        rag_context: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        tracing::debug!(
            topic = %topic,
            format = ?format,
            has_rag_context = rag_context.is_some(),
            "Generating tweet",
        );

        let voice_section = self.format_voice_section();
        let content_section = match &self.business.content_style {
            Some(s) if !s.is_empty() => format!("\nContent style: {s}"),
            _ => "\nContent style: Be informative and engaging.".to_string(),
        };
        let format_section = match format {
            Some(f) => format!("\n{}", f.prompt_fragment()),
            None => String::new(),
        };
        let persona_section = self.format_persona_context();
        let rag_section = Self::format_rag_section(rag_context);
        let audience_section = self.format_audience_section();

        let system = format!(
            "You are {}'s social media voice. {}.\
             {audience_section}\
             {voice_section}\
             {content_section}\
             {format_section}\
             {persona_section}\
             {rag_section}\n\n\
             Rules:\n\
             - Write a single educational tweet about the topic below.\n\
             - Maximum 280 characters.\n\
             - Do not use hashtags.\n\
             - Do not mention {} directly unless it is central to the topic.",
            self.business.product_name,
            self.business.product_description,
            self.business.product_name,
        );

        let user_message = format!("Write a tweet about: {topic}");
        let params = GenerationParams {
            max_tokens: 150,
            temperature: 0.8,
            ..Default::default()
        };

        self.generate_single(&system, &user_message, &params).await
    }

    // -----------------------------------------------------------------
    // Thread generation
    // -----------------------------------------------------------------

    /// Generate an educational thread of 5-8 tweets.
    pub async fn generate_thread(&self, topic: &str) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, None, None).await
    }

    /// Generate a thread using a specific structure for varied content.
    pub async fn generate_thread_with_structure(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, structure, None).await
    }

    /// Generate a thread with optional RAG context injected into the prompt.
    pub async fn generate_thread_with_context(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
        rag_context: Option<&str>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, structure, rag_context)
            .await
    }

    /// Internal thread generation with optional structure and RAG context.
    async fn generate_thread_inner(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
        rag_context: Option<&str>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        tracing::debug!(
            topic = %topic,
            structure = ?structure,
            has_rag_context = rag_context.is_some(),
            "Generating thread",
        );

        let voice_section = self.format_voice_section();
        let content_section = match &self.business.content_style {
            Some(s) if !s.is_empty() => format!("\nContent style: {s}"),
            _ => "\nContent style: Be informative, not promotional.".to_string(),
        };
        let structure_section = match structure {
            Some(s) => format!("\n{}", s.prompt_fragment()),
            None => String::new(),
        };
        let persona_section = self.format_persona_context();
        let rag_section = Self::format_rag_section(rag_context);
        let audience_section = self.format_audience_section();

        let system = format!(
            "You are {}'s social media voice. {}.\
             {audience_section}\
             {voice_section}\
             {content_section}\
             {structure_section}\
             {persona_section}\
             {rag_section}\n\n\
             Rules:\n\
             - Write an educational thread of 5 to 8 tweets about the topic below.\n\
             - Separate each tweet with a line containing only \"---\".\n\
             - Each tweet must be under 280 characters.\n\
             - The first tweet should hook the reader.\n\
             - The last tweet should include a call to action or summary.\n\
             - Do not use hashtags.",
            self.business.product_name, self.business.product_description,
        );

        let user_message = format!("Write a thread about: {topic}");
        let params = GenerationParams {
            max_tokens: 1500,
            temperature: 0.7,
            ..Default::default()
        };

        let mut usage = TokenUsage::default();
        let provider_name = self.provider.name().to_string();
        let mut model = String::new();

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
            usage.accumulate(&resp.usage);
            model.clone_from(&resp.model);
            let tweets = parse_thread(&resp.text);

            if (5..=8).contains(&tweets.len())
                && tweets
                    .iter()
                    .all(|t| validate_tweet_length(t, MAX_TWEET_CHARS))
            {
                return Ok(ThreadGenerationOutput {
                    tweets,
                    usage,
                    model,
                    provider: provider_name,
                });
            }
        }

        Err(LlmError::GenerationFailed(
            "Failed to generate valid thread after retries".to_string(),
        ))
    }

    // -----------------------------------------------------------------
    // Shared helpers
    // -----------------------------------------------------------------

    /// Generate a single tweet/reply with retry and truncation fallback.
    async fn generate_single(
        &self,
        system: &str,
        user_message: &str,
        params: &GenerationParams,
    ) -> Result<GenerationOutput, LlmError> {
        let resp = self.provider.complete(system, user_message, params).await?;
        let mut usage = resp.usage.clone();
        let provider_name = self.provider.name().to_string();
        let model = resp.model.clone();
        let text = resp.text.trim().to_string();

        tracing::debug!(chars = text.len(), "Generated content");

        if validate_tweet_length(&text, MAX_TWEET_CHARS) {
            return Ok(GenerationOutput {
                text,
                usage,
                model,
                provider: provider_name,
            });
        }

        // Retry with stricter instruction
        let retry_msg = format!(
            "{user_message}\n\nImportant: Your response MUST be under 280 characters. Be more concise."
        );
        let resp = self.provider.complete(system, &retry_msg, params).await?;
        usage.accumulate(&resp.usage);
        let text = resp.text.trim().to_string();

        if validate_tweet_length(&text, MAX_TWEET_CHARS) {
            return Ok(GenerationOutput {
                text,
                usage,
                model,
                provider: provider_name,
            });
        }

        // Last resort: truncate at sentence boundary
        Ok(GenerationOutput {
            text: truncate_at_sentence(&text, MAX_TWEET_CHARS),
            usage,
            model,
            provider: provider_name,
        })
    }

    fn format_voice_section(&self) -> String {
        match &self.business.brand_voice {
            Some(v) if !v.is_empty() => format!("\nVoice & personality: {v}"),
            _ => String::new(),
        }
    }

    fn format_audience_section(&self) -> String {
        if self.business.target_audience.is_empty() {
            String::new()
        } else {
            format!("\nYour audience: {}.", self.business.target_audience)
        }
    }

    fn format_rag_section(rag_context: Option<&str>) -> String {
        match rag_context {
            Some(ctx) if !ctx.is_empty() => format!("\n{ctx}"),
            _ => String::new(),
        }
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
