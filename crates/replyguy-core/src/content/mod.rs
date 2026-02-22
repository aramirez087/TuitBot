//! Content generation for tweets, replies, and threads.
//!
//! Uses an LLM provider to produce content that matches the user's
//! business profile and adheres to X's format constraints.

pub mod generator;

pub use generator::ContentGenerator;
