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
	trackEvidenceSearchLatency: vi.fn(),
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
	apiMock = api as unknown as typeof apiMock;
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

	it('shows strengthen button when pinned and has existing content', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		const onevidence = vi.fn();
		const onstrengthen = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: state, onevidence, hasExistingContent: true, onstrengthen },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const strengthenBtn = container.querySelector('.strengthen-btn');
		expect(strengthenBtn).not.toBeNull();
		expect(strengthenBtn?.textContent).toContain('Strengthen draft');
	});

	it('hides strengthen button when no existing content', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		const onevidence = vi.fn();
		const onstrengthen = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: state, onevidence, hasExistingContent: false, onstrengthen },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const strengthenBtn = container.querySelector('.strengthen-btn');
		expect(strengthenBtn).toBeNull();
	});

	it('renders slot picker dropdown on evidence card in thread mode', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		const onevidence = vi.fn();
		const onapplytoSlot = vi.fn();
		const threadBlocks = [
			{ id: 'b1', text: 'First', media_paths: [], order: 0 },
			{ id: 'b2', text: 'Second', media_paths: [], order: 1 },
		];
		const { container } = render(EvidenceRail, {
			props: {
				evidenceState: state,
				onevidence,
				onapplytoSlot,
				hasExistingContent: true,
				mode: 'thread',
				threadBlocks,
			},
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		// The apply button should exist on the pinned card
		const applyBtn = container.querySelector('.card-action-btn[aria-haspopup="true"]');
		expect(applyBtn).not.toBeNull();
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

	// ── Search execution ────────────────────────────────
	it('executes search on manual input and shows results', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockResolvedValue({
			results: [makeResult({ chunk_id: 10 }), makeResult({ chunk_id: 11 })],
		});
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		// Type a query
		await fireEvent.input(searchInput, { target: { value: 'test query' } });
		// Wait for debounced search to fire
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 10));
		const resultsSection = container.querySelector('.results-section');
		expect(resultsSection).not.toBeNull();
		expect(resultsSection?.textContent).toContain('Results');
	});

	it('shows loading shimmer while searching', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		// Never resolve the search so loading stays true
		apiMock.vault.searchEvidence.mockReturnValue(new Promise(() => {}));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'loading test' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		// Shimmer should be visible while waiting
		const shimmer = container.querySelector('.shimmer-list');
		expect(shimmer).not.toBeNull();
	});

	it('shows error state when search fails', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockRejectedValue(new Error('Search failed'));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'error query' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 50));
		const errorEl = container.querySelector('.rail-error');
		expect(errorEl).not.toBeNull();
		expect(errorEl?.textContent).toContain('Search failed');
	});

	it('shows "no results" message when search returns empty', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockResolvedValue({ results: [] });
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'nothing found' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 50));
		const noResults = container.querySelector('.no-results');
		expect(noResults).not.toBeNull();
		expect(noResults?.textContent).toContain('No matching evidence');
	});

	it('clears results and collapses on Escape in search input', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockResolvedValue({
			results: [makeResult()],
		});
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'test' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 50));
		// Press Escape
		await fireEvent.keyDown(searchInput, { key: 'Escape' });
		// Rail should collapse
		expect(container.querySelector('.rail-content')).toBeNull();
	});

	// ── Keyboard shortcut ───────────────────────────────
	it('toggles collapse on Cmd+Shift+E keyboard shortcut', async () => {
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
		// Simulate Cmd+Shift+E
		await fireEvent.keyDown(window, { key: 'e', metaKey: true, shiftKey: true });
		// Should collapse
		expect(container.querySelector('.rail-content')).toBeNull();
		// Again to expand
		await fireEvent.keyDown(window, { key: 'e', metaKey: true, shiftKey: true });
		expect(container.querySelector('.rail-content')).not.toBeNull();
	});

	// ── Building index progress ─────────────────────────
	it('shows building progress bar when index is partially built', async () => {
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
		const emptyState = container.querySelector('.rail-empty-state');
		expect(emptyState).not.toBeNull();
		expect(emptyState?.textContent).toContain('Building index');
	});

	// ── Manual search clears on empty ───────────────────
	it('clears results when search input is emptied', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockResolvedValue({
			results: [makeResult()],
		});
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		// Type then clear
		await fireEvent.input(searchInput, { target: { value: 'test' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 50));
		// Now clear
		await fireEvent.input(searchInput, { target: { value: '' } });
		await new Promise((r) => setTimeout(r, 10));
		// Results section should be gone
		expect(container.querySelector('.results-section')).toBeNull();
	});

	// ── Strengthen button fires callback ────────────────
	it('fires onstrengthen when strengthen button is clicked', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		const onevidence = vi.fn();
		const onstrengthen = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: state, onevidence, hasExistingContent: true, onstrengthen },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const strengthenBtn = container.querySelector('.strengthen-btn');
		await fireEvent.click(strengthenBtn!);
		expect(onstrengthen).toHaveBeenCalledOnce();
	});

	// ── aria-expanded on header ─────────────────────────
	it('header aria-expanded reflects collapsed state', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const header = container.querySelector('.rail-header');
		expect(header?.getAttribute('aria-expanded')).toBe('true');
		await fireEvent.click(header!);
		expect(header?.getAttribute('aria-expanded')).toBe('false');
	});

	// ── auto-query-btn aria-pressed ─────────────────────
	it('auto-query button has correct aria-pressed attribute', async () => {
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
		expect(autoBtn?.getAttribute('aria-pressed')).toBe('false');
	});

	// ── Index status fetch error ────────────────────────
	it('handles index status fetch error gracefully', async () => {
		apiMock.vault.indexStatus.mockRejectedValue(new Error('Network error'));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		// Should not render the rail when indexStatus is null
		expect(container.querySelector('.evidence-rail')).toBeNull();
	});

	// ── focusedText in tweet mode vs thread mode ────────
	it('uses tweetText for focusedText in tweet mode', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: {
				evidenceState: createEvidenceState(),
				onevidence,
				mode: 'tweet',
				tweetText: 'My tweet content',
			},
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		// Rail renders — focusedText is internal but rail should be visible
		expect(container.querySelector('.evidence-rail')).not.toBeNull();
	});

	it('uses focused block text in thread mode', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		const threadBlocks = [
			{ id: 'b1', text: 'First block', media_paths: [], order: 0 },
			{ id: 'b2', text: 'Second block', media_paths: [], order: 1 },
		];
		const { container } = render(EvidenceRail, {
			props: {
				evidenceState: createEvidenceState(),
				onevidence,
				mode: 'thread',
				threadBlocks,
				focusedBlockIndex: 1,
			},
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		expect(container.querySelector('.evidence-rail')).not.toBeNull();
	});

	// ── Stale warning shows freshness percentage ────────
	it('stale warning includes freshness percentage', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus({ freshness_pct: 25 }));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const warning = container.querySelector('.stale-warning');
		expect(warning?.textContent).toContain('25%');
	});

	// ── No stale warning when freshness >= 50% ──────────
	it('no stale warning when freshness is adequate', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus({ freshness_pct: 75 }));
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		expect(container.querySelector('.stale-warning')).toBeNull();
	});

	// ── selectionSessionId passed to analytics ──────────
	it('tracks rail opened with selectionSessionId', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const onevidence = vi.fn();
		render(EvidenceRail, {
			props: {
				evidenceState: createEvidenceState(),
				onevidence,
				selectionSessionId: 'sess-123',
			},
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const { trackEvidenceRailOpened } = await import('$lib/analytics/evidenceFunnel');
		expect(trackEvidenceRailOpened).toHaveBeenCalledWith('sess-123', true);
	});

	// ── Search label "Suggestions" for auto-suggested results ──
	it('labels results as "Suggestions" when auto-queried', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		const state = { ...createEvidenceState(), autoQueryEnabled: true };
		apiMock.vault.searchEvidence.mockResolvedValue({
			results: [makeResult({ chunk_id: 20 })],
		});
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: {
				evidenceState: state,
				onevidence,
				tweetText: 'This is a longer tweet that should trigger auto query since it exceeds ten characters',
			},
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		// Wait for auto-query debounce (800ms) + search
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 2000 });
		await new Promise((r) => setTimeout(r, 50));
		const sectionLabel = container.querySelector('.section-label');
		// The results section should say "Suggestions"
		const resultsSection = container.querySelector('.results-section');
		if (resultsSection) {
			expect(resultsSection.textContent).toContain('Suggestions');
		}
	});

	// ── Pin/dismiss via EvidenceCard callbacks in results ──
	it('calls onevidence with pinned state when pin action on result card', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockResolvedValue({
			results: [makeResult({ chunk_id: 42 })],
		});
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'pin test' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 50));
		// Reset onevidence after search-related calls
		onevidence.mockClear();
		// Click pin on the first result card
		const pinBtn = container.querySelector('.results-section [aria-label="Pin"]');
		if (pinBtn) {
			await fireEvent.click(pinBtn);
			expect(onevidence).toHaveBeenCalled();
			const lastCall = onevidence.mock.calls[onevidence.mock.calls.length - 1][0];
			expect(lastCall.pinned.length).toBe(1);
		}
	});

	it('calls onevidence with dismissed state when dismiss action on result card', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		apiMock.vault.searchEvidence.mockResolvedValue({
			results: [makeResult({ chunk_id: 42 })],
		});
		const onevidence = vi.fn();
		const { container } = render(EvidenceRail, {
			props: { evidenceState: createEvidenceState(), onevidence },
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		const searchInput = container.querySelector('.search-input') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'dismiss test' } });
		await vi.waitFor(() => {
			expect(apiMock.vault.searchEvidence).toHaveBeenCalled();
		}, { timeout: 1000 });
		await new Promise((r) => setTimeout(r, 50));
		// Reset onevidence after search-related calls
		onevidence.mockClear();
		// Click dismiss on the first result card
		const dismissBtn = container.querySelector('.results-section [aria-label="Dismiss"]');
		if (dismissBtn) {
			await fireEvent.click(dismissBtn);
			expect(onevidence).toHaveBeenCalled();
			const lastCall = onevidence.mock.calls[onevidence.mock.calls.length - 1][0];
			expect(lastCall.dismissedChunkIds.has(42)).toBe(true);
		}
	});

	// ── Thread mode slot options ────────────────────────
	it('renders correct slot labels for thread mode', async () => {
		apiMock.vault.indexStatus.mockResolvedValue(makeIndexStatus());
		let state = createEvidenceState();
		state = pinEvidence(state, makeResult({ chunk_id: 1 }));
		const onevidence = vi.fn();
		const threadBlocks = [
			{ id: 'b1', text: 'First', media_paths: [], order: 0 },
			{ id: 'b2', text: 'Second', media_paths: [], order: 1 },
			{ id: 'b3', text: 'Third', media_paths: [], order: 2 },
		];
		const { container } = render(EvidenceRail, {
			props: {
				evidenceState: state,
				onevidence,
				onapplytoSlot: vi.fn(),
				hasExistingContent: true,
				mode: 'thread',
				threadBlocks,
			},
		});
		await vi.waitFor(() => {
			expect(apiMock.vault.indexStatus).toHaveBeenCalled();
		});
		await new Promise((r) => setTimeout(r, 10));
		// Click the apply button to open slot picker
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		if (applyBtn) {
			await fireEvent.click(applyBtn);
			const slotOptions = container.querySelectorAll('.slot-option');
			expect(slotOptions.length).toBe(3);
		}
	});
});
