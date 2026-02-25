import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { parseToolResult, formatErrorMessage } from "../errors.js";
// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
function makeResult(text, isError) {
    return { content: [{ type: "text", text }], isError };
}
function makeEnvelope(opts) {
    return JSON.stringify(opts);
}
// ---------------------------------------------------------------------------
// parseToolResult
// ---------------------------------------------------------------------------
describe("parseToolResult", () => {
    it("extracts data and meta from a success envelope", () => {
        const text = makeEnvelope({
            success: true,
            data: { count: 42 },
            meta: { tool_version: "0.12.0", elapsed_ms: 15, mode: "autopilot" },
        });
        const result = parseToolResult(makeResult(text));
        assert.equal(result.success, true);
        assert.deepEqual(result.data, { count: 42 });
        assert.equal(result.meta?.tool_version, "0.12.0");
        assert.equal(result.meta?.elapsed_ms, 15);
        assert.equal(result.errorMessage, undefined);
    });
    it("parses a retryable error envelope", () => {
        const text = makeEnvelope({
            success: false,
            error: { code: "x_rate_limited", message: "rate limited", retryable: true },
            meta: { tool_version: "0.12.0", elapsed_ms: 5 },
        });
        const result = parseToolResult(makeResult(text));
        assert.equal(result.success, false);
        assert.equal(result.errorCode, "x_rate_limited");
        assert.equal(result.retryable, true);
        assert.ok(result.errorMessage?.includes("rate limit"));
    });
    it("parses a non-retryable error envelope", () => {
        const text = makeEnvelope({
            success: false,
            error: { code: "policy_denied_blocked", message: "blocked by policy", retryable: false },
        });
        const result = parseToolResult(makeResult(text));
        assert.equal(result.success, false);
        assert.equal(result.retryable, false);
        assert.ok(result.errorMessage?.includes("blocked"));
    });
    it("passes through legacy JSON (no envelope)", () => {
        const text = JSON.stringify({ tweets: [1, 2, 3] });
        const result = parseToolResult(makeResult(text));
        assert.equal(result.success, true);
        assert.deepEqual(result.data, { tweets: [1, 2, 3] });
        assert.equal(result.meta, undefined);
    });
    it("returns raw text for non-JSON content", () => {
        const result = parseToolResult(makeResult("OK: 3 tweets found"));
        assert.equal(result.success, true);
        assert.equal(result.data, "OK: 3 tweets found");
    });
    it("returns error for empty content array", () => {
        const result = parseToolResult({ content: [] });
        assert.equal(result.success, false);
        assert.equal(result.errorCode, "empty_response");
    });
    it("returns error for empty text in content", () => {
        const result = parseToolResult({ content: [{ type: "text", text: "" }] });
        assert.equal(result.success, false);
        assert.equal(result.errorCode, "empty_response");
    });
    it("returns error when isError flag is set", () => {
        const result = parseToolResult(makeResult("something went wrong", true));
        assert.equal(result.success, false);
        assert.equal(result.errorCode, "mcp_error");
        assert.equal(result.errorMessage, "something went wrong");
    });
    it("handles missing content with isError flag", () => {
        const result = parseToolResult({ content: [], isError: true });
        assert.equal(result.success, false);
        assert.equal(result.errorCode, "mcp_error");
    });
});
// ---------------------------------------------------------------------------
// formatErrorMessage
// ---------------------------------------------------------------------------
describe("formatErrorMessage", () => {
    it("maps all known error codes to non-empty messages", () => {
        const knownCodes = [
            "x_rate_limited", "x_auth_expired", "x_auth_missing", "x_forbidden",
            "x_not_found", "x_api_error", "llm_not_configured", "llm_generation_failed",
            "llm_parse_error", "config_invalid", "config_not_found", "db_error",
            "policy_denied_blocked", "policy_denied_approval", "policy_not_evaluated",
            "safety_duplicate", "safety_rate_limit", "safety_banned_phrase",
        ];
        for (const code of knownCodes) {
            const msg = formatErrorMessage(code, "server detail");
            assert.ok(msg.length > 0, `Expected non-empty message for code "${code}"`);
        }
    });
    it("falls back to server message for unknown codes", () => {
        const msg = formatErrorMessage("totally_unknown", "custom server error");
        assert.equal(msg, "custom server error");
    });
    it("appends server detail when it differs from template", () => {
        const msg = formatErrorMessage("x_rate_limited", "429 Too Many Requests");
        assert.ok(msg.includes("rate limit"));
        assert.ok(msg.includes("429 Too Many Requests"));
    });
    it("does not duplicate when server message matches template", () => {
        const template = "X API rate limit hit. Wait before retrying.";
        const msg = formatErrorMessage("x_rate_limited", template);
        assert.equal(msg, template);
    });
    it("does not append when server message is just the code", () => {
        const msg = formatErrorMessage("x_rate_limited", "x_rate_limited");
        assert.equal(msg, "X API rate limit hit. Wait before retrying.");
    });
});
//# sourceMappingURL=errors.test.js.map