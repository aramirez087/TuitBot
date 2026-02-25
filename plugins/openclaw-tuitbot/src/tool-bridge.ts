/**
 * Converts MCP tools into OpenClaw tool registrations with layered filtering.
 *
 * Applies a multi-stage filter pipeline (name allowlist, mutation gate,
 * category filters, risk ceiling) and enriches tool descriptions with
 * metadata from the tool catalog. Wraps execution to parse the MCP
 * result envelope into structured `EnrichedToolResult`.
 */

import type { McpToolResult } from "./mcp-client.js";
import { getToolMeta, riskAtMost, type ToolCategory, type RiskLevel } from "./tool-catalog.js";
import { parseToolResult, type EnrichedToolResult } from "./errors.js";

// ---------------------------------------------------------------------------
// OpenClaw API types (minimal surface used by the plugin)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MCP client interface (minimal surface for testability)
// ---------------------------------------------------------------------------

export interface McpClientLike {
  listTools(): Promise<Array<{ name: string; description?: string; inputSchema: Record<string, unknown> }>>;
  callTool(name: string, args: Record<string, unknown>): Promise<McpToolResult>;
}

// ---------------------------------------------------------------------------
// Bridge options
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Filter pipeline
// ---------------------------------------------------------------------------

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
export function shouldRegisterTool(name: string, options: BridgeOptions): boolean {
  // 1. Name allowlist
  if (options.allowedTools && options.allowedTools.length > 0) {
    if (!options.allowedTools.includes(name)) return false;
  }

  // 2. Catalog lookup
  const meta = getToolMeta(name);
  if (!meta) return true; // Unknown tools pass (forward-compatible)

  // 3. Mutation gate
  const isMutating =
    meta.category === "mutation" ||
    (meta.category === "composite" && meta.requiresPolicyCheck);
  if (isMutating && !options.enableMutations) return false;

  // 4. Category allowlist
  if (options.allowCategories && options.allowCategories.length > 0) {
    if (!options.allowCategories.includes(meta.category)) return false;
  }

  // 5. Category denylist
  if (options.denyCategories && options.denyCategories.length > 0) {
    if (options.denyCategories.includes(meta.category)) return false;
  }

  // 6. Risk ceiling
  if (options.maxRiskLevel) {
    if (!riskAtMost(meta.riskLevel, options.maxRiskLevel)) return false;
  }

  return true;
}

// ---------------------------------------------------------------------------
// Bridge
// ---------------------------------------------------------------------------

/**
 * Bridge MCP tools into OpenClaw tool registrations.
 *
 * @returns The number of tools registered.
 */
export async function bridgeTools(
  client: McpClientLike,
  api: OpenClawApi,
  options: BridgeOptions = {},
): Promise<number> {
  const mcpTools = await client.listTools();
  let count = 0;

  for (const tool of mcpTools) {
    if (!shouldRegisterTool(tool.name, options)) continue;

    const meta = getToolMeta(tool.name);
    const tag = meta
      ? `[${meta.category}${meta.requiresPolicyCheck ? " | policy-gated" : ""}]`
      : "[unknown]";

    const description = `${tag} ${tool.description ?? `Tuitbot MCP tool: ${tool.name}`}`;
    const openclawName = `tuitbot_${tool.name}`;

    api.registerTool({
      name: openclawName,
      description,
      parameters: tool.inputSchema,
      optional: true,
      execute: async (args: Record<string, unknown>): Promise<EnrichedToolResult> => {
        const result = await client.callTool(tool.name, args);
        return parseToolResult(result);
      },
    });

    count++;
  }

  return count;
}
