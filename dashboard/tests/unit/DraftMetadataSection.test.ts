/**
 * DraftMetadataSection.test.ts — Unit tests for hook style badge rendering.
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import DraftMetadataSection from '$lib/components/drafts/DraftMetadataSection.svelte';
import type { DraftSummary } from '$lib/api/types';

function makeDraft(overrides: Partial<DraftSummary> = {}): DraftSummary {
	return {
		id: 1,
		title: 'Test draft',
		content_type: 'tweet',
		content_preview: 'Hello world',
		status: 'active',
		scheduled_for: null,
		archived_at: null,
		updated_at: new Date().toISOString(),
		created_at: new Date().toISOString(),
		source: 'manual',
		...overrides
	};
}

describe('DraftMetadataSection', () => {
	it('renders "Manual" for source "manual"', () => {
		const { container } = render(DraftMetadataSection, {
			props: { draftSummary: makeDraft({ source: 'manual' }) }
		});
		const metaValues = container.querySelectorAll('.meta-value');
		const sourceValue = Array.from(metaValues).find((el) => el.textContent?.includes('Manual'));
		expect(sourceValue).toBeTruthy();
		expect(container.querySelector('.hook-badge')).toBeNull();
	});

	it('renders "AI Assist" for source "assist"', () => {
		const { container } = render(DraftMetadataSection, {
			props: { draftSummary: makeDraft({ source: 'assist' }) }
		});
		const metaValues = container.querySelectorAll('.meta-value');
		const sourceValue = Array.from(metaValues).find((el) => el.textContent?.includes('AI Assist'));
		expect(sourceValue).toBeTruthy();
		expect(container.querySelector('.hook-badge')).toBeNull();
	});

	it('renders "AI Assist" with "Hot Take" badge for source "assist:hook:contrarian_take"', () => {
		const { container } = render(DraftMetadataSection, {
			props: { draftSummary: makeDraft({ source: 'assist:hook:contrarian_take' }) }
		});
		const badge = container.querySelector('.hook-badge');
		expect(badge).toBeTruthy();
		expect(badge?.textContent?.trim()).toBe('Hot Take');
		const metaValues = container.querySelectorAll('.meta-value');
		const sourceValue = Array.from(metaValues).find((el) => el.textContent?.includes('AI Assist'));
		expect(sourceValue).toBeTruthy();
	});

	it('renders "Quick Tip" badge for source "assist:hook:tip"', () => {
		const { container } = render(DraftMetadataSection, {
			props: { draftSummary: makeDraft({ source: 'assist:hook:tip' }) }
		});
		const badge = container.querySelector('.hook-badge');
		expect(badge).toBeTruthy();
		expect(badge?.textContent?.trim()).toBe('Quick Tip');
	});

	it('renders "Discovery" for source "discovery"', () => {
		const { container } = render(DraftMetadataSection, {
			props: { draftSummary: makeDraft({ source: 'discovery' }) }
		});
		const metaValues = container.querySelectorAll('.meta-value');
		const sourceValue = Array.from(metaValues).find((el) => el.textContent?.includes('Discovery'));
		expect(sourceValue).toBeTruthy();
		expect(container.querySelector('.hook-badge')).toBeNull();
	});

	it('does not render badge for plain "assist" source', () => {
		const { container } = render(DraftMetadataSection, {
			props: { draftSummary: makeDraft({ source: 'assist' }) }
		});
		expect(container.querySelector('.hook-badge')).toBeNull();
	});
});
