import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import { api, setAccountId } from '$lib/api';
import { clearEvents } from '$lib/stores/websocket';
import type { Account } from '$lib/api';

export type { Account };

/** Custom event name dispatched on account switch so pages can refetch. */
export const ACCOUNT_SWITCHED_EVENT = 'tuitbot:account-switched';

const STORAGE_KEY = 'tuitbot-account-id';
const DEFAULT_ACCOUNT_ID = '00000000-0000-0000-0000-000000000000';

function getPersistedAccountId(): string {
	if (browser) {
		return localStorage.getItem(STORAGE_KEY) || DEFAULT_ACCOUNT_ID;
	}
	return DEFAULT_ACCOUNT_ID;
}

export type BootstrapState = 'loading' | 'ready' | 'error';

export const currentAccountId = writable<string>(getPersistedAccountId());
export const accounts = writable<Account[]>([]);
export const bootstrapState = writable<BootstrapState>('loading');

export const currentAccount = derived(
	[accounts, currentAccountId],
	([$accounts, $currentAccountId]) =>
		$accounts.find((a) => a.id === $currentAccountId) ?? null
);

/** Fetch all accounts from the API and update the store. */
export async function fetchAccounts(): Promise<Account[]> {
	try {
		const list = await api.accounts.list();
		accounts.set(list);
		return list;
	} catch {
		accounts.set([]);
		return [];
	}
}

/** Switch to a different account, flush stale state, and notify pages. */
export function switchAccount(id: string): void {
	currentAccountId.set(id);
	setAccountId(id);
	if (browser) {
		localStorage.setItem(STORAGE_KEY, id);
	}
	clearEvents();
	if (browser) {
		window.dispatchEvent(new CustomEvent(ACCOUNT_SWITCHED_EVENT));
	}
}

/**
 * Initialize account context on app mount.
 *
 * Fetches the account list, validates the persisted account ID against it,
 * and falls back to default or first active account if stale.
 */
export async function initAccounts(): Promise<void> {
	bootstrapState.set('loading');
	try {
		const persistedId = getPersistedAccountId();
		setAccountId(persistedId);

		const list = await fetchAccounts();

		if (list.length === 0) {
			// No accounts returned — keep persisted ID (backward compat).
			bootstrapState.set('ready');
			return;
		}

		const valid = list.some((a) => a.id === persistedId);
		if (!valid) {
			// Persisted ID is stale — fall back to default or first account.
			const fallback =
				list.find((a) => a.id === DEFAULT_ACCOUNT_ID) ?? list[0];
			switchAccount(fallback.id);
		}

		bootstrapState.set('ready');
	} catch {
		// On error, mark ready anyway so the app renders with default account.
		bootstrapState.set('ready');
	}
}

/** Sync X profile data (avatar, display name) for the current account. */
export async function syncCurrentProfile(): Promise<void> {
	const id = getPersistedAccountId();
	try {
		const updated = await api.accounts.syncProfile(id);
		accounts.update((list) =>
			list.map((a) => (a.id === updated.id ? updated : a))
		);
	} catch {
		// Non-critical — profile data is optional.
	}
}
