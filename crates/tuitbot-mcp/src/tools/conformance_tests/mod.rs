//! Conformance tests for all kernel tool functions and spec-generated tools.
//!
//! Validates that every kernel read/write/engage tool produces a valid
//! ToolResponse envelope, and that error paths produce correct ErrorCode
//! values with accurate retryable flags and retry_after_ms fields.
//!
//! ## Test categories
//!
//! - **Deterministic** (`aggregate`, `read`, `write`, `engage`, `errors`):
//!   Mock-based, run in CI without credentials.
//! - **Spec conformance** (`dm`, `ads`, `enterprise_admin`): Validates
//!   generated tool properties, schemas, profiles, and safety invariants.
//! - **Parity** (`parity`): Structural check that every manifest tool
//!   has a backing implementation (curated handler or spec endpoint).
//! - **Live** (`live`): `#[ignore]` tests that hit real X API with sandbox
//!   credentials. Run with `cargo test -p tuitbot-mcp live -- --ignored`.
//! - **Coverage** (`coverage`): Generates machine-readable and markdown
//!   endpoint coverage reports from manifest introspection.

#[cfg(test)]
mod ads;
#[cfg(test)]
mod aggregate;
#[cfg(test)]
mod coverage;
#[cfg(test)]
mod dm;
#[cfg(test)]
mod engage;
#[cfg(test)]
mod enterprise_admin;
#[cfg(test)]
mod errors;
#[cfg(test)]
mod live;
#[cfg(test)]
mod parity;
#[cfg(test)]
mod read;
#[cfg(test)]
mod write;
