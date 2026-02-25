/**
 * MCP tool result parsing and error formatting.
 *
 * Tool call results arrive as `McpToolResult` with a JSON envelope inside
 * `content[0].text`. This module extracts the envelope, maps error codes
 * to actionable messages, and returns a structured `EnrichedToolResult`.
 */
// ---------------------------------------------------------------------------
// Error code → actionable message map
// ---------------------------------------------------------------------------
const ERROR_MESSAGES = {
    x_rate_limited: "X API rate limit hit. Wait before retrying.",
    x_auth_expired: "X API authentication expired. Re-authenticate with `tuitbot auth`.",
    x_auth_missing: "X API credentials not configured. Run `tuitbot auth` to set up.",
    x_forbidden: "X API returned 403 Forbidden. Check account permissions.",
    x_not_found: "The requested X resource was not found.",
    x_api_error: "X API returned an unexpected error.",
    llm_not_configured: "LLM provider not configured. Set up the [llm] section in config.toml.",
    llm_generation_failed: "LLM generation failed. Check provider connectivity and API key.",
    llm_parse_error: "Failed to parse LLM response. The model returned an unexpected format.",
    config_invalid: "Configuration is invalid. Run `tuitbot validate-config` for details.",
    config_not_found: "Configuration file not found. Run `tuitbot init` to create one.",
    db_error: "Database error. Check that the SQLite database is accessible.",
    policy_denied_blocked: "This tool is blocked by MCP policy configuration.",
    policy_denied_approval: "This action requires approval. Submit via the approval queue.",
    policy_not_evaluated: "Policy evaluation failed. Check policy configuration.",
    safety_duplicate: "Duplicate content detected. This reply was already posted.",
    safety_rate_limit: "Internal rate limit reached. Wait before posting again.",
    safety_banned_phrase: "Content contains a banned phrase. Edit and retry.",
};
// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------
/**
 * Format an error code into an actionable message.
 *
 * Uses a known mapping for recognized codes, falls back to the server
 * message for unknown codes. Appends server detail when it adds context.
 */
export function formatErrorMessage(code, serverMsg) {
    const template = ERROR_MESSAGES[code];
    if (!template)
        return serverMsg;
    if (serverMsg && serverMsg !== template && serverMsg !== code) {
        return `${template} (${serverMsg})`;
    }
    return template;
}
/**
 * Parse an MCP tool result into a structured `EnrichedToolResult`.
 *
 * 1. Extracts `content[0].text`; empty content → error result.
 * 2. Tries JSON.parse; non-JSON → returns raw text as data.
 * 3. If the parsed JSON has a `"success"` key, it's the Tuitbot envelope.
 * 4. Non-envelope JSON → returned as-is.
 */
export function parseToolResult(result) {
    // Handle isError flag from MCP layer
    if (result.isError) {
        const text = result.content?.[0]?.text ?? "Unknown MCP error";
        return {
            data: null,
            success: false,
            errorMessage: text,
            errorCode: "mcp_error",
        };
    }
    // Empty content
    if (!result.content || result.content.length === 0 || !result.content[0]?.text) {
        return {
            data: null,
            success: false,
            errorMessage: "Empty response from tool",
            errorCode: "empty_response",
        };
    }
    const text = result.content[0].text;
    // Try to parse as JSON
    let parsed;
    try {
        parsed = JSON.parse(text);
    }
    catch {
        // Not JSON — return raw text as data
        return { data: text, success: true };
    }
    // Check if this is the Tuitbot ToolResponse envelope
    if (parsed !== null &&
        typeof parsed === "object" &&
        "success" in parsed) {
        const envelope = parsed;
        if (envelope.success) {
            return {
                data: envelope.data,
                success: true,
                meta: envelope.meta,
            };
        }
        // Error envelope
        const code = envelope.error?.code ?? "unknown";
        const msg = envelope.error?.message ?? "Unknown error";
        return {
            data: null,
            success: false,
            errorCode: code,
            errorMessage: formatErrorMessage(code, msg),
            retryable: envelope.error?.retryable,
            meta: envelope.meta,
        };
    }
    // Non-envelope JSON — return as-is
    return { data: parsed, success: true };
}
//# sourceMappingURL=errors.js.map