//! Content generation for tweets, replies, and threads.
//!
//! Uses an LLM provider to produce content that matches the user's
//! business profile and adheres to X's format constraints.
//! The `frameworks` module provides reply archetypes, tweet formats,
//! and thread structures that shape LLM prompts for varied output.

pub mod frameworks;
pub mod generator;
pub mod length;
pub mod thread;

pub use frameworks::{ReplyArchetype, ThreadStructure, TweetFormat};
pub use generator::{ContentGenerator, GenerationOutput, ThreadGenerationOutput};
pub use length::{
    truncate_at_sentence, tweet_weighted_len, validate_tweet_length, MAX_TWEET_CHARS,
    TCO_URL_LENGTH,
};
pub use thread::{
    deserialize_blocks_from_content, serialize_blocks_for_storage, validate_thread_blocks,
    ThreadBlock, ThreadBlockError, ThreadBlocksPayload, MAX_MEDIA_PER_BLOCK,
};
