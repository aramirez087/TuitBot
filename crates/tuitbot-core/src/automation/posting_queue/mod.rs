//! Serialized posting queue for concurrent automation loops.
//!
//! All loops funnel post actions through a single bounded MPSC channel,
//! preventing race conditions and ensuring rate limits are respected
//! globally. A single consumer task processes actions sequentially with
//! configurable delays between posts.

pub use dispatch::{ApprovalQueue, PostExecutor, run_posting_queue, run_posting_queue_with_approval};
pub use queue::{create_posting_queue, PostAction, QUEUE_CAPACITY};

mod dispatch;
mod queue;

#[cfg(test)]
mod tests_basic;
#[cfg(test)]
mod tests_dispatch;
