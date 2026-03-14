//! Tweet generation, topic selection, and text-length utilities.
//!
//! Implements the `generate_and_post` and `pick_topic_epsilon_greedy` methods
//! on [`ContentLoop`], plus the free functions used by the scheduler.

use super::{ContentLoop, ContentResult, EXPLOIT_RATIO};
use rand::seq::SliceRandom;

impl ContentLoop {
    /// Generate a tweet and post it (or print in dry-run mode).
    pub(super) async fn generate_and_post(&self, topic: &str) -> ContentResult {
        tracing::info!(topic = %topic, "Generating tweet on topic");

        // Generate tweet
        let content = match self.generator.generate_tweet(topic).await {
            Ok(text) => text,
            Err(e) => {
                return ContentResult::Failed {
                    error: format!("Generation failed: {e}"),
                }
            }
        };

        // Validate length (280 char limit, URL-aware)
        let content = if crate::content::length::tweet_weighted_len(&content)
            > crate::content::length::MAX_TWEET_CHARS
        {
            // Retry once with explicit shorter instruction
            tracing::debug!(
                chars = content.len(),
                "Generated tweet too long, retrying with shorter instruction"
            );

            let shorter_topic = format!("{topic} (IMPORTANT: keep under 280 characters)");
            match self.generator.generate_tweet(&shorter_topic).await {
                Ok(text)
                    if crate::content::length::tweet_weighted_len(&text)
                        <= crate::content::length::MAX_TWEET_CHARS =>
                {
                    text
                }
                Ok(text) => {
                    // Truncate at word boundary
                    tracing::warn!(
                        chars = text.len(),
                        "Retry still too long, truncating at word boundary"
                    );
                    truncate_at_word_boundary(&text, 280)
                }
                Err(e) => {
                    // Use original but truncated
                    tracing::warn!(error = %e, "Retry generation failed, truncating original");
                    truncate_at_word_boundary(&content, 280)
                }
            }
        } else {
            content
        };

        if self.dry_run {
            tracing::info!(
                "DRY RUN: Would post tweet on topic '{}': \"{}\" ({} chars)",
                topic,
                content,
                content.len()
            );

            let _ = self
                .storage
                .log_action(
                    "tweet",
                    "dry_run",
                    &format!("Topic '{}': {}", topic, truncate_display(&content, 80)),
                )
                .await;
        } else {
            if let Err(e) = self.storage.post_tweet(topic, &content).await {
                tracing::error!(error = %e, "Failed to post tweet");
                let _ = self
                    .storage
                    .log_action("tweet", "failure", &format!("Post failed: {e}"))
                    .await;
                return ContentResult::Failed {
                    error: e.to_string(),
                };
            }

            let _ = self
                .storage
                .log_action(
                    "tweet",
                    "success",
                    &format!("Topic '{}': {}", topic, truncate_display(&content, 80)),
                )
                .await;
        }

        ContentResult::Posted {
            topic: topic.to_string(),
            content,
        }
    }

    /// Pick a topic using epsilon-greedy selection.
    ///
    /// If a topic scorer is available:
    /// - 80% of the time: pick from top-performing topics (exploit)
    /// - 20% of the time: pick a random topic (explore)
    ///
    /// Falls back to uniform random selection if no scorer is set or
    /// if the scorer returns no data.
    pub(super) async fn pick_topic_epsilon_greedy(
        &self,
        recent_topics: &mut Vec<String>,
        rng: &mut impl rand::Rng,
    ) -> String {
        if let Some(scorer) = &self.topic_scorer {
            let roll: f64 = rng.gen();
            if roll < EXPLOIT_RATIO {
                // Exploit: try to pick from top-performing topics
                if let Ok(top_topics) = scorer.get_top_topics(10).await {
                    // Filter to topics that are in our configured list and not recent
                    let candidates: Vec<&String> = top_topics
                        .iter()
                        .filter(|t| self.topics.contains(t) && !recent_topics.contains(t))
                        .collect();

                    if !candidates.is_empty() {
                        let topic = candidates[0].clone();
                        tracing::debug!(topic = %topic, "Epsilon-greedy: exploiting top topic");
                        return topic;
                    }
                }
                // Fall through to random if no top topics match
                tracing::debug!("Epsilon-greedy: no top topics available, falling back to random");
            } else {
                tracing::debug!("Epsilon-greedy: exploring random topic");
            }
        }

        pick_topic(&self.topics, recent_topics, rng)
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Pick a topic that is not in the recent list.
/// If all topics are recent, clear the list and pick any.
pub(super) fn pick_topic(
    topics: &[String],
    recent: &mut Vec<String>,
    rng: &mut impl rand::Rng,
) -> String {
    let available: Vec<&String> = topics.iter().filter(|t| !recent.contains(t)).collect();

    if available.is_empty() {
        // All topics recently used -- clear and pick any
        recent.clear();
        topics.choose(rng).expect("topics is non-empty").clone()
    } else {
        available
            .choose(rng)
            .expect("available is non-empty")
            .to_string()
    }
}

/// Truncate content at a word boundary, fitting within max_len characters.
pub(super) fn truncate_at_word_boundary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    // Find last space before max_len - 3 (for "...")
    let cutoff = max_len.saturating_sub(3);
    match s[..cutoff].rfind(' ') {
        Some(pos) => format!("{}...", &s[..pos]),
        None => format!("{}...", &s[..cutoff]),
    }
}

/// Truncate a string for display purposes.
pub(super) fn truncate_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::test_mocks::{
        make_topics, FailingGenerator, FailingTopicScorer, FirstCallRng, MockGenerator, MockSafety,
        MockStorage, MockTopicScorer, OverlongGenerator,
    };
    use super::super::{ContentLoop, ContentResult};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn overlong_tweet_gets_truncated() {
        let long_text = "a ".repeat(200); // 400 chars
        let content = ContentLoop::new(
            Arc::new(OverlongGenerator {
                first_response: long_text.clone(),
                retry_response: long_text,
                call_count: Mutex::new(0),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            14400,
            true,
        );

        let result = content.run_once(Some("Rust")).await;
        if let ContentResult::Posted { content, .. } = result {
            assert!(content.len() <= 280);
        } else {
            panic!("Expected Posted result");
        }
    }

    #[test]
    fn truncate_at_word_boundary_short() {
        let result = super::truncate_at_word_boundary("Hello world", 280);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn truncate_at_word_boundary_long() {
        let text = "The quick brown fox jumps over the lazy dog and more words here";
        let result = super::truncate_at_word_boundary(text, 30);
        assert!(result.len() <= 30);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn truncate_display_short() {
        assert_eq!(super::truncate_display("hello", 10), "hello");
    }

    #[test]
    fn truncate_display_long() {
        let result = super::truncate_display("hello world this is long", 10);
        assert_eq!(result, "hello worl...");
    }

    #[test]
    fn pick_topic_avoids_recent() {
        let topics = make_topics();
        let mut recent = vec!["Rust".to_string(), "CLI tools".to_string()];
        let mut rng = rand::thread_rng();

        for _ in 0..20 {
            let topic = super::pick_topic(&topics, &mut recent, &mut rng);
            assert_ne!(topic, "Rust");
            assert_ne!(topic, "CLI tools");
        }
    }

    #[test]
    fn pick_topic_clears_when_all_recent() {
        let topics = make_topics();
        let mut recent = topics.clone();
        let mut rng = rand::thread_rng();

        let topic = super::pick_topic(&topics, &mut recent, &mut rng);
        assert!(topics.contains(&topic));
        assert!(recent.is_empty());
    }

    #[tokio::test]
    async fn epsilon_greedy_exploits_top_topic() {
        let storage = Arc::new(MockStorage::new(None));
        let scorer = Arc::new(MockTopicScorer {
            top_topics: vec!["Rust".to_string()],
        });

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        )
        .with_topic_scorer(scorer);

        let mut recent = Vec::new();
        let mut rng = FirstCallRng::low_roll();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert_eq!(topic, "Rust");
    }

    #[tokio::test]
    async fn epsilon_greedy_explores_when_roll_high() {
        let storage = Arc::new(MockStorage::new(None));
        let scorer = Arc::new(MockTopicScorer {
            top_topics: vec!["Rust".to_string()],
        });

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        )
        .with_topic_scorer(scorer);

        let mut recent = Vec::new();
        let mut rng = FirstCallRng::high_roll();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert!(make_topics().contains(&topic));
    }

    #[tokio::test]
    async fn epsilon_greedy_falls_back_on_scorer_error() {
        let storage = Arc::new(MockStorage::new(None));
        let scorer = Arc::new(FailingTopicScorer);

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        )
        .with_topic_scorer(scorer);

        let mut recent = Vec::new();
        let mut rng = FirstCallRng::low_roll();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert!(make_topics().contains(&topic));
    }

    #[tokio::test]
    async fn epsilon_greedy_without_scorer_picks_random() {
        let storage = Arc::new(MockStorage::new(None));

        let content = ContentLoop::new(
            Arc::new(MockGenerator {
                response: "tweet".to_string(),
            }),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            storage,
            make_topics(),
            14400,
            false,
        );

        let mut recent = Vec::new();
        let mut rng = rand::thread_rng();

        let topic = content
            .pick_topic_epsilon_greedy(&mut recent, &mut rng)
            .await;
        assert!(make_topics().contains(&topic));
    }

    #[tokio::test]
    async fn generation_failure_returns_failed() {
        let content = ContentLoop::new(
            Arc::new(FailingGenerator),
            Arc::new(MockSafety {
                can_tweet: true,
                can_thread: true,
            }),
            Arc::new(MockStorage::new(None)),
            make_topics(),
            14400,
            false,
        );

        let result = content.run_once(Some("Rust")).await;
        assert!(matches!(result, ContentResult::Failed { .. }));
    }
}
