//! Audit helper for X API mutation tools.
//!
//! Provides [`begin_audited_mutation`] which combines:
//! 1. In-memory idempotency check (30s window)
//! 2. DB-backed idempotency check (5min window)
//! 3. Pending audit record creation
//!
//! And [`build_audited_response`] which:
//! 1. Completes the audit record
//! 2. Attaches correlation_id and rollback guidance to the response

use std::time::Instant;

use crate::state::SharedState;
use crate::tools::idempotency::{begin_mutation, MutationGuard};
use crate::tools::response::ToolMeta;
use crate::tools::rollback;

/// Result of beginning an audited mutation.
///
/// On `Proceed`, the caller executes the mutation and then calls
/// [`build_audited_response`] with the result.
/// On `EarlyReturn`, the caller returns the string immediately.
pub enum AuditGateResult {
    /// Mutation may proceed. Contains the guard to complete later.
    Proceed(MutationGuard),
    /// Return this JSON directly (duplicate or error).
    EarlyReturn(String),
}

/// Begin an audited mutation: in-memory dedup + DB idempotency + pending record.
pub async fn begin_audited_mutation(
    state: &SharedState,
    tool_name: &str,
    params_json: &str,
) -> AuditGateResult {
    // 1. In-memory dedup (fast path, 30s window).
    if let Some(err) = state.idempotency.check_and_record(tool_name, params_json) {
        return AuditGateResult::EarlyReturn(err);
    }

    // 2. DB-backed idempotency + audit record (5min window).
    match begin_mutation(&state.pool, tool_name, params_json).await {
        Ok(guard) => AuditGateResult::Proceed(guard),
        Err(cached_json) => AuditGateResult::EarlyReturn(cached_json),
    }
}

/// Build an audited success response with correlation_id and rollback guidance.
pub async fn complete_audited_success(
    guard: &MutationGuard,
    state: &SharedState,
    result_data: &serde_json::Value,
    elapsed: Instant,
) -> ToolMeta {
    let elapsed_ms = elapsed.elapsed().as_millis() as u64;
    let result_json = serde_json::to_string(result_data).unwrap_or_default();
    let guidance = rollback::guidance_for(&guard.tool_name, result_data);
    let rollback_json = rollback::guidance_to_json(&guidance);

    guard
        .complete(
            &state.pool,
            &result_json,
            rollback_json.as_deref(),
            elapsed_ms,
        )
        .await;

    let rollback_value = serde_json::to_value(&guidance).unwrap_or_default();

    ToolMeta::new(elapsed_ms)
        .with_correlation_id(&guard.correlation_id)
        .with_rollback(rollback_value)
}

/// Record a failed mutation in the audit trail and return metadata.
pub async fn complete_audited_failure(
    guard: &MutationGuard,
    state: &SharedState,
    error_msg: &str,
    elapsed: Instant,
) -> ToolMeta {
    let elapsed_ms = elapsed.elapsed().as_millis() as u64;
    guard.fail(&state.pool, error_msg, elapsed_ms).await;

    ToolMeta::new(elapsed_ms).with_correlation_id(&guard.correlation_id)
}

/// Build an audited error response for X API errors.
///
/// Records the failure in the audit trail and returns a JSON error response
/// with the correlation ID attached to the metadata.
pub async fn audited_x_error_response(
    guard: &MutationGuard,
    state: &SharedState,
    e: &tuitbot_core::error::XApiError,
    start: Instant,
) -> String {
    let meta = complete_audited_failure(guard, state, &e.to_string(), start).await;
    let provider_err = crate::provider::x_api::map_x_error(e);
    crate::contract::error::provider_error_to_audited_response(&provider_err, meta)
}
