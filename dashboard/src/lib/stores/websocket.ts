import { writable } from 'svelte/store';

/** Events pushed by the tuitbot-server WebSocket. */
export interface WsEvent {
    type: 'ActionPerformed' | 'ApprovalQueued' | 'ApprovalUpdated' | 'FollowerUpdate' | 'RuntimeStatus' | 'ContentScheduled' | 'Error';
    [key: string]: unknown;
}

/** Recent WebSocket events (newest first, capped at 200). */
export const events = writable<WsEvent[]>([]);

/** Whether the WebSocket is currently connected. */
export const connected = writable(false);

/** Whether the tuitbot-server runtime is running. */
export const runtimeRunning = writable(false);

let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 1000;
const MAX_RECONNECT_DELAY = 30000;

/**
 * Connect to the tuitbot-server WebSocket.
 * Automatically reconnects with exponential backoff on disconnect.
 */
export function connectWs(token: string) {
    if (ws) {
        ws.close();
    }

    ws = new WebSocket(`ws://localhost:3001/api/ws?token=${token}`);

    ws.onopen = () => {
        connected.set(true);
        reconnectDelay = 1000; // Reset backoff on successful connect
    };

    ws.onclose = () => {
        connected.set(false);
        runtimeRunning.set(false);
        ws = null;

        // Reconnect with exponential backoff
        if (reconnectTimer) clearTimeout(reconnectTimer);
        reconnectTimer = setTimeout(() => {
            connectWs(token);
        }, reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
    };

    ws.onerror = () => {
        // onclose will fire after onerror, so reconnect is handled there
    };

    ws.onmessage = (e) => {
        try {
            const event: WsEvent = JSON.parse(e.data);
            events.update((list) => [event, ...list].slice(0, 200));

            // Track runtime status
            if (event.type === 'RuntimeStatus') {
                runtimeRunning.set(event.running as boolean);
            }
        } catch {
            // Ignore malformed messages
        }
    };
}

/** Disconnect the WebSocket and stop reconnection attempts. */
export function disconnectWs() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    if (ws) {
        ws.close();
        ws = null;
    }
    connected.set(false);
    runtimeRunning.set(false);
}
