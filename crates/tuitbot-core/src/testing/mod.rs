//! Test helpers, factories, and mock implementations for `tuitbot-core`.
//!
//! This module is gated behind the `test-helpers` feature flag so it is never
//! compiled into production binaries. Enable it in dev-dependencies with:
//!
//! ```toml
//! [dev-dependencies]
//! tuitbot-core = { path = "../tuitbot-core", features = ["test-helpers"] }
//! ```
//!
//! Or in `[features]` for crates that use it in `#[cfg(test)]` blocks.

pub mod account_factory;
pub mod approval_item_factory;
pub mod config_fixture;
pub mod mock_x_client;
pub mod tweet_factory;

pub use account_factory::AccountFactory;
pub use approval_item_factory::ApprovalItemFactory;
pub use config_fixture::ConfigFixture;
pub use mock_x_client::MockXClient;
pub use tweet_factory::TweetFactory;
