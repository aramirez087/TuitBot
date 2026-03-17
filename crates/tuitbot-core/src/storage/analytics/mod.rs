//! CRUD operations for analytics tables.
//!
//! Manages follower snapshots, reply/tweet performance metrics,
//! content score running averages, and deep analytics signals:
//! engagement rate, reach, follower growth, and best-time-to-post.

mod ancestors;
mod best_times;
mod content_scores;
mod engagement;
mod performance_items;
mod reply_performance;
mod snapshots;
mod summary;
mod tweet_performance;

#[cfg(test)]
mod tests;

pub use ancestors::*;
pub use best_times::*;
pub use content_scores::*;
pub use engagement::*;
pub use performance_items::*;
pub use reply_performance::*;
pub use snapshots::*;
pub use summary::*;
pub use tweet_performance::*;
