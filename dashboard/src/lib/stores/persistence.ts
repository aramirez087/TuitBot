/**
 * Persistent UI state using @tauri-apps/plugin-store.
 * Gracefully falls back to in-memory when not running in Tauri.
 */

let storeInstance: any = null;

async function getStore() {
	if (storeInstance) return storeInstance;

	try {
		const { load } = await import('@tauri-apps/plugin-store');
		storeInstance = await load('ui-state.json', { autoSave: true, defaults: {} } as any);
		return storeInstance;
	} catch {
		// Not in Tauri — return null, callers handle gracefully.
		return null;
	}
}

export async function persistGet<T>(key: string, fallback: T): Promise<T> {
	const store = await getStore();
	if (!store) return fallback;

	try {
		const value = await store.get(key);
		return value !== null && value !== undefined ? (value as T) : fallback;
	} catch {
		return fallback;
	}
}

export async function persistSet<T>(key: string, value: T): Promise<void> {
	const store = await getStore();
	if (!store) return;

	try {
		await store.set(key, value);
		await store.save();
	} catch {
		// Silently ignore persistence failures.
	}
}
