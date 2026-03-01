/**
 * Persistent UI state using @tauri-apps/plugin-store.
 * Falls back to localStorage when not running in Tauri (browser-only dev).
 */

let storeInstance: any = null;
let useFallback = false;

async function getStore() {
	if (storeInstance) return storeInstance;
	if (useFallback) return null;

	try {
		const { load } = await import('@tauri-apps/plugin-store');
		storeInstance = await load('ui-state.json', { autoSave: true, defaults: {} } as any);
		return storeInstance;
	} catch {
		useFallback = true;
		return null;
	}
}

export async function persistGet<T>(key: string, fallback: T): Promise<T> {
	const store = await getStore();
	if (store) {
		try {
			const value = await store.get(key);
			return value !== null && value !== undefined ? (value as T) : fallback;
		} catch {
			return fallback;
		}
	}

	// localStorage fallback (browser-only dev)
	try {
		const raw = localStorage.getItem(`tuitbot:ui:${key}`);
		if (raw === null) return fallback;
		return JSON.parse(raw) as T;
	} catch {
		return fallback;
	}
}

export async function persistSet<T>(key: string, value: T): Promise<void> {
	const store = await getStore();
	if (store) {
		try {
			await store.set(key, value);
			await store.save();
		} catch {
			// Silently ignore persistence failures.
		}
		return;
	}

	// localStorage fallback (browser-only dev)
	try {
		localStorage.setItem(`tuitbot:ui:${key}`, JSON.stringify(value));
	} catch {
		// Silently ignore quota errors.
	}
}
