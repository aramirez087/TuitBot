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
let consecutiveErrors = 0;

/** Send a native notification if available and the app is in the background. */
async function sendNativeNotification(title: string, body: string) {
    if (typeof document === 'undefined' || !document.hidden) return;

    try {
        const { isPermissionGranted, requestPermission, sendNotification } =
            await import('@tauri-apps/plugin-notification');

        let permitted = await isPermissionGranted();
        if (!permitted) {
            const result = await requestPermission();
            permitted = result === 'granted';
        }
        if (permitted) {
            sendNotification({ title, body });
        }
    } catch {
        // Not in Tauri context — skip notifications.
    }
}

/**
 * Connect to the tuitbot-server WebSocket.
 * Automatically reconnects with exponential backoff on disconnect.
 *
 * If `token` is provided, authenticates via query parameter (Tauri/API mode).
 * If omitted, the server authenticates via the session cookie (web/LAN mode).
 */
function resolveWsBase(): string {
    if (typeof window === 'undefined') return 'ws://localhost:3001';
    if ('__TAURI_INTERNALS__' in window || window.location.port === '5173') {
        return 'ws://localhost:3001';
    }
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${protocol}//${window.location.host}`;
}

export function connectWs(token?: string) {
    if (ws) {
        ws.close();
    }

    const wsBase = resolveWsBase();
    const url = token
        ? `${wsBase}/api/ws?token=${token}`
        : `${wsBase}/api/ws`;
    ws = new WebSocket(url);

    ws.onopen = () => {
        connected.set(true);
        reconnectDelay = 1000; // Reset backoff on successful connect
        consecutiveErrors = 0;
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

            // Native notifications when app is in background
            if (event.type === 'ApprovalQueued') {
                sendNativeNotification('Tuitbot', 'New item pending approval');
                consecutiveErrors = 0;
            } else if (event.type === 'FollowerUpdate') {
                const count = event.count as number;
                if (count > 0 && count % 100 === 0) {
                    sendNativeNotification('Tuitbot', `Follower milestone: ${count} followers!`);
                }
                consecutiveErrors = 0;
            } else if (event.type === 'Error') {
                consecutiveErrors++;
                if (consecutiveErrors >= 3) {
                    sendNativeNotification('Tuitbot', 'Multiple automation errors detected');
                    consecutiveErrors = 0;
                }
            } else {
                consecutiveErrors = 0;
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
