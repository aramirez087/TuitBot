import { writable, get } from 'svelte/store';
import { api, setAuthMode, setCsrfToken } from '$lib/api';

export type AuthMode = 'tauri' | 'web' | 'none';

/** Current authentication mode. */
export const authMode = writable<AuthMode>('none');

/** Whether the user is authenticated. */
export const isAuthenticated = writable(false);

/** Login with a passphrase (web/LAN mode). */
export async function login(passphrase: string): Promise<void> {
	const result = await api.auth.login(passphrase);
	setCsrfToken(result.csrf_token);
	setAuthMode('cookie');
	authMode.set('web');
	isAuthenticated.set(true);
}

/** Logout (web/LAN mode). */
export async function logout(): Promise<void> {
	await api.auth.logout();
	setCsrfToken('');
	setAuthMode('bearer');
	authMode.set('none');
	isAuthenticated.set(false);
}

/** Check if there's an existing valid session (cookie-based). */
export async function checkAuth(): Promise<boolean> {
	try {
		const result = await api.auth.status();
		if (result.authenticated && result.csrf_token) {
			setCsrfToken(result.csrf_token);
			setAuthMode('cookie');
			authMode.set('web');
			isAuthenticated.set(true);
			return true;
		}
	} catch {
		// Server not reachable — not authenticated.
	}
	return false;
}
