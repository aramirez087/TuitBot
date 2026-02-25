/**
 * OpenClaw plugin entry point for Tuitbot.
 *
 * Starts an MCP client connected to `tuitbot mcp serve`, bridges its
 * tools into native OpenClaw tool registrations, and handles graceful
 * shutdown via a registered service.
 */

import { McpClient } from "./mcp-client.js";
import { bridgeTools, type OpenClawApi } from "./tool-bridge.js";
import type { ToolCategory, RiskLevel } from "./tool-catalog.js";

// ---------------------------------------------------------------------------
// OpenClaw plugin API types (minimal surface)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Plugin export
// ---------------------------------------------------------------------------

export default {
  id: "tuitbot",
  name: "Tuitbot",

  async register(api: OpenClawPluginApi): Promise<void> {
    const config = api.config;

    const client = new McpClient({
      binaryPath: config.tuitbotBinaryPath,
      configPath: config.configPath,
    });

    client.on("error", (err: Error) => {
      api.log("error", `Tuitbot MCP client error: ${err.message}`);
    });

    client.on("exit", (code: number | null) => {
      api.log("warn", `Tuitbot MCP process exited with code ${code}`);
    });

    await client.start();

    const enableMutations = config.enableMutations ?? false;

    const count = await bridgeTools(client, api, {
      allowedTools: config.allowedTools,
      enableMutations,
      allowCategories: config.allowCategories,
      denyCategories: config.denyCategories,
      maxRiskLevel: config.maxRiskLevel,
    });

    const mutLabel = enableMutations ? "enabled" : "disabled";
    api.log("info", `Tuitbot plugin registered ${count} tools (mutations: ${mutLabel})`);

    // Register a service for graceful shutdown.
    api.registerService({
      name: "tuitbot-mcp",
      stop: () => client.stop(),
    });
  },
};
