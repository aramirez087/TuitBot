//! High-level content generation combining LLM providers with business context.
//!
//! Produces replies, tweets, and threads that meet X's format requirements
//! (280 characters per tweet, 5-8 tweets per thread) with retry logic.

pub(crate) mod angles;
pub(crate) mod parser;

#[cfg(test)]
mod tests;

use crate::config::BusinessProfile;
use crate::content::frameworks::{ReplyArchetype, ThreadStructure, TweetFormat};
use crate::content::length::{truncate_at_sentence, validate_tweet_length, MAX_TWEET_CHARS};
use crate::error::LlmError;
use crate::llm::{GenerationParams, LlmProvider, TokenUsage};

use parser::{parse_hooks_response, parse_thread};

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

/// A single hook option returned by the hook generation pipeline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HookOption {
    /// The tweet format style (e.g., "question", "contrarian_take").
    pub style: String,
    /// The hook text (max 280 chars).
    pub text: String,
    /// Character count of the hook text.
    pub char_count: usize,
    /// Confidence heuristic: "high" if under 240 chars, "medium" otherwise.
    pub confidence: String,
}

/// Output from hook generation.
#[derive(Debug, Clone)]
pub struct HookGenerationOutput {
    /// The generated hook options (3–5).
    pub hooks: Vec<HookOption>,
    /// Accumulated token usage across all attempts.
    pub usage: TokenUsage,
    /// The model that produced the response.
    pub model: String,
    /// The provider name.
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

    /// Returns a reference to the business profile.
    pub fn business(&self) -> &BusinessProfile {
        &self.business
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
    // Draft improvement
    // -----------------------------------------------------------------

    /// Rewrite/improve an existing draft tweet with an optional tone cue.
    pub async fn improve_draft(
        &self,
        draft: &str,
        tone_cue: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        self.improve_draft_inner(draft, tone_cue, None).await
    }

    /// Rewrite/improve an existing draft tweet with optional RAG context
    /// injected into the system prompt.
    pub async fn improve_draft_with_context(
        &self,
        draft: &str,
        tone_cue: Option<&str>,
        rag_context: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        self.improve_draft_inner(draft, tone_cue, rag_context).await
    }

    /// Internal draft improvement with optional RAG context.
    async fn improve_draft_inner(
        &self,
        draft: &str,
        tone_cue: Option<&str>,
        rag_context: Option<&str>,
    ) -> Result<GenerationOutput, LlmError> {
        tracing::debug!(
            draft_len = draft.len(),
            tone_cue = ?tone_cue,
            has_rag_context = rag_context.is_some(),
            "Improving draft",
        );

        let voice_section = self.format_voice_section();
        let persona_section = self.format_persona_context();
        let rag_section = Self::format_rag_section(rag_context);

        let tone_instruction = match tone_cue {
            Some(cue) if !cue.is_empty() => {
                format!("\n\nTone/style directive (MUST follow): {cue}")
            }
            _ => String::new(),
        };

        let system = format!(
            "You are {}'s social media voice. {}.\
             {voice_section}\
             {persona_section}\
             {rag_section}\n\n\
             Task: Rewrite and improve the draft tweet below. \
             Keep the core message but make it sharper, more engaging, \
             and better-written.{tone_instruction}\n\n\
             Rules:\n\
             - Maximum 280 characters.\n\
             - Do not use hashtags.\n\
             - Output only the improved tweet text, nothing else.",
            self.business.product_name, self.business.product_description,
        );

        let user_message = format!("Draft to improve:\n{draft}");
        let params = GenerationParams {
            max_tokens: 150,
            temperature: 0.7,
            ..Default::default()
        };

        self.generate_single(&system, &user_message, &params).await
    }

    // -----------------------------------------------------------------
    // Hook generation (5 differentiated options)
    // -----------------------------------------------------------------

    /// Generate 5 differentiated hook options for the given topic.
    pub async fn generate_hooks(
        &self,
        topic: &str,
        rag_context: Option<&str>,
    ) -> Result<HookGenerationOutput, LlmError> {
        tracing::debug!(
            topic = %topic,
            has_rag_context = rag_context.is_some(),
            "Generating hooks",
        );

        let styles = Self::select_hook_styles();
        let style_list = styles
            .iter()
            .enumerate()
            .map(|(i, f)| format!("{}. {}", i + 1, f))
            .collect::<Vec<_>>()
            .join("\n");

        let voice_section = self.format_voice_section();
        let persona_section = self.format_persona_context();
        let rag_section = Self::format_rag_section(rag_context);
        let audience_section = self.format_audience_section();

        let system = format!(
            "You are {}'s social media voice. {}.\
             {audience_section}\
             {voice_section}\
             {persona_section}\
             {rag_section}\n\n\
             Task: Generate exactly 5 hook tweets for the topic below, \
             one per style listed. Each hook must be a standalone tweet \
             (max 280 characters) that grabs attention.\n\n\
             Required styles (one hook per style):\n{style_list}\n\n\
             Output format (strictly follow this, no extra text):\n\
             STYLE: <style_name>\n\
             HOOK: <hook text>\n\
             ---\n\
             (repeat for all 5)",
            self.business.product_name, self.business.product_description,
        );

        let user_message = format!("Generate hooks about: {topic}");
        let params = GenerationParams {
            max_tokens: 800,
            temperature: 0.9,
            ..Default::default()
        };

        let mut usage = TokenUsage::default();
        let provider_name = self.provider.name().to_string();

        let resp = self
            .provider
            .complete(&system, &user_message, &params)
            .await?;
        usage.accumulate(&resp.usage);
        let model = resp.model.clone();

        tracing::debug!(
            raw_response = %resp.text,
            "Raw LLM response for hook generation"
        );

        let mut hooks = Self::build_hook_options(&parse_hooks_response(&resp.text));

        // Retry once if fewer than 3 hooks
        if hooks.len() < 3 {
            tracing::debug!(count = hooks.len(), "Too few hooks, retrying");
            let retry_msg = format!(
                "{user_message}\n\nIMPORTANT: Output exactly 5 hooks, \
                 each with STYLE: and HOOK: lines, separated by ---."
            );
            let resp = self.provider.complete(&system, &retry_msg, &params).await?;
            usage.accumulate(&resp.usage);

            tracing::debug!(
                raw_response = %resp.text,
                "Raw LLM retry response for hook generation"
            );

            hooks = Self::build_hook_options(&parse_hooks_response(&resp.text));
        }

        if hooks.is_empty() {
            return Err(LlmError::GenerationFailed(
                "No valid hooks could be generated".to_string(),
            ));
        }

        // Truncate to 5 if the LLM returned more
        hooks.truncate(5);

        Ok(HookGenerationOutput {
            hooks,
            usage,
            model,
            provider: provider_name,
        })
    }

    /// Select 5 TweetFormat styles for hook generation.
    /// Always includes Question and ContrarianTake, plus 3 from the rest.
    fn select_hook_styles() -> Vec<TweetFormat> {
        use rand::seq::SliceRandom;

        let mut styles = vec![TweetFormat::Question, TweetFormat::ContrarianTake];
        let remaining = [
            TweetFormat::List,
            TweetFormat::MostPeopleThinkX,
            TweetFormat::Storytelling,
            TweetFormat::BeforeAfter,
            TweetFormat::Tip,
        ];
        let mut rng = rand::rng();
        let mut pool = remaining.to_vec();
        pool.shuffle(&mut rng);
        styles.extend(pool.into_iter().take(3));
        styles
    }

    /// Convert parsed (style, text) pairs into HookOption structs,
    /// filtering out any that exceed MAX_TWEET_CHARS.
    fn build_hook_options(parsed: &[(String, String)]) -> Vec<HookOption> {
        parsed
            .iter()
            .filter(|(_, text)| !text.is_empty() && text.len() <= MAX_TWEET_CHARS)
            .map(|(style, text)| {
                let char_count = text.len();
                let confidence = if char_count <= 240 {
                    "high".to_string()
                } else {
                    "medium".to_string()
                };
                HookOption {
                    style: style.clone(),
                    text: text.clone(),
                    char_count,
                    confidence,
                }
            })
            .collect()
    }

    // -----------------------------------------------------------------
    // Mined angle generation (Hook Miner)
    // -----------------------------------------------------------------

    /// Generate evidence-backed content angles from neighbor notes.
    pub async fn generate_mined_angles(
        &self,
        topic: &str,
        neighbors: &[crate::content::evidence::NeighborContent],
        selection_context: Option<&str>,
    ) -> Result<crate::content::angles::AngleMiningOutput, LlmError> {
        angles::generate_mined_angles(
            &*self.provider,
            &self.business,
            topic,
            neighbors,
            selection_context,
        )
        .await
    }

    // -----------------------------------------------------------------
    // Key highlights extraction
    // -----------------------------------------------------------------

    /// Extract 3-5 concise, tweetable key highlights from provided context.
    ///
    /// Used as an intermediate curation step before generating content
    /// from vault notes, letting the user review and select which
    /// insights to include.
    pub async fn extract_highlights(&self, rag_context: &str) -> Result<Vec<String>, LlmError> {
        tracing::debug!(context_len = rag_context.len(), "Extracting key highlights",);

        let system = format!(
            "You are {}'s content strategist. {}.\n\n\
             Task: Read the context below and extract 3 to 5 concise, \
             tweetable key insights as bullet points.\n\n\
             Rules:\n\
             - Each bullet should be a single clear insight or idea.\n\
             - Keep each bullet under 200 characters.\n\
             - Output only the bullet list, one per line.\n\
             - Use a dash (-) prefix for each bullet.\n\
             - No numbering, no sub-bullets, no headers.",
            self.business.product_name, self.business.product_description,
        );

        let user_message = format!("Context:\n{rag_context}");
        let params = GenerationParams {
            max_tokens: 500,
            temperature: 0.5,
            ..Default::default()
        };

        let resp = self
            .provider
            .complete(&system, &user_message, &params)
            .await?;

        tracing::debug!(
            raw_response = %resp.text,
            "Raw LLM response for highlight extraction"
        );

        let highlights: Vec<String> = resp
            .text
            .lines()
            .map(|line| strip_bullet_prefix(line.trim()))
            .filter(|s| !s.is_empty())
            .collect();

        if highlights.is_empty() {
            tracing::warn!(
                raw_response = %resp.text,
                "Highlight extraction produced no results after parsing"
            );
            return Err(LlmError::GenerationFailed(
                "No highlights could be extracted from the provided context".to_string(),
            ));
        }

        Ok(highlights)
    }

    // -----------------------------------------------------------------
    // Thread generation
    // -----------------------------------------------------------------

    /// Generate an educational thread of 5-8 tweets.
    pub async fn generate_thread(&self, topic: &str) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, None, None, None).await
    }

    /// Generate a thread using a specific structure for varied content.
    pub async fn generate_thread_with_structure(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, structure, None, None)
            .await
    }

    /// Generate a thread with optional RAG context injected into the prompt.
    pub async fn generate_thread_with_context(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
        rag_context: Option<&str>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, structure, rag_context, None)
            .await
    }

    /// Generate a thread that starts with a specific opening hook tweet.
    ///
    /// The hook is used verbatim as the first tweet, and the LLM generates
    /// 4-7 additional tweets continuing from that opening.
    pub async fn generate_thread_with_hook(
        &self,
        topic: &str,
        opening_hook: &str,
        structure: Option<ThreadStructure>,
        rag_context: Option<&str>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        self.generate_thread_inner(topic, structure, rag_context, Some(opening_hook))
            .await
    }

    /// Internal thread generation with optional structure, RAG context, and opening hook.
    async fn generate_thread_inner(
        &self,
        topic: &str,
        structure: Option<ThreadStructure>,
        rag_context: Option<&str>,
        opening_hook: Option<&str>,
    ) -> Result<ThreadGenerationOutput, LlmError> {
        tracing::debug!(
            topic = %topic,
            structure = ?structure,
            has_rag_context = rag_context.is_some(),
            has_opening_hook = opening_hook.is_some(),
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

        let (hook_rule, tweet_count_rule) = match opening_hook {
            Some(hook) => (
                format!(
                    "\n- The first tweet of the thread is ALREADY WRITTEN. \
                     Do NOT include it in your output.\n\
                     - Here is the first tweet (for context only): \"{hook}\"\n\
                     - Write 4 to 7 ADDITIONAL tweets that continue from that opening."
                ),
                "4 to 7",
            ),
            None => (
                "\n- The first tweet should hook the reader.".to_string(),
                "5 to 8",
            ),
        };

        let system = format!(
            "You are {}'s social media voice. {}.\
             {audience_section}\
             {voice_section}\
             {content_section}\
             {structure_section}\
             {persona_section}\
             {rag_section}\n\n\
             Rules:\n\
             - Write an educational thread of {tweet_count_rule} tweets about the topic below.\n\
             - Separate each tweet with a line containing only \"---\".\n\
             - Each tweet must be under 280 characters.{hook_rule}\n\
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

        // When we have an opening hook, we expect 4-7 generated tweets (prepend hook for 5-8 total).
        let (min_gen, max_gen) = if opening_hook.is_some() {
            (4, 7)
        } else {
            (5, 8)
        };

        for attempt in 0..=MAX_THREAD_RETRIES {
            let msg = if attempt == 0 {
                user_message.clone()
            } else {
                format!(
                    "{user_message}\n\nIMPORTANT: Write exactly {tweet_count_rule} tweets, \
                     each under 280 characters, separated by lines containing only \"---\"."
                )
            };

            let resp = self.provider.complete(&system, &msg, &params).await?;
            usage.accumulate(&resp.usage);
            model.clone_from(&resp.model);
            let mut tweets = parse_thread(&resp.text);

            // If hook provided, prepend it to form the complete thread.
            if let Some(hook) = opening_hook {
                tweets.insert(0, hook.to_string());
            }

            let gen_count = tweets.len() - if opening_hook.is_some() { 1 } else { 0 };
            if (min_gen..=max_gen).contains(&gen_count)
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

/// Strip common bullet/list prefixes from a line, tolerating varied LLM formats.
///
/// Handles: `- `, `* `, `• `, `1. `, `1) `, `(1) `, and combinations thereof.
/// Returns the remaining text trimmed, or empty string if the line is only a prefix.
fn strip_bullet_prefix(line: &str) -> String {
    let s = line
        .trim_start_matches(|c: char| c == '(' || c.is_ascii_whitespace())
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start_matches(['.', ')', ':', '-', '*', '•', '—'])
        .trim();
    s.to_string()
}
