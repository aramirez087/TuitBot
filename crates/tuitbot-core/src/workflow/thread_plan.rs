//! Thread plan step: generate and analyze a multi-tweet thread via LLM.
//!
//! Produces a structured thread plan with hook analysis and performance estimates.

use std::sync::Arc;

use crate::config::Config;
use crate::content::frameworks::ThreadStructure;
use crate::llm::LlmProvider;

use super::{make_content_gen, WorkflowError};

/// Input for the thread plan step.
#[derive(Debug, Clone)]
pub struct ThreadPlanInput {
    /// The topic to write a thread about.
    pub topic: String,
    /// Optional objective (e.g., "establish expertise").
    pub objective: Option<String>,
    /// Optional target audience description.
    pub target_audience: Option<String>,
    /// Optional structure override (e.g., "transformation", "framework").
    pub structure: Option<String>,
}

/// Output from the thread plan step.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThreadPlanOutput {
    /// The generated tweets in thread order.
    pub thread_tweets: Vec<String>,
    /// Number of tweets in the thread.
    pub tweet_count: usize,
    /// The structure that was used.
    pub structure_used: String,
    /// Hook analysis for the first tweet.
    pub hook_type: String,
    /// First tweet preview.
    pub first_tweet_preview: String,
    /// Estimated performance based on topic relevance.
    pub estimated_performance: String,
    /// Objective alignment description.
    pub objective_alignment: String,
    /// Target audience description.
    pub target_audience: String,
    /// Topic relevance assessment.
    pub topic_relevance: String,
}

/// Parse a structure string into a `ThreadStructure`.
fn parse_structure(s: &str) -> Option<ThreadStructure> {
    match s.to_lowercase().as_str() {
        "transformation" => Some(ThreadStructure::Transformation),
        "framework" => Some(ThreadStructure::Framework),
        "mistakes" => Some(ThreadStructure::Mistakes),
        "analysis" => Some(ThreadStructure::Analysis),
        _ => None,
    }
}

/// Analyze the hook type of the first tweet.
fn analyze_hook(first_tweet: &str) -> &'static str {
    let trimmed = first_tweet.trim();
    if trimmed.ends_with('?') {
        "question"
    } else if trimmed.starts_with("Most people")
        || trimmed.starts_with("Everyone")
        || trimmed.starts_with("most people")
    {
        "contrarian"
    } else if trimmed.starts_with("I ") || trimmed.starts_with("I'") {
        "story"
    } else {
        "statement"
    }
}

/// Execute the thread plan step.
pub async fn execute(
    llm: &Arc<dyn LlmProvider>,
    config: &Config,
    input: ThreadPlanInput,
) -> Result<ThreadPlanOutput, WorkflowError> {
    let structure_override = input.structure.as_deref().and_then(parse_structure);

    let gen = make_content_gen(llm, &config.business);

    let thread = gen
        .generate_thread_with_structure(&input.topic, structure_override)
        .await?;

    let tweet_count = thread.tweets.len();
    let hook_type = thread
        .tweets
        .first()
        .map(|t| analyze_hook(t))
        .unwrap_or("unknown");

    // Relevance heuristic: check if topic overlaps with configured industry topics
    let topic_lower = input.topic.to_lowercase();
    let relevance = config.business.effective_industry_topics().iter().any(|t| {
        topic_lower.contains(&t.to_lowercase()) || t.to_lowercase().contains(&topic_lower)
    });

    let estimated_performance = if relevance { "high" } else { "medium" };
    let structure_used = input.structure.as_deref().unwrap_or("auto_selected");

    Ok(ThreadPlanOutput {
        first_tweet_preview: thread.tweets.first().cloned().unwrap_or_default(),
        thread_tweets: thread.tweets,
        tweet_count,
        structure_used: structure_used.to_string(),
        hook_type: hook_type.to_string(),
        estimated_performance: estimated_performance.to_string(),
        objective_alignment: input
            .objective
            .unwrap_or_else(|| "general engagement".to_string()),
        target_audience: input
            .target_audience
            .unwrap_or_else(|| "general".to_string()),
        topic_relevance: if relevance {
            "matches_industry_topics"
        } else {
            "novel_topic"
        }
        .to_string(),
    })
}
