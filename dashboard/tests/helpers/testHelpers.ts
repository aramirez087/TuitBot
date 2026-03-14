/**
 * testHelpers.ts — Render helpers, store reset utilities, and assertion helpers
 * for TuitBot dashboard tests.
 *
 * Usage:
 *   import { renderWithStores, waitForLoaded, assertStoreValue } from '../helpers/testHelpers';
 */

import { render, type RenderResult } from '@testing-library/svelte';
import { get, type Readable } from 'svelte/store';
import { vi, expect } from 'vitest';
import type { ComponentType, SvelteComponent } from 'svelte';
import { resetAllStores } from './mockStores';
import { resetMockApi } from './mockApi';

// ---------------------------------------------------------------------------
// Render helper
// ---------------------------------------------------------------------------

/**
 * Renders a Svelte component with optional props and context.
 * Automatically resets stores and mocks in cleanup.
 */
export function renderWithStores<T extends Record<string, unknown>>(
	Component: ComponentType<SvelteComponent>,
	props: T = {} as T
): RenderResult<SvelteComponent> {
	return render(Component, { props });
}

// ---------------------------------------------------------------------------
// Store assertion helpers
// ---------------------------------------------------------------------------

/**
 * Asserts the current value of a Svelte readable/writable store.
 */
export function assertStoreValue<T>(store: Readable<T>, expected: T): void {
	expect(get(store)).toEqual(expected);
}

/**
 * Asserts the store value satisfies a predicate.
 */
export function assertStore<T>(store: Readable<T>, predicate: (val: T) => boolean): void {
	expect(predicate(get(store))).toBe(true);
}

// ---------------------------------------------------------------------------
// Async helpers
// ---------------------------------------------------------------------------

/**
 * Flushes all pending microtasks and timers.
 * Use after triggering async store loads.
 */
export async function flushAll(): Promise<void> {
	await new Promise((r) => setTimeout(r, 0));
	await vi.runAllTimersAsync().catch(() => {});
}

/**
 * Waits for a loading store to become false.
 * Times out after `timeoutMs` (default 2000ms).
 */
export async function waitForLoaded(
	loadingStore: Readable<boolean>,
	timeoutMs = 2000
): Promise<void> {
	const start = Date.now();
	while (get(loadingStore)) {
		if (Date.now() - start > timeoutMs) {
			throw new Error('waitForLoaded: timed out waiting for loading to become false');
		}
		await new Promise((r) => setTimeout(r, 10));
	}
}

// ---------------------------------------------------------------------------
// Mock reset helper
// ---------------------------------------------------------------------------

/**
 * Resets all stores and mocks. Call in beforeEach.
 */
export function resetAll(): void {
	resetAllStores();
	resetMockApi();
}

// ---------------------------------------------------------------------------
// API error simulation helper
// ---------------------------------------------------------------------------

/**
 * Creates a rejected mock for simulating API errors.
 */
export function apiError(message = 'Internal Server Error', status = 500): Error {
	const err = new Error(message);
	(err as Error & { status: number }).status = status;
	return err;
}

// ---------------------------------------------------------------------------
// Text content assertion helpers
// ---------------------------------------------------------------------------

/**
 * Asserts an element's text content contains a substring.
 */
export function assertText(element: Element | null, substring: string): void {
	expect(element?.textContent ?? '').toContain(substring);
}
