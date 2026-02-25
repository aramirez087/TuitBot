/**
 * OpenClaw plugin entry point for Tuitbot.
 *
 * Starts an MCP client connected to `tuitbot mcp serve`, bridges its
 * tools into native OpenClaw tool registrations, and handles graceful
 * shutdown via a registered service.
 */
import { McpClient } from "./mcp-client.js";
import { bridgeTools } from "./tool-bridge.js";
// ---------------------------------------------------------------------------
// Plugin export
// ---------------------------------------------------------------------------
export default {
    id: "tuitbot",
    name: "Tuitbot",
    async register(api) {
        const config = api.config;
        const client = new McpClient({
            binaryPath: config.tuitbotBinaryPath,
            configPath: config.configPath,
        });
        client.on("error", (err) => {
            api.log("error", `Tuitbot MCP client error: ${err.message}`);
        });
        client.on("exit", (code) => {
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
//# sourceMappingURL=index.js.map