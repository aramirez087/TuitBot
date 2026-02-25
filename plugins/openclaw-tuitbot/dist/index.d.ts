/**
 * OpenClaw plugin entry point for Tuitbot.
 *
 * Starts an MCP client connected to `tuitbot mcp serve`, bridges its
 * tools into native OpenClaw tool registrations, and handles graceful
 * shutdown via a registered service.
 */
import { type OpenClawApi } from "./tool-bridge.js";
import type { ToolCategory, RiskLevel } from "./tool-catalog.js";
interface PluginConfig {
    tuitbotBinaryPath?: string;
    configPath?: string;
    allowedTools?: string[];
    /** Enable mutation tools. Default: false (safe startup). */
    enableMutations?: boolean;
    /** Category inclusion filter. Only these categories are registered. */
    allowCategories?: ToolCategory[];
    /** Category exclusion filter. These categories are blocked. */
    denyCategories?: ToolCategory[];
    /** Risk ceiling. Tools above this level are blocked. */
    maxRiskLevel?: RiskLevel;
}
interface OpenClawPluginApi extends OpenClawApi {
    config: PluginConfig;
    registerService(service: {
        name: string;
        stop: () => Promise<void>;
    }): void;
    log(level: "info" | "warn" | "error", message: string): void;
}
declare const _default: {
    id: string;
    name: string;
    register(api: OpenClawPluginApi): Promise<void>;
};
export default _default;
