import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import { api, setAccountId } from '$lib/api';
import type { Account } from '$lib/api';

export type { Account };

const STORAGE_KEY = 'tuitbot-account-id';
const DEFAULT_ACCOUNT_ID = '00000000-0000-0000-0000-000000000000';

function getPersistedAccountId(): string {
	if (browser) {
		return localStorage.getItem(STORAGE_KEY) || DEFAULT_ACCOUNT_ID;
	}
	return DEFAULT_ACCOUNT_ID;
}

export const currentAccountId = writable<string>(getPersistedAccountId());
export const accounts = writable<Account[]>([]);

export const currentAccount = derived(
	[accounts, currentAccountId],
	([$accounts, $currentAccountId]) =>
		$accounts.find((a) => a.id === $currentAccountId) ?? null
);

/** Fetch all accounts from the API and update the store. */
export async function fetchAccounts(): Promise<void> {
	try {
		const list = await api.accounts.list();
		accounts.set(list);
	} catch {
		// If accounts endpoint fails (e.g., single-user setup), use empty list.
		accounts.set([]);
	}
}

/** Switch to a different account. */
export function switchAccount(id: string): void {
	currentAccountId.set(id);
	setAccountId(id);
	if (browser) {
		localStorage.setItem(STORAGE_KEY, id);
	}
}

/** Initialize account context on app mount. */
export function initAccounts(): void {
	const id = getPersistedAccountId();
	setAccountId(id);
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
