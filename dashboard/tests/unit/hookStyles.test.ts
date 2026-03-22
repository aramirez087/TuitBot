/**
 * hookStyles.test.ts — Unit tests for hook style label mapping and confidence badges.
 */

import { describe, it, expect } from 'vitest';
import { getStyleLabel, getConfidenceBadge } from '$lib/utils/hookStyles';

describe('getStyleLabel', () => {
	it('maps "question" to "Question"', () => {
		expect(getStyleLabel('question')).toBe('Question');
	});

	it('maps "contrarian_take" to "Hot Take"', () => {
		expect(getStyleLabel('contrarian_take')).toBe('Hot Take');
	});

	it('maps "tip" to "Quick Tip"', () => {
		expect(getStyleLabel('tip')).toBe('Quick Tip');
	});

	it('maps "list" to "List"', () => {
		expect(getStyleLabel('list')).toBe('List');
	});

	it('maps "most_people_think_x" to "Myth Buster"', () => {
		expect(getStyleLabel('most_people_think_x')).toBe('Myth Buster');
	});

	it('maps "storytelling" to "Story"', () => {
		expect(getStyleLabel('storytelling')).toBe('Story');
	});

	it('maps "before_after" to "Before/After"', () => {
		expect(getStyleLabel('before_after')).toBe('Before/After');
	});

	it('maps "general" to "General"', () => {
		expect(getStyleLabel('general')).toBe('General');
	});

	it('falls back to title-cased version for unknown styles', () => {
		expect(getStyleLabel('some_new_style')).toBe('Some New Style');
	});

	it('handles single-word unknown style', () => {
		expect(getStyleLabel('custom')).toBe('Custom');
	});

	it('maps "story" to "Story" (angle type)', () => {
		expect(getStyleLabel('story')).toBe('Story');
	});

	it('maps "listicle" to "Listicle" (angle type)', () => {
		expect(getStyleLabel('listicle')).toBe('Listicle');
	});

	it('maps "hot_take" to "Hot Take" (angle type)', () => {
		expect(getStyleLabel('hot_take')).toBe('Hot Take');
	});
});

describe('getConfidenceBadge', () => {
	it('returns "Strong" for high confidence', () => {
		const badge = getConfidenceBadge('high');
		expect(badge.label).toBe('Strong');
		expect(badge.cssClass).toBe('confidence-high');
	});

	it('returns "Good" for medium confidence', () => {
		const badge = getConfidenceBadge('medium');
		expect(badge.label).toBe('Good');
		expect(badge.cssClass).toBe('confidence-medium');
	});

	it('returns "Good" for unknown confidence values', () => {
		const badge = getConfidenceBadge('low');
		expect(badge.label).toBe('Good');
		expect(badge.cssClass).toBe('confidence-medium');
	});
});
