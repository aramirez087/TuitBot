/**
 * analyticsSyncPrompt.test.ts — Unit tests for analytics sync prompt
 * state management in src/lib/stores/settings.ts.
 *
 * Covers: analyticsSyncPromptDismissed, dismissAnalyticsSyncPrompt,
 * resetAnalyticsSyncPrompt, pendingAnalyticsSyncPrompt,
 * setPendingAnalyticsSyncPrompt, clearPendingAnalyticsSyncPrompt.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

// --- Mocks (hoisted) --------------------------------------------------------

vi.mock('$lib/api', () => ({
	api: {
		settings: {
			get: vi.fn(),
			patch: vi.fn(),
			validate: vi.fn(),
			defaults: vi.fn(),
			testLlm: vi.fn()
		}
	}
}));

vi.mock('$lib/api/http', () => ({
	getAccountId: vi.fn().mockReturnValue('00000000-0000-0000-0000-000000000000')
}));

// --- Imports after mocks ----------------------------------------------------

import {
	analyticsSyncPromptDismissed,
	dismissAnalyticsSyncPrompt,
	resetAnalyticsSyncPrompt,
	pendingAnalyticsSyncPrompt,
	setPendingAnalyticsSyncPrompt,
	clearPendingAnalyticsSyncPrompt
} from '../../src/lib/stores/settings';

// --- Setup ------------------------------------------------------------------

beforeEach(() => {
	localStorage.clear();
	analyticsSyncPromptDismissed.set(false);
	pendingAnalyticsSyncPrompt.set(false);
});

// --- Tests ------------------------------------------------------------------

describe('analyticsSyncPromptDismissed', () => {
	it('defaults to false when localStorage is empty', () => {
		expect(get(analyticsSyncPromptDismissed)).toBe(false);
	});

	it('dismissAnalyticsSyncPrompt sets store and localStorage', () => {
		dismissAnalyticsSyncPrompt();
		expect(get(analyticsSyncPromptDismissed)).toBe(true);
		expect(localStorage.getItem('tuitbot:analytics-sync-prompt-dismissed')).toBe('true');
	});

	it('resetAnalyticsSyncPrompt clears store and localStorage', () => {
		dismissAnalyticsSyncPrompt();
		expect(get(analyticsSyncPromptDismissed)).toBe(true);

		resetAnalyticsSyncPrompt();
		expect(get(analyticsSyncPromptDismissed)).toBe(false);
		expect(localStorage.getItem('tuitbot:analytics-sync-prompt-dismissed')).toBeNull();
	});
});

describe('pendingAnalyticsSyncPrompt', () => {
	it('defaults to false when localStorage is empty', () => {
		expect(get(pendingAnalyticsSyncPrompt)).toBe(false);
	});

	it('setPendingAnalyticsSyncPrompt sets store and localStorage', () => {
		setPendingAnalyticsSyncPrompt();
		expect(get(pendingAnalyticsSyncPrompt)).toBe(true);
		expect(localStorage.getItem('tuitbot:pending-analytics-sync-prompt')).toBe('true');
	});

	it('clearPendingAnalyticsSyncPrompt clears store and localStorage', () => {
		setPendingAnalyticsSyncPrompt();
		expect(get(pendingAnalyticsSyncPrompt)).toBe(true);

		clearPendingAnalyticsSyncPrompt();
		expect(get(pendingAnalyticsSyncPrompt)).toBe(false);
		expect(localStorage.getItem('tuitbot:pending-analytics-sync-prompt')).toBeNull();
	});
});

describe('prompt lifecycle', () => {
	it('dismiss then reset produces fresh prompt opportunity', () => {
		dismissAnalyticsSyncPrompt();
		expect(get(analyticsSyncPromptDismissed)).toBe(true);

		resetAnalyticsSyncPrompt();
		expect(get(analyticsSyncPromptDismissed)).toBe(false);
	});

	it('pending and dismissed are independent', () => {
		setPendingAnalyticsSyncPrompt();
		dismissAnalyticsSyncPrompt();
		expect(get(pendingAnalyticsSyncPrompt)).toBe(true);
		expect(get(analyticsSyncPromptDismissed)).toBe(true);

		clearPendingAnalyticsSyncPrompt();
		expect(get(pendingAnalyticsSyncPrompt)).toBe(false);
		expect(get(analyticsSyncPromptDismissed)).toBe(true);
	});
});
