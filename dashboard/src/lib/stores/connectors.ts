import { writable, derived } from 'svelte/store';
import { api, resolveBaseUrl, type Connection } from '$lib/api';

// --- Writable stores ---

export const connections = writable<Connection[]>([]);
export const connectionsLoaded = writable(false);
export const linkingState = writable<'idle' | 'linking' | 'success' | 'error'>('idle');
export const linkError = writable<string | null>(null);

// --- Derived stores ---

export const activeGoogleDrive = derived(connections, ($conns) =>
	$conns.find((c) => c.connector_type === 'google_drive' && c.status === 'active') ?? null
);

export const expiredGoogleDrive = derived(connections, ($conns) =>
	$conns.find((c) => c.connector_type === 'google_drive' && c.status === 'expired') ?? null
);

// --- Actions ---

export async function loadConnections(): Promise<void> {
	try {
		const resp = await api.connectors.googleDrive.status();
		connections.set(resp.connections);
		connectionsLoaded.set(true);
	} catch {
		// Server may not have connector routes configured -- treat as empty.
		connections.set([]);
		connectionsLoaded.set(true);
	}
}

/**
 * Start the Google Drive OAuth link flow.
 * Opens a popup for the user to authorize, listens for postMessage callback.
 * Returns the new connection ID on success, or null on failure.
 */
export async function startLink(force?: boolean): Promise<number | null> {
	linkingState.set('linking');
	linkError.set(null);

	let resp: { authorization_url: string; state: string };
	try {
		resp = await api.connectors.googleDrive.link(force);
	} catch (e) {
		linkingState.set('error');
		linkError.set(e instanceof Error ? e.message : 'Failed to start link flow');
		return null;
	}

	// Open popup
	const popupRef = window.open(resp.authorization_url, 'tuitbot_connector', 'width=600,height=700');
	if (!popupRef) {
		linkingState.set('error');
		linkError.set(
			'Popup blocked by browser. Please allow popups for this site and try again.'
		);
		return null;
	}
	const popup: Window = popupRef;

	// Wait for postMessage or popup close
	return new Promise<number | null>((resolve) => {
		const expectedOrigin = resolveBaseUrl() || window.location.origin;
		let resolved = false;

		function cleanup() {
			window.removeEventListener('message', onMessage);
			clearInterval(closedCheck);
			clearTimeout(timeout);
		}

		function onMessage(event: MessageEvent) {
			// Validate origin -- accept both the expected base URL and current origin
			if (event.origin !== expectedOrigin && event.origin !== window.location.origin) {
				return;
			}
			if (event.data?.type !== 'connector_linked') {
				return;
			}

			resolved = true;
			cleanup();

			const connectionId = event.data.id as number;
			linkingState.set('success');
			loadConnections();
			resolve(connectionId);

			// Close the popup if still open
			try {
				popup.close();
			} catch {
				// Ignore cross-origin close errors
			}
		}

		window.addEventListener('message', onMessage);

		// Detect if user closes the popup manually
		const closedCheck = setInterval(() => {
			if (popup.closed && !resolved) {
				resolved = true;
				cleanup();
				linkingState.set('idle');
				resolve(null);
			}
		}, 1000);

		// 5-minute timeout
		const timeout = setTimeout(() => {
			if (!resolved) {
				resolved = true;
				cleanup();
				linkingState.set('error');
				linkError.set('Authorization timed out. Please try again.');
				try {
					popup.close();
				} catch {
					// Ignore
				}
				resolve(null);
			}
		}, 5 * 60 * 1000);
	});
}

/**
 * Disconnect a Google Drive connection by ID.
 */
export async function disconnectConnection(id: number): Promise<void> {
	try {
		await api.connectors.googleDrive.disconnect(id);
		await loadConnections();
	} catch (e) {
		linkError.set(e instanceof Error ? e.message : 'Failed to disconnect');
	}
}
