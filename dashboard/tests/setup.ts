import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

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

// Suppress console errors in tests unless explicitly needed
const originalConsoleError = console.error;
beforeEach(() => {
	console.error = vi.fn();
});
afterEach(() => {
	console.error = originalConsoleError;
});
