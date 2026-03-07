//! CRUD operations for analytics tables.
//!
//! Manages follower snapshots, reply/tweet performance metrics,
//! and content score running averages.

mod ancestors;
mod content_scores;
mod performance_items;
mod reply_performance;
mod snapshots;
mod summary;
mod tweet_performance;

#[cfg(test)]
mod tests;

pub use ancestors::*;
pub use content_scores::*;
pub use performance_items::*;
pub use reply_performance::*;
pub use snapshots::*;
pub use summary::*;
pub use tweet_performance::*;
