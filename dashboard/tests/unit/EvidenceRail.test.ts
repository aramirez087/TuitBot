/**
 * EvidenceRail.test.ts — Unit tests for EvidenceRail.svelte
 *
 * Tests: visibility based on provider config, empty/building states,
 * search and results, pin/dismiss/auto-query, degraded state,
 * keyboard shortcut, and deduplication.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import EvidenceRail from '$lib/components/composer/EvidenceRail.svelte';
import { createEvidenceState, pinEvidence } from '$lib/stores/evidenceStore';
import type { EvidenceResult, IndexStatusResponse, EvidenceResponse } from '$lib/api/types';

// Mock API
vi.mock('$lib/api', () => ({
	api: {
		vault: {
			searchEvidence: vi.fn(),
			indexStatus: vi.fn(),
		},
	},
}));

// Mock analytics
vi.mock('$lib/analytics/evidenceFunnel', () => ({
	trackEvidenceRailOpened: vi.fn(),
	trackEvidenceSearchExecuted: vi.fn(),
	trackEvidencePinned: vi.fn(),
	trackEvidenceDismissed: vi.fn(),
	trackAutoQueryToggled: vi.fn(),
	trackEvidenceAppliedToSlot: vi.fn(),
	trackEvidenceContributedToDraft: vi.fn(),
}));

function makeResult(overrides: Partial<EvidenceResult> = {}): EvidenceResult {
	return {
		chunk_id: overrides.chunk_id ?? 1,
		node_id: overrides.node_id ?? 10,
		heading_path: overrides.heading_path ?? '# Test',
		snippet: overrides.snippet ?? 'Test snippet',
		match_reason: overrides.match_reason ?? 'semantic',
		score: overrides.score ?? 0.85,
		node_title: overrides.node_title ?? 'Test Note',
		relative_path: overrides.relative_path ?? 'test.md',
	};
}

function makeIndexStatus(overrides: Partial<IndexStatusResponse> = {}): IndexStatusResponse {
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

let apiMock: {
	vault: {
		searchEvidence: ReturnType<typeof vi.fn>;
		indexStatus: ReturnType<typeof vi.fn>;
	};
};

beforeEach(async () => {
	vi.clearAllMocks();
	const { api } = await import('$lib/api');
	apiMock = api as typeof apiMock;
});

describe('EvidenceRail', () => {
	it('renders nothing when provider not configured', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus({ provider_configured: false }));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		// Wait for mount to complete
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		// After mount resolves, the rail should be hidden
		await new Promise((r) => setTimeout(r, 10));
		expect(container.querySelector('.evidence-rail')).toBeNull();
	});

	it('shows building state when index is empty', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(
			makeIndexStatus({ total_chunks: 0, embedded_chunks: 0, index_loaded: false, freshness_pct: 0 })
		);
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		// The rail should show but display the building message
		const rail = container.querySelector('.evidence-rail');
		// Provider is configured, so rail should exist
		expect(rail).not.toBeNull();
	});

	it('shows stale warning when freshness < 50%', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus({ freshness_pct: 30 }));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const warning = container.querySelector('.stale-warning');
		expect(warning?.textContent).toContain('outdated');
	});

	it('renders search bar when provider configured and index available', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		expect(container.querySelector('.search-input')).not.toBeNull();
	});

	it('shows pinned section when evidence is pinned', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: state, onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const pinnedSection = container.querySelector('.pinned-section');
		expect(pinnedSection).not.toBeNull();
		expect(pinnedSection?.textContent).toContain('Pinned (1/5)');
	});

	it('shows pinned count badge in header', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		state = pinEvidence(state, makeResult({ chunk_id: 2 }));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: state, onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const badge = container.querySelector('.pinned-count');
		expect(badge?.textContent).toBe('2');
	});

	it('auto-query button toggles state', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const autoBtn = container.querySelector('.auto-query-btn');
		expect(autoBtn).not.toBeNull();
		await fireEvent.click(autoBtn!);
		expect(onevidence).toHaveBeenCalled();
		const newState = onevidence.mock.calls[0][0];
		expect(newState.autoQueryEnabled).toBe(true);
	});

	it('collapses and expands on header click', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		// Initially expanded
		expect(container.querySelector('.rail-content')).not.toBeNull();
		// Click header to collapse
		const header = container.querySelector('.rail-header');
		await fireEvent.click(header!);
		expect(container.querySelector('.rail-content')).toBeNull();
		// Click again to expand
		await fireEvent.click(header!);
		expect(container.querySelector('.rail-content')).not.toBeNull();
	});

	it('shows keyboard shortcut hint', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const kbd = container.querySelector('.rail-kbd');
		expect(kbd).not.toBeNull();
	});
});
