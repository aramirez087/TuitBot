//! Serialized posting queue for concurrent automation loops.
//!
//! All loops funnel post actions through a single bounded MPSC channel,
//! preventing race conditions and ensuring rate limits are respected
//! globally. A single consumer task processes actions sequentially with
//! configurable delays between posts.

mod dispatch;
mod queue;

// Re-export public API
pub use dispatch::{
    run_posting_queue, run_posting_queue_with_approval, ApprovalQueue, PostExecutor,
};
pub use queue::{create_posting_queue, PostAction, QUEUE_CAPACITY};
