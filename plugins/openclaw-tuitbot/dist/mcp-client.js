/**
 * MCP JSON-RPC client over stdio.
 *
 * Spawns `tuitbot mcp serve` as a child process and communicates via
 * newline-delimited JSON-RPC 2.0 over stdin/stdout.
 */
import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { once, EventEmitter } from "node:events";
import { createInterface } from "node:readline";
// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------
export class McpClient extends EventEmitter {
    options;
    process = null;
    readline = null;
    pending = new Map();
    started = false;
    constructor(options = {}) {
        super();
        this.options = options;
    }
    /** Start the MCP server child process and perform the initialize handshake. */
    async start() {
        if (this.started)
            return;
        const bin = this.options.binaryPath ?? "tuitbot";
        const args = ["mcp", "serve"];
        if (this.options.configPath) {
            args.unshift("--config", this.options.configPath);
        }
        const env = {
            ...process.env,
            OPENCLAW_PLUGIN: "tuitbot",
            ...this.options.env,
        };
        this.process = spawn(bin, args, {
            stdio: ["pipe", "pipe", "pipe"],
            env,
        });
        this.process.on("error", (err) => {
            this.emit("error", err);
        });
        this.process.on("exit", (code) => {
            this.started = false;
            // Reject all pending requests.
            for (const [id, pending] of this.pending) {
                pending.reject(new Error(`MCP process exited with code ${code}`));
                this.pending.delete(id);
            }
            this.emit("exit", code);
        });
        // Read newline-delimited JSON-RPC responses from stdout.
        this.readline = createInterface({ input: this.process.stdout });
        this.readline.on("line", (line) => {
            this.handleLine(line);
        });
        // Perform initialize handshake.
        const initResult = await this.request("initialize", {
            protocolVersion: "2024-11-05",
            capabilities: {},
            clientInfo: { name: "openclaw-tuitbot-plugin", version: "0.1.0" },
        });
        // Send initialized notification.
        this.notify("notifications/initialized");
        this.started = true;
        this.emit("ready", initResult);
    }
    /** List all tools exposed by the MCP server. */
    async listTools() {
        const result = (await this.request("tools/list", {}));
        return result.tools;
    }
    /** Call an MCP tool by name with the given arguments. */
    async callTool(name, args = {}) {
        const result = (await this.request("tools/call", {
            name,
            arguments: args,
        }));
        return result;
    }
    /** Gracefully stop the MCP server child process. */
    async stop() {
        if (!this.process)
            return;
        this.readline?.close();
        this.readline = null;
        const proc = this.process;
        this.process = null;
        this.started = false;
        // Give the process time to exit gracefully.
        const exitPromise = once(proc, "exit").catch(() => { });
        proc.kill("SIGTERM");
        const timeout = setTimeout(() => {
            proc.kill("SIGKILL");
        }, 5_000);
        await exitPromise;
        clearTimeout(timeout);
    }
    // -----------------------------------------------------------------------
    // Private
    // -----------------------------------------------------------------------
    request(method, params) {
        return new Promise((resolve, reject) => {
            if (!this.process?.stdin?.writable) {
                return reject(new Error("MCP process is not running"));
            }
            const id = randomUUID();
            const message = {
                jsonrpc: "2.0",
                id,
                method,
                ...(params && { params }),
            };
            this.pending.set(id, { resolve, reject });
            this.process.stdin.write(JSON.stringify(message) + "\n");
        });
    }
    notify(method, params) {
        if (!this.process?.stdin?.writable)
            return;
        const message = {
            jsonrpc: "2.0",
            method,
            ...(params && { params }),
        };
        this.process.stdin.write(JSON.stringify(message) + "\n");
    }
    handleLine(line) {
        const trimmed = line.trim();
        if (!trimmed)
            return;
        let response;
        try {
            response = JSON.parse(trimmed);
        }
        catch {
            // Not a valid JSON-RPC message — might be a log line, ignore.
            return;
        }
        if (!response.id)
            return; // Notification from server, ignore.
        const pending = this.pending.get(response.id);
        if (!pending)
            return;
        this.pending.delete(response.id);
        if (response.error) {
            pending.reject(new Error(`MCP error ${response.error.code}: ${response.error.message}`));
        }
        else {
            pending.resolve(response.result);
        }
    }
}
//# sourceMappingURL=mcp-client.js.map