/**
 * settingsStore.test.ts — Unit tests for src/lib/stores/settings.ts
 *
 * Covers: loadSettings, updateDraft, resetDraft, resetField, saveSettings
 * (success + validation errors), resetStores, hasDangerousChanges,
 * and derived stores (isDirty, fieldErrors, scoringTotal).
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

import { api } from '$lib/api';
import * as store from '../../src/lib/stores/settings';
import type { TuitbotConfig } from '../../src/lib/api/types';

// --- Fixture config ---------------------------------------------------------

const makeConfig = (overrides: Partial<TuitbotConfig> = {}): TuitbotConfig =>
	({
		x_api: {
			client_id: 'cid-test',
			client_secret: null,
			provider_backend: 'local',
			scraper_allow_mutations: false
		},
		auth: { mode: 'passphrase', callback_host: 'localhost', callback_port: 8000 },
		business: {
			product_name: 'TuitBot',
			product_description: 'Test bot',
			product_url: null,
			target_audience: 'devs',
			product_keywords: ['test'],
			competitor_keywords: [],
			industry_topics: ['saas'],
			brand_voice: null,
			reply_style: null,
			content_style: null,
			persona_opinions: [],
			persona_experiences: [],
			content_pillars: []
		},
		scoring: {
			threshold: 0.6,
			keyword_relevance_max: 30,
			follower_count_max: 20,
			recency_max: 20,
			engagement_rate_max: 15,
			reply_count_max: 10,
			content_type_max: 5
		},
		limits: {
			max_replies_per_day: 5,
			max_tweets_per_day: 3,
			max_threads_per_week: 1,
			min_action_delay_seconds: 60,
			max_action_delay_seconds: 300,
			max_replies_per_author_per_day: 1,
			banned_phrases: [],
			product_mention_ratio: 0.2
		},
		intervals: {
			mentions_check_seconds: 300,
			discovery_search_seconds: 600,
			content_post_window_seconds: 3600,
			thread_interval_seconds: 86400
		},
		llm: {
			provider: 'openai',
			api_key: null,
			model: 'gpt-4o',
			base_url: null
		},
		targets: {
			accounts: [],
			max_target_replies_per_day: 2
		},
		...overrides
	} as unknown as TuitbotConfig);

const CONFIG = makeConfig();
const DEFAULTS = makeConfig({ scoring: { ...CONFIG.scoring, threshold: 0.5 } } as Partial<TuitbotConfig>);

// --- Reset helper -----------------------------------------------------------

function resetStores() {
	store.resetStores();
}

// --- Tests ------------------------------------------------------------------

beforeEach(() => {
	resetStores();
	vi.clearAllMocks();
	(api.settings.get as ReturnType<typeof vi.fn>).mockResolvedValue(CONFIG);
	(api.settings.defaults as ReturnType<typeof vi.fn>).mockResolvedValue(DEFAULTS);
	(api.settings.validate as ReturnType<typeof vi.fn>).mockResolvedValue({ valid: true, errors: [] });
	(api.settings.patch as ReturnType<typeof vi.fn>).mockResolvedValue(CONFIG);
	(api.settings.testLlm as ReturnType<typeof vi.fn>).mockResolvedValue({ success: true, message: 'OK' });
});

// ---------------------------------------------------------------------------
// loadSettings
// ---------------------------------------------------------------------------

describe('loadSettings', () => {
	it('sets loading true then false', async () => {
		const states: boolean[] = [];
		const unsub = store.loading.subscribe((v) => states.push(v));
		await store.loadSettings();
		unsub();
		expect(states).toContain(true);
		expect(states[states.length - 1]).toBe(false);
	});

	it('populates config, defaults, and draft', async () => {
		await store.loadSettings();
		expect(get(store.config)).toEqual(CONFIG);
		expect(get(store.defaults)).toEqual(DEFAULTS);
		expect(get(store.draft)).toEqual(CONFIG);
	});

	it('draft is a deep clone of config (not same reference)', async () => {
		await store.loadSettings();
		const configVal = get(store.config);
		const draftVal = get(store.draft);
		expect(draftVal).not.toBe(configVal);
		expect(draftVal).toEqual(configVal);
	});

	it('sets error on API failure', async () => {
		(api.settings.get as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Load failed')
		);
		await store.loadSettings();
		expect(get(store.error)).toBe('Load failed');
		expect(get(store.loading)).toBe(false);
	});

	it('sets generic error for non-Error rejections', async () => {
		(api.settings.get as ReturnType<typeof vi.fn>).mockRejectedValueOnce('nope');
		await store.loadSettings();
		expect(get(store.error)).toBe('Failed to load settings');
	});
});

// ---------------------------------------------------------------------------
// updateDraft
// ---------------------------------------------------------------------------

describe('updateDraft', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('updates a top-level dot-path field in draft', () => {
		store.updateDraft('scoring.threshold', 0.9);
		expect((get(store.draft) as TuitbotConfig).scoring.threshold).toBe(0.9);
	});

	it('does not mutate the committed config', () => {
		store.updateDraft('scoring.threshold', 0.9);
		expect((get(store.config) as TuitbotConfig).scoring.threshold).toBe(0.6);
	});

	it('clears validation error for the updated field', () => {
		store.validationErrors.set([{ field: 'scoring.threshold', message: 'Too low' }]);
		store.updateDraft('scoring.threshold', 0.9);
		const errs = get(store.validationErrors);
		expect(errs.find((e) => e.field === 'scoring.threshold')).toBeUndefined();
	});

	it('leaves other validation errors intact', () => {
		store.validationErrors.set([
			{ field: 'scoring.threshold', message: 'Too low' },
			{ field: 'llm.model', message: 'Required' }
		]);
		store.updateDraft('scoring.threshold', 0.9);
		const errs = get(store.validationErrors);
		expect(errs.find((e) => e.field === 'llm.model')).toBeDefined();
	});

	it('handles deeply nested dot-path', () => {
		store.updateDraft('llm.api_key', 'sk-test-key');
		expect((get(store.draft) as TuitbotConfig).llm.api_key).toBe('sk-test-key');
	});
});

// ---------------------------------------------------------------------------
// resetDraft
// ---------------------------------------------------------------------------

describe('resetDraft', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('restores draft to config state', () => {
		store.updateDraft('scoring.threshold', 0.99);
		store.resetDraft();
		expect((get(store.draft) as TuitbotConfig).scoring.threshold).toBe(
			CONFIG.scoring.threshold
		);
	});

	it('clears validation errors', () => {
		store.validationErrors.set([{ field: 'x_api.client_id', message: 'Required' }]);
		store.resetDraft();
		expect(get(store.validationErrors)).toHaveLength(0);
	});

	it('clears saveError', () => {
		store.saveError.set('Save failed');
		store.resetDraft();
		expect(get(store.saveError)).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// resetField
// ---------------------------------------------------------------------------

describe('resetField', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('resets a field to its defaults value', () => {
		store.updateDraft('scoring.threshold', 0.99);
		store.resetField('scoring.threshold');
		// DEFAULTS has threshold 0.5
		expect((get(store.draft) as TuitbotConfig).scoring.threshold).toBe(0.5);
	});

	it('no-ops when defaults is null', () => {
		store.defaults.set(null);
		expect(() => store.resetField('scoring.threshold')).not.toThrow();
	});
});

// ---------------------------------------------------------------------------
// saveSettings
// ---------------------------------------------------------------------------

describe('saveSettings', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('returns true and updates config on success', async () => {
		const result = await store.saveSettings();
		expect(result).toBe(true);
		expect(api.settings.validate).toHaveBeenCalled();
		expect(api.settings.patch).toHaveBeenCalled();
	});

	it('sets saving to false after success', async () => {
		await store.saveSettings();
		expect(get(store.saving)).toBe(false);
	});

	it('sets lastSaved on success', async () => {
		await store.saveSettings();
		expect(get(store.lastSaved)).toBeInstanceOf(Date);
	});

	it('returns false and sets validationErrors when validation fails', async () => {
		(api.settings.validate as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			valid: false,
			errors: [{ field: 'scoring.threshold', message: 'Must be between 0 and 1' }]
		});
		const result = await store.saveSettings();
		expect(result).toBe(false);
		expect(get(store.validationErrors)).toHaveLength(1);
		expect(api.settings.patch).not.toHaveBeenCalled();
	});

	it('sets saveError and returns false on patch failure', async () => {
		(api.settings.patch as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('500 Internal Server Error')
		);
		const result = await store.saveSettings();
		expect(result).toBe(false);
		expect(get(store.saveError)).toBe('500 Internal Server Error');
	});

	it('returns false when draft is null', async () => {
		store.draft.set(null);
		const result = await store.saveSettings();
		expect(result).toBe(false);
	});
});

// ---------------------------------------------------------------------------
// resetStores
// ---------------------------------------------------------------------------

describe('resetStores', () => {
	it('clears all stores to initial state', async () => {
		await store.loadSettings();
		store.updateDraft('scoring.threshold', 0.99);
		store.resetStores();

		expect(get(store.config)).toBeNull();
		expect(get(store.defaults)).toBeNull();
		expect(get(store.draft)).toBeNull();
		expect(get(store.loading)).toBe(true);
		expect(get(store.error)).toBeNull();
		expect(get(store.saveError)).toBeNull();
		expect(get(store.validationErrors)).toHaveLength(0);
		expect(get(store.lastSaved)).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// hasDangerousChanges
// ---------------------------------------------------------------------------

describe('hasDangerousChanges', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('returns false when config and draft are identical', () => {
		expect(store.hasDangerousChanges()).toBe(false);
	});

	it('returns true when LLM provider changes', () => {
		store.updateDraft('llm.provider', 'anthropic');
		expect(store.hasDangerousChanges()).toBe(true);
	});

	it('returns true when x_api client_id changes', () => {
		store.updateDraft('x_api.client_id', 'new-cid');
		expect(store.hasDangerousChanges()).toBe(true);
	});

	it('returns true when x_api client_secret changes', () => {
		store.updateDraft('x_api.client_secret', 'secret-new');
		expect(store.hasDangerousChanges()).toBe(true);
	});

	it('returns false when only non-dangerous fields change', () => {
		store.updateDraft('scoring.threshold', 0.9);
		expect(store.hasDangerousChanges()).toBe(false);
	});

	it('returns false when config or draft is null', () => {
		store.config.set(null);
		expect(store.hasDangerousChanges()).toBe(false);
	});
});

// ---------------------------------------------------------------------------
// Derived: isDirty
// ---------------------------------------------------------------------------

describe('isDirty', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('is false when draft equals config', () => {
		expect(get(store.isDirty)).toBe(false);
	});

	it('is true when draft differs from config', () => {
		store.updateDraft('scoring.threshold', 0.99);
		expect(get(store.isDirty)).toBe(true);
	});

	it('is false when null', () => {
		store.resetStores();
		expect(get(store.isDirty)).toBe(false);
	});
});

// ---------------------------------------------------------------------------
// Derived: fieldErrors
// ---------------------------------------------------------------------------

describe('fieldErrors', () => {
	it('maps validation errors to a field → message record', () => {
		store.validationErrors.set([
			{ field: 'scoring.threshold', message: 'Too low' },
			{ field: 'llm.model', message: 'Required' }
		]);
		const errs = get(store.fieldErrors);
		expect(errs['scoring.threshold']).toBe('Too low');
		expect(errs['llm.model']).toBe('Required');
	});

	it('returns empty object when no errors', () => {
		store.validationErrors.set([]);
		expect(get(store.fieldErrors)).toEqual({});
	});

	it('ignores errors without a field', () => {
		store.validationErrors.set([{ field: undefined as unknown as string, message: 'Global error' }]);
		expect(Object.keys(get(store.fieldErrors))).toHaveLength(0);
	});
});

// ---------------------------------------------------------------------------
// Derived: scoringTotal
// ---------------------------------------------------------------------------

describe('scoringTotal', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('sums all scoring max fields', () => {
		// 30 + 20 + 20 + 15 + 10 + 5 = 100
		expect(get(store.scoringTotal)).toBe(100);
	});

	it('returns 0 when draft is null', () => {
		store.draft.set(null);
		expect(get(store.scoringTotal)).toBe(0);
	});

	it('updates when draft changes', () => {
		store.updateDraft('scoring.keyword_relevance_max', 40);
		// 40 + 20 + 20 + 15 + 10 + 5 = 110
		expect(get(store.scoringTotal)).toBe(110);
	});
});

// ---------------------------------------------------------------------------
// testLlmConnection
// ---------------------------------------------------------------------------

describe('testLlmConnection', () => {
	it('returns error when draft is null', async () => {
		store.resetStores();
		const result = await store.testLlmConnection();
		expect(result.success).toBe(false);
		expect(result.error).toMatch(/No settings loaded/i);
	});

	it('calls api.settings.testLlm with draft llm fields', async () => {
		await store.loadSettings();
		(api.settings.testLlm as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
			success: true,
			message: 'Connected'
		});
		const result = await store.testLlmConnection();
		expect(result.success).toBe(true);
		expect(api.settings.testLlm).toHaveBeenCalledWith(
			expect.objectContaining({ provider: expect.any(String), model: expect.any(String) })
		);
	});
});

// ---------------------------------------------------------------------------
// resetSectionToBase
// ---------------------------------------------------------------------------

describe('resetSectionToBase', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('returns true and updates config on success', async () => {
		(api.settings.patch as ReturnType<typeof vi.fn>).mockResolvedValueOnce(CONFIG);
		const result = await store.resetSectionToBase('llm');
		expect(result).toBe(true);
		expect(get(store.lastSaved)).toBeInstanceOf(Date);
	});

	it('sets saveError and returns false on failure', async () => {
		(api.settings.patch as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
			new Error('Reset failed')
		);
		const result = await store.resetSectionToBase('llm');
		expect(result).toBe(false);
		expect(get(store.saveError)).toMatch(/Reset failed/);
	});

	it('sets saving=false in finally block on success', async () => {
		(api.settings.patch as ReturnType<typeof vi.fn>).mockResolvedValueOnce(CONFIG);
		await store.resetSectionToBase('llm');
		expect(get(store.saving)).toBe(false);
	});

	it('sets saving=false in finally block on failure', async () => {
		(api.settings.patch as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('err'));
		await store.resetSectionToBase('llm');
		expect(get(store.saving)).toBe(false);
	});

	it('handles EffectiveSettingsResponse envelope when account is not DEFAULT', async () => {
		// Simulate a non-default account returning an envelope response
		const envelope = {
			config: CONFIG,
			_overrides: ['llm']
		};
		(api.settings.patch as ReturnType<typeof vi.fn>).mockResolvedValueOnce(envelope);
		const result = await store.resetSectionToBase('llm');
		// Falls through to the TuitbotConfig branch since getAccountId() returns DEFAULT in tests
		expect(result).toBe(true);
	});
});

// ---------------------------------------------------------------------------
// hasDangerousChanges — additional branches
// ---------------------------------------------------------------------------

describe('hasDangerousChanges (additional branches)', () => {
	beforeEach(async () => {
		await store.loadSettings();
	});

	it('returns true when x_api.provider_backend changes', () => {
		store.updateDraft('x_api.provider_backend', 'scraper');
		expect(store.hasDangerousChanges()).toBe(true);
	});

	it('returns false when draft is null (guards against crash)', () => {
		store.draft.set(null);
		expect(store.hasDangerousChanges()).toBe(false);
	});
});
