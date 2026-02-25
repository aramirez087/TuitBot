/**
 * MCP JSON-RPC client over stdio.
 *
 * Spawns `tuitbot mcp serve` as a child process and communicates via
 * newline-delimited JSON-RPC 2.0 over stdin/stdout.
 */
import { EventEmitter } from "node:events";
export interface McpTool {
    name: string;
    description?: string;
    inputSchema: Record<string, unknown>;
}
export interface McpToolResult {
    content: Array<{
        type: string;
        text: string;
    }>;
    isError?: boolean;
}
export interface McpClientOptions {
    /** Path to the tuitbot binary. Defaults to "tuitbot". */
    binaryPath?: string;
    /** Path to the tuitbot config file. */
    configPath?: string;
    /** Additional environment variables for the child process. */
    env?: Record<string, string>;
}
export declare class McpClient extends EventEmitter {
    private options;
    private process;
    private readline;
    private pending;
    private started;
    constructor(options?: McpClientOptions);
    /** Start the MCP server child process and perform the initialize handshake. */
    start(): Promise<void>;
    /** List all tools exposed by the MCP server. */
    listTools(): Promise<McpTool[]>;
    /** Call an MCP tool by name with the given arguments. */
    callTool(name: string, args?: Record<string, unknown>): Promise<McpToolResult>;
    /** Gracefully stop the MCP server child process. */
    stop(): Promise<void>;
    private request;
    private notify;
    private handleLine;
}
