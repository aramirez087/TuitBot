//! Adapter implementations bridging port traits to real dependencies.
//!
//! Each adapter struct wraps one or more concrete dependencies (X API client,
//! content generator, scoring engine, safety guard, database pool, posting queue)
//! and implements the port traits defined in [`loop_helpers`], [`analytics_loop`],
//! [`target_loop`], [`thread_loop`], [`posting_queue`], and [`status_reporter`].

mod helpers;
mod llm;
mod queue;
mod safety;
mod scoring;
mod status;
mod storage;
mod x_api;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_safety;
#[cfg(test)]
mod tests_storage;

pub use llm::*;
pub use queue::*;
pub use safety::*;
pub use scoring::*;
pub use status::*;
pub use storage::*;
pub use x_api::*;
