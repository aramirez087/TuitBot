/**
 * Static metadata catalog for all Tuitbot MCP tools.
 *
 * Maps each tool name to its category, risk level, and whether it
 * requires a policy check before execution.
 */
export type ToolCategory = "read" | "mutation" | "composite" | "ops";
export type RiskLevel = "low" | "medium" | "high";
export interface ToolMeta {
    category: ToolCategory;
    riskLevel: RiskLevel;
    requiresPolicyCheck: boolean;
}
export declare const RISK_ORDER: RiskLevel[];
/** Returns true if `level` is at most `max` in the risk ordering. */
export declare function riskAtMost(level: RiskLevel, max: RiskLevel): boolean;
/** Look up metadata for a tool by name. Returns `undefined` for unknown tools. */
export declare function getToolMeta(name: string): ToolMeta | undefined;
