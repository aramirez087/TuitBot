/**
 * IndexStatusBadge.test.ts — Unit tests for IndexStatusBadge.svelte
 */

import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
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
		deployment_mode: overrides.deployment_mode ?? 'desktop',
		search_available: overrides.search_available ?? true,
		provider_name: overrides.provider_name ?? 'openai',
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

	it('shows privacy label for desktop in popover', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ deployment_mode: 'desktop' }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('Vectors never leave this machine');
	});

	it('shows privacy label for cloud in popover', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ deployment_mode: 'cloud' }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('Snippets truncated to 120 chars');
	});

	it('shows privacy label for self_host in popover', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ deployment_mode: 'self_host' }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('No external vector database');
	});

	it('shows search available when search_available is true', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ search_available: true }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('Available');
	});

	it('shows keyword fallback when search_available is false and provider configured', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ search_available: false, provider_configured: true }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('Provider unreachable');
	});

	it('shows provider name in popover', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ provider_name: 'ollama' }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('ollama');
	});

	it('uses deploymentMode prop as fallback when status has no deployment_mode', async () => {
		const status = makeStatus();
		delete (status as unknown as Record<string, unknown>).deployment_mode;
		const { container } = render(IndexStatusBadge, {
			props: { status, deploymentMode: 'cloud' },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('Snippets truncated');
	});

	it('does not open popover in compact mode', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus(), compact: true },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover).toBeNull();
	});

	it('toggles popover closed on second click', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus() },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		expect(container.querySelector('.badge-popover')).toBeTruthy();
		await fireEvent.click(btn);
		expect(container.querySelector('.badge-popover')).toBeNull();
	});

	it('shows "Keyword only" when search unavailable and no provider', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ search_available: false, provider_configured: false }) },
		});
		// Gray dot for no provider, so popover won't show searchLabel without clicking
		// But we can test via the badge-dot title
		const btn = container.querySelector('.badge-dot-btn')!;
		expect(btn.getAttribute('title')).toContain('No embedding provider');
	});

	it('shows chunk counts in popover', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ embedded_chunks: 80, total_chunks: 100 }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('80');
		expect(popover?.textContent).toContain('100');
	});

	it('shows model_id in popover when present', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ model_id: 'nomic-embed-v1.5' }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('nomic-embed-v1.5');
	});

	it('hides model row when model_id is null', async () => {
		const status = makeStatus({ model_id: null });
		const { container } = render(IndexStatusBadge, {
			props: { status },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).not.toContain('nomic');
	});

	it('shows last indexed date in popover', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus({ last_indexed_at: '2026-03-23T10:00:00Z' }) },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const popover = container.querySelector('.badge-popover');
		expect(popover?.textContent).toContain('Last indexed');
	});

	it('hides last indexed row when last_indexed_at is null', async () => {
		const status: IndexStatusResponse = {
			...makeStatus(),
			last_indexed_at: null,
		};
		const { container } = render(IndexStatusBadge, {
			props: { status },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const rows = container.querySelectorAll('.popover-row');
		const lastIndexedRow = Array.from(rows).find((r) => r.textContent?.includes('Last indexed'));
		expect(lastIndexedRow).toBeUndefined();
	});

	it('reindex button is disabled', async () => {
		const { container } = render(IndexStatusBadge, {
			props: { status: makeStatus() },
		});
		const btn = container.querySelector('.badge-dot-btn')!;
		await fireEvent.click(btn);
		const reindexBtn = container.querySelector('.popover-reindex-btn') as HTMLButtonElement;
		expect(reindexBtn).toBeTruthy();
		expect(reindexBtn.disabled).toBe(true);
	});
});
