//! Kernel layer: tool dispatch against provider traits.
//!
//! Tools in this layer depend only on [`SocialReadProvider`](crate::provider::SocialReadProvider)
//! and the contract envelope — never on `AppState`, `DbPool`, or concrete API clients.

pub mod read;

#[cfg(test)]
mod tests;
