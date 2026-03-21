/**
 * composeHandlers.test.ts — Unit tests for buildComposeRequest provenance + hook_style support.
 */

import { describe, it, expect } from 'vitest';
import { buildComposeRequest } from '$lib/utils/composeHandlers';
import type { ProvenanceRef } from '$lib/api/types';

const baseOpts = {
	mode: 'tweet' as const,
	tweetText: 'Hello world',
	threadBlocks: [],
	selectedTime: null,
	targetDate: new Date('2026-03-21'),
	attachedMedia: []
};

describe('buildComposeRequest', () => {
	it('includes provenance when provided', () => {
		const provenance: ProvenanceRef[] = [
			{ node_id: 42, source_path: 'notes/rust.md' },
			{ node_id: 43, source_path: 'notes/testing.md' }
		];
		const result = buildComposeRequest({ ...baseOpts, provenance });
		expect(result.provenance).toEqual(provenance);
		expect(result.provenance).toHaveLength(2);
	});

	it('omits provenance field when not provided', () => {
		const result = buildComposeRequest(baseOpts);
		expect(result.provenance).toBeUndefined();
	});

	it('omits provenance field when empty array', () => {
		const result = buildComposeRequest({ ...baseOpts, provenance: [] });
		expect(result.provenance).toBeUndefined();
	});

	it('includes hook_style when provided', () => {
		const result = buildComposeRequest({ ...baseOpts, hookStyle: 'contrarian_take' });
		expect(result.hook_style).toBe('contrarian_take');
	});

	it('omits hook_style when not provided', () => {
		const result = buildComposeRequest(baseOpts);
		expect(result.hook_style).toBeUndefined();
	});

	it('includes both provenance and hook_style together', () => {
		const provenance: ProvenanceRef[] = [{ node_id: 1 }];
		const result = buildComposeRequest({
			...baseOpts,
			provenance,
			hookStyle: 'question'
		});
		expect(result.provenance).toEqual(provenance);
		expect(result.hook_style).toBe('question');
	});

	it('still builds correct content_type and content', () => {
		const provenance: ProvenanceRef[] = [{ source_path: 'note.md' }];
		const result = buildComposeRequest({ ...baseOpts, provenance, hookStyle: 'tip' });
		expect(result.content_type).toBe('tweet');
		expect(result.content).toBe('Hello world');
	});
});
