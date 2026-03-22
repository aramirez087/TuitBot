import '@testing-library/jest-dom/vitest';
import { vi, beforeEach, afterEach } from 'vitest';

// jsdom v29 + vitest: localStorage stub lacks Storage methods.
// Provide a full in-memory implementation so component code and tests
// can call getItem/setItem/removeItem/clear without runtime errors.
// Cleared between tests via afterEach so state doesn't bleed across cases.
const localStorageMock = (() => {
	let store: Record<string, string> = {};
	return {
		getItem: (key: string): string | null => store[key] ?? null,
		setItem: (key: string, value: string): void => { store[key] = String(value); },
		removeItem: (key: string): void => { delete store[key]; },
		clear: (): void => { store = {}; },
		get length(): number { return Object.keys(store).length; },
		key: (index: number): string | null => Object.keys(store)[index] ?? null,
		_reset: (): void => { store = {}; }
	};
})();
Object.defineProperty(window, 'localStorage', { value: localStorageMock, writable: false });

// Mock SvelteKit app modules
vi.mock('$app/navigation', () => ({
	goto: vi.fn(),
	invalidate: vi.fn(),
	invalidateAll: vi.fn(),
	preloadCode: vi.fn(),
	preloadData: vi.fn(),
	pushState: vi.fn(),
	replaceState: vi.fn()
}));

vi.mock('$app/stores', () => ({
	page: {
		subscribe: vi.fn((cb) => {
			cb({
				url: new URL('http://localhost/'),
				params: {},
				route: { id: null },
				status: 200,
				error: null,
				data: {},
				state: {},
				form: undefined
			});
			return () => {};
		})
	},
	navigating: {
		subscribe: vi.fn((cb) => {
			cb(null);
			return () => {};
		})
	},
	updated: {
		subscribe: vi.fn((cb) => {
			cb(false);
			return () => {};
		})
	}
}));

vi.mock('$app/environment', () => ({
	browser: true,
	dev: true,
	building: false,
	version: 'test'
}));

// jsdom lacks the Web Animations API. Svelte transitions (fly, fade, etc.)
// call element.animate() which doesn't exist in jsdom. Provide a stub that
// immediately resolves so out-transitions complete and elements are removed.
if (typeof Element.prototype.animate !== 'function') {
	Element.prototype.animate = function () {
		const anim = {
			cancel: () => {},
			finish: () => {},
			finished: Promise.resolve(),
			onfinish: null as (() => void) | null,
			playState: 'finished',
			currentTime: null,
			effect: null,
			id: '',
			pending: false,
			ready: Promise.resolve(),
			startTime: null,
			timeline: null,
			addEventListener: () => {},
			removeEventListener: () => {},
			dispatchEvent: () => true,
			persist: () => {},
			play: () => {},
			pause: () => {},
			reverse: () => {},
			updatePlaybackRate: () => {},
			commitStyles: () => {},
			oncancel: null,
			onremove: null,
			playbackRate: 1,
			replaceState: 'active',
		};
		// Trigger onfinish synchronously so Svelte removes out-transitioned elements
		queueMicrotask(() => { anim.onfinish?.(); });
		return anim as unknown as Animation;
	};
}

// Suppress console errors in tests unless explicitly needed
const originalConsoleError = console.error;
beforeEach(() => {
	console.error = vi.fn();
	localStorageMock._reset();
});
afterEach(() => {
	console.error = originalConsoleError;
});
