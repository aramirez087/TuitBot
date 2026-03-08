/**
 * Reactive store for the home surface preference.
 * Persists via Tauri plugin-store (production) or localStorage (browser dev).
 */

import { writable, derived } from 'svelte/store';
import { persistGet, persistSet } from './persistence';

export type HomeSurface = 'drafts' | 'analytics';

const PREF_KEY = 'home_surface';

const _surface = writable<HomeSurface>('drafts');
const _loaded = writable(false);

/** Read-only store with current home surface value. */
export const homeSurface = { subscribe: _surface.subscribe };

/** True once the persisted preference has been loaded. */
export const homeSurfaceReady = derived(_loaded, ($l) => $l);

/** Load the persisted preference. Call once during app mount. */
export async function loadHomeSurface(): Promise<void> {
	let value = await persistGet<string>(PREF_KEY, 'drafts');
	// Migrate legacy 'composer' preference to 'drafts'
	if (value === 'composer') {
		value = 'drafts';
		await persistSet(PREF_KEY, value);
	}
	_surface.set(value as HomeSurface);
	_loaded.set(true);
}

/** Update the preference and persist it. */
export async function setHomeSurface(value: HomeSurface): Promise<void> {
	_surface.set(value);
	await persistSet(PREF_KEY, value);
}
