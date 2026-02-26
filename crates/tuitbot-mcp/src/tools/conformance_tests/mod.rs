//! Conformance tests for all kernel tool functions.
//!
//! Validates that every kernel read/write/engage tool produces a valid
//! ToolResponse envelope, and that error paths produce correct ErrorCode
//! values with accurate retryable flags and retry_after_ms fields.

#[cfg(test)]
mod aggregate;
#[cfg(test)]
mod engage;
#[cfg(test)]
mod errors;
#[cfg(test)]
mod read;
#[cfg(test)]
mod write;
