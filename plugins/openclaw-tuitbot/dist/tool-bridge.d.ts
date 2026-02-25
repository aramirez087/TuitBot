/**
 * Converts MCP tools into OpenClaw tool registrations with layered filtering.
 *
 * Applies a multi-stage filter pipeline (name allowlist, mutation gate,
 * category filters, risk ceiling) and enriches tool descriptions with
 * metadata from the tool catalog. Wraps execution to parse the MCP
 * result envelope into structured `EnrichedToolResult`.
 */
import type { McpToolResult } from "./mcp-client.js";
import { type ToolCategory, type RiskLevel } from "./tool-catalog.js";
import { type EnrichedToolResult } from "./errors.js";
export interface OpenClawToolRegistration {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
    optional: boolean;
    execute: (args: Record<string, unknown>) => Promise<EnrichedToolResult>;
}
export interface OpenClawApi {
    registerTool(tool: OpenClawToolRegistration): void;
}
export interface McpClientLike {
    listTools(): Promise<Array<{
        name: string;
        description?: string;
        inputSchema: Record<string, unknown>;
    }>>;
    callTool(name: string, args: Record<string, unknown>): Promise<McpToolResult>;
}
export interface BridgeOptions {
    /** MCP tool names to register. Empty or undefined = register all that pass filters. */
    allowedTools?: string[];
    /** Enable mutation tools. Default: false (safe startup). */
    enableMutations?: boolean;
    /** Category inclusion filter. Only these categories pass (if set). */
    allowCategories?: ToolCategory[];
    /** Category exclusion filter. These categories are blocked (if set). */
    denyCategories?: ToolCategory[];
    /** Risk ceiling. Tools above this level are blocked (if set). */
    maxRiskLevel?: RiskLevel;
}
/**
 * Determine whether a tool should be registered based on layered filters.
 *
 * Filter order:
 * 1. Name allowlist (most restrictive — if set, only named tools pass)
 * 2. Catalog lookup (unknown tools pass by default for forward-compatibility)
 * 3. Mutation gate: mutations and policy-gated composites require `enableMutations`
 * 4. Category allowlist (if set)
 * 5. Category denylist (if set)
 * 6. Risk level ceiling (if set)
 */
export declare function shouldRegisterTool(name: string, options: BridgeOptions): boolean;
/**
 * Bridge MCP tools into OpenClaw tool registrations.
 *
 * @returns The number of tools registered.
 */
export declare function bridgeTools(client: McpClientLike, api: OpenClawApi, options?: BridgeOptions): Promise<number>;
