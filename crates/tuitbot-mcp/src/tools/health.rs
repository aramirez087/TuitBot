//! Health check tool.

use serde::Serialize;

use tuitbot_core::llm::LlmProvider;
use tuitbot_core::storage;
use tuitbot_core::storage::DbPool;

#[derive(Serialize)]
struct HealthStatus {
    database: ComponentStatus,
    llm: ComponentStatus,
}

#[derive(Serialize)]
struct ComponentStatus {
    status: String,
    message: String,
}

/// Check system health: database connectivity and LLM provider status.
pub async fn health_check(pool: &DbPool, llm_provider: Option<&dyn LlmProvider>) -> String {
    // Check database by running a simple query through the storage layer
    let db_status = match storage::analytics::get_follower_snapshots(pool, 1).await {
        Ok(_) => ComponentStatus {
            status: "ok".to_string(),
            message: "Database is accessible".to_string(),
        },
        Err(e) => ComponentStatus {
            status: "error".to_string(),
            message: format!("Database error: {e}"),
        },
    };

    // Check LLM provider
    let llm_status = match llm_provider {
        Some(provider) => match provider.health_check().await {
            Ok(()) => ComponentStatus {
                status: "ok".to_string(),
                message: format!("LLM provider '{}' is reachable", provider.name()),
            },
            Err(e) => ComponentStatus {
                status: "error".to_string(),
                message: format!("LLM provider '{}' error: {e}", provider.name()),
            },
        },
        None => ComponentStatus {
            status: "not_configured".to_string(),
            message: "No LLM provider configured. Content generation tools will not work."
                .to_string(),
        },
    };

    let out = HealthStatus {
        database: db_status,
        llm: llm_status,
    };

    serde_json::to_string_pretty(&out).unwrap_or_else(|e| format!("Error serializing health: {e}"))
}
