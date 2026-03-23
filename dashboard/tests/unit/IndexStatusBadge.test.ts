/**
 * IndexStatusBadge.test.ts — Unit tests for IndexStatusBadge.svelte
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import IndexStatusBadge from '$lib/components/composer/IndexStatusBadge.svelte';
import type { IndexStatusResponse } from '$lib/api/types';

function makeStatus(overrides: Partial<IndexStatusResponse> = {}): IndexStatusResponse {
	return {
		total_chunks: overrides.total_chunks ?? 100,
		embedded_chunks: overrides.embedded_chunks ?? 95,
		dirty_chunks: overrides.dirty_chunks ?? 5,
		freshness_pct: overrides.freshness_pct ?? 95,
		last_indexed_at: overrides.last_indexed_at ?? '2026-03-23T10:00:00Z',
		model_id: overrides.model_id ?? 'text-embedding-3-small',
		provider_configured: overrides.provider_configured ?? true,
		index_loaded: overrides.index_loaded ?? true,
		index_size: overrides.index_size ?? 1024,
	};
}

describe('IndexStatusBadge', () => {
	it('renders green dot for 95%+ freshness', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ freshness_pct: 97 }) },
		});
		const btn = container.querySelector('.badge-dot-btn');
		expect(btn?.classList.contains('green')).toBe(true);
		expect(btn?.getAttribute('title')).toContain('fresh');
	});

	it('renders amber dot for 50-94% freshness', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ freshness_pct: 70 }) },
		});
		const btn = container.querySelector('.badge-dot-btn');
		expect(btn?.classList.contains('amber')).toBe(true);
		expect(btn?.getAttribute('title')).toContain('partially stale');
	});

	it('renders red dot for <50% freshness', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ freshness_pct: 30 }) },
		});
		const btn = container.querySelector('.badge-dot-btn');
		expect(btn?.classList.contains('red')).toBe(true);
		expect(btn?.getAttribute('title')).toContain('stale');
	});

	it('renders gray dot when no provider configured', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ provider_configured: false }) },
		});
		const btn = container.querySelector('.badge-dot-btn');
		expect(btn?.classList.contains('gray')).toBe(true);
		expect(btn?.getAttribute('title')).toContain('No embedding provider');
	});

	it('renders gray dot when no index exists', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ index_loaded: false, total_chunks: 0 }) },
		});
		const btn = container.querySelector('.badge-dot-btn');
		expect(btn?.classList.contains('gray')).toBe(true);
		expect(btn?.getAttribute('title')).toContain('No index');
	});

	it('renders gray dot when status is null', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: null },
		});
		const btn = container.querySelector('.badge-dot-btn');
		expect(btn?.classList.contains('gray')).toBe(true);
		expect(btn?.getAttribute('title')).toContain('Loading');
	});

	it('has pulse animation on amber dot', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ freshness_pct: 60 }) },
		});
		const dot = container.querySelector('.badge-dot');
		expect(dot?.classList.contains('pulse')).toBe(true);
	});

	it('has no pulse on green dot', () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ freshness_pct: 99 }) },
		});
		const dot = container.querySelector('.badge-dot');
		expect(dot?.classList.contains('pulse')).toBe(false);
	});
});
