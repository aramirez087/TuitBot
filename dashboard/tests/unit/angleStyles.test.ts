/**
 * angleStyles.test.ts — Unit tests for angle type labels, evidence type config, and citation truncation.
 */

import { describe, it, expect } from 'vitest';
import { getAngleTypeLabel, getEvidenceTypeConfig, truncateCitation } from '$lib/utils/angleStyles';

describe('getAngleTypeLabel', () => {
	it('maps "story" to "Story"', () => {
		expect(getAngleTypeLabel('story')).toBe('Story');
	});

	it('maps "listicle" to "Listicle"', () => {
		expect(getAngleTypeLabel('listicle')).toBe('Listicle');
	});

	it('maps "hot_take" to "Hot Take"', () => {
		expect(getAngleTypeLabel('hot_take')).toBe('Hot Take');
	});

	it('falls back to title-cased version for unknown types', () => {
		expect(getAngleTypeLabel('deep_dive')).toBe('Deep Dive');
	});

	it('handles single-word unknown type', () => {
		expect(getAngleTypeLabel('rant')).toBe('Rant');
	});
});

describe('getEvidenceTypeConfig', () => {
	it('returns warning config for contradiction', () => {
		const cfg = getEvidenceTypeConfig('contradiction');
		expect(cfg.label).toBe('Contradiction');
		expect(cfg.cssVar).toBe('--color-warning');
	});

	it('returns accent config for data_point', () => {
		const cfg = getEvidenceTypeConfig('data_point');
		expect(cfg.label).toBe('Data Point');
		expect(cfg.cssVar).toBe('--color-accent');
	});

	it('returns success config for aha_moment', () => {
		const cfg = getEvidenceTypeConfig('aha_moment');
		expect(cfg.label).toBe('Aha Moment');
		expect(cfg.cssVar).toBe('--color-success');
	});

	it('returns fallback config for unknown evidence type', () => {
		const cfg = getEvidenceTypeConfig('some_new_type');
		expect(cfg.label).toBe('Some New Type');
		expect(cfg.cssVar).toBe('--color-text-subtle');
	});
});

describe('truncateCitation', () => {
	it('returns text unchanged if shorter than max', () => {
		expect(truncateCitation('short text')).toBe('short text');
	});

	it('returns text unchanged if exactly max length', () => {
		const text = 'a'.repeat(40);
		expect(truncateCitation(text)).toBe(text);
	});

	it('truncates text longer than max with ellipsis', () => {
		const text = 'a'.repeat(50);
		const result = truncateCitation(text);
		expect(result.length).toBe(40);
		expect(result.endsWith('\u2026')).toBe(true);
	});

	it('handles empty string', () => {
		expect(truncateCitation('')).toBe('');
	});

	it('respects custom max parameter', () => {
		const text = 'hello world, this is a test';
		const result = truncateCitation(text, 10);
		expect(result.length).toBe(10);
		expect(result.endsWith('\u2026')).toBe(true);
	});
});
