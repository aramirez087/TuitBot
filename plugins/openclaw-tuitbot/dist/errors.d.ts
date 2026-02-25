/**
 * MCP tool result parsing and error formatting.
 *
 * Tool call results arrive as `McpToolResult` with a JSON envelope inside
 * `content[0].text`. This module extracts the envelope, maps error codes
 * to actionable messages, and returns a structured `EnrichedToolResult`.
 */
import type { McpToolResult } from "./mcp-client.js";
export interface EnrichedToolResult {
    data: unknown;
    success: boolean;
    errorMessage?: string;
    errorCode?: string;
    retryable?: boolean;
    meta?: {
        tool_version: string;
        elapsed_ms: number;
        mode?: string;
        approval_mode?: boolean;
    };
}
/**
 * Format an error code into an actionable message.
 *
 * Uses a known mapping for recognized codes, falls back to the server
 * message for unknown codes. Appends server detail when it adds context.
 */
export declare function formatErrorMessage(code: string, serverMsg: string): string;
/**
 * Parse an MCP tool result into a structured `EnrichedToolResult`.
 *
 * 1. Extracts `content[0].text`; empty content → error result.
 * 2. Tries JSON.parse; non-JSON → returns raw text as data.
 * 3. If the parsed JSON has a `"success"` key, it's the Tuitbot envelope.
 * 4. Non-envelope JSON → returned as-is.
 */
export declare function parseToolResult(result: McpToolResult): EnrichedToolResult;
