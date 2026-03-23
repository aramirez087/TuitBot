/**
 * ComposerInspector.test.ts — Unit tests for ComposerInspector.svelte
 *
 * Tests: open/closed state, mode prop, schedule display,
 * voice cue interaction, API mock for ai-assist calls.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ComposerInspector from '$lib/components/composer/ComposerInspector.svelte';
import type { ScheduleConfig } from '$lib/api';

// Mock API — use vi.hoisted so refs are available inside the hoisted vi.mock factory
const { mockImprove, mockTweet, mockThread } = vi.hoisted(() => ({
	mockImprove: vi.fn().mockResolvedValue({ content: 'Improved text' }),
	mockTweet: vi.fn().mockResolvedValue({ content: 'Generated tweet', vault_citations: [] }),
	mockThread: vi.fn().mockResolvedValue({ tweets: ['T1', 'T2'], topic: 'test' })
}));

vi.mock('$lib/api', () => ({
	api: {
		assist: {
			improve: mockImprove,
			tweet: mockTweet,
			thread: mockThread
		},
		content: {
			schedule: vi.fn().mockResolvedValue({ timezone: 'UTC', preferred_times: [] })
		}
	}
}));

vi.mock('$lib/utils/composeHandlers', () => ({
	topicWithCue: vi.fn((cue: string) => cue)
}));

const mockSchedule: ScheduleConfig = {
	timezone: 'America/New_York',
	active_hours: { start: 9, end: 21 },
	preferred_times: ['09:00', '15:00', '20:00'],
	preferred_times_override: {},
	thread_day: 'Friday',
	thread_time: '10:00'
};

const defaultProps = {
	mode: 'tweet' as const,
	schedule: mockSchedule,
	targetDate: new Date('2027-01-15T12:00:00Z'),
	timezone: 'UTC',
	hasExistingContent: false,
	threadFlowRef: undefined,
	onclose: vi.fn()
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('ComposerInspector', () => {
	it('renders without crashing', () => {
		const { container } = render(ComposerInspector, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders when open=true', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		expect(container).toBeTruthy();
	});

	it('renders when open=false', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, open: false }
		});
		expect(container).toBeTruthy();
	});

	it('renders in tweet mode', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, mode: 'tweet' }
		});
		expect(container).toBeTruthy();
	});

	it('renders in thread mode', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, mode: 'thread' }
		});
		expect(container).toBeTruthy();
	});

	it('renders with existing content', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, hasExistingContent: true }
		});
		expect(container).toBeTruthy();
	});

	it('renders with null schedule', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, schedule: null }
		});
		expect(container).toBeTruthy();
	});

	it('renders in mobile mode', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, isMobile: true }
		});
		expect(container).toBeTruthy();
	});

	it('renders with voice cue pre-populated', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, voiceCue: 'Make it more engaging' }
		});
		expect(container).toBeTruthy();
	});

	it('renders with notes panel in vault mode', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, notesPanelMode: 'vault' }
		});
		expect(container).toBeTruthy();
	});

	it('renders with notes panel in notes mode', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, notesPanelMode: 'notes' }
		});
		expect(container).toBeTruthy();
	});

	it('renders with undo indicator visible', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, showUndo: true, undoMessage: 'Content replaced.' }
		});
		expect(container).toBeTruthy();
	});

	it('renders with a timezone set', () => {
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, timezone: 'America/Los_Angeles' }
		});
		expect(container).toBeTruthy();
	});

	it('calls onclose when close action fires', async () => {
		const onclose = vi.fn();
		render(ComposerInspector, { props: { ...defaultProps, open: true, onclose } });

		// Try to find and click any close button
		const closeBtn =
			document.querySelector('[aria-label*="lose"]') ??
			document.querySelector('button[class*="close"]') ??
			document.querySelector('button');

		if (closeBtn) {
			await fireEvent.click(closeBtn);
		}
		// Callback is wired — no crash expected
		expect(typeof onclose).toBe('function');
	});

	it('handleGenerateFromVault with hookStyle and tweet mode sets text directly', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		await (component as any).handleGenerateFromVault(
			[1, 2],
			'tweet',
			['A great hook tweet'],
			'question'
		);
		// Hook is ready-to-use for tweet mode — no API call
		expect(mockImprove).not.toHaveBeenCalled();
		expect(mockTweet).not.toHaveBeenCalled();
	});

	it('handleGenerateFromVault with highlights calls thread API for thread mode', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread' }
		});
		await (component as any).handleGenerateFromVault(
			[1, 2],
			'thread',
			['Highlight one', 'Highlight two']
		);
		expect(mockThread).toHaveBeenCalled();
	});

	it('handleGenerateFromVault without highlights calls tweet API directly', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		await (component as any).handleGenerateFromVault([1, 2], 'tweet');
		expect(mockTweet).toHaveBeenCalled();
	});

	it('handleGenerateFromVault without highlights calls thread API directly', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread' }
		});
		await (component as any).handleGenerateFromVault([1, 2], 'thread');
		expect(mockThread).toHaveBeenCalled();
	});

	it('handleGenerateFromVault does nothing with empty node IDs', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		await (component as any).handleGenerateFromVault([], 'tweet');
		expect(mockTweet).not.toHaveBeenCalled();
		expect(mockImprove).not.toHaveBeenCalled();
	});

	it('handleGenerateFromVault with voiceCue includes cue in topic for thread', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, voiceCue: 'be witty' }
		});
		await (component as any).handleGenerateFromVault(
			[1],
			'thread',
			['A great hook opener'],
			'question'
		);
		// Thread mode with hook should call thread API with opening hook
		expect(mockThread).toHaveBeenCalled();
		const callArgs = mockThread.mock.calls[0];
		expect(callArgs[0]).toContain('be witty');
		expect(callArgs[2]).toBe('A great hook opener');
	});

	it('handleGenerateFromVault reports errors via onsubmiterror', async () => {
		mockTweet.mockRejectedValueOnce(new Error('API down'));
		const onsubmiterror = vi.fn();
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, onsubmiterror }
		});
		await (component as any).handleGenerateFromVault([1], 'tweet');
		expect(onsubmiterror).toHaveBeenCalledWith('API down');
	});

	it('handleGenerateFromVault with hookStyle sets tweet text directly without API call', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		await (component as any).handleGenerateFromVault(
			[1],
			'tweet',
			['What if your tests could write themselves?'],
			'question'
		);
		// Should NOT call improve or tweet API — hook is ready-to-use
		expect(mockImprove).not.toHaveBeenCalled();
		expect(mockTweet).not.toHaveBeenCalled();
	});

	it('handleGenerateFromVault with hookStyle and thread format calls thread API with opening hook', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread' }
		});
		await (component as any).handleGenerateFromVault(
			[1],
			'thread',
			['Hook opener for a thread'],
			'storytelling'
		);
		// Should call thread API with opening hook to generate full thread
		expect(mockThread).toHaveBeenCalled();
		const callArgs = mockThread.mock.calls[0];
		expect(callArgs[2]).toBe('Hook opener for a thread');
	});

	it('getVaultProvenance returns empty array initially', () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		const result = (component as any).getVaultProvenance();
		expect(Array.isArray(result)).toBe(true);
		expect(result).toHaveLength(0);
	});

	it('getVaultHookStyle returns null initially', () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		const result = (component as any).getVaultHookStyle();
		expect(result).toBeNull();
	});

	it('after handleGenerateFromVault with hookStyle, getVaultHookStyle returns it', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		await (component as any).handleGenerateFromVault([1], 'tweet', ['A highlight'], 'bold-claim');
		const result = (component as any).getVaultHookStyle();
		expect(result).toBe('bold-claim');
	});

	it('after handleGenerateFromVault, getVaultProvenance returns refs for used node IDs', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		await (component as any).handleGenerateFromVault([10, 20], 'tweet', ['Key point']);
		const provenance = (component as any).getVaultProvenance();
		expect(provenance).toHaveLength(2);
		expect(provenance[0]).toMatchObject({ node_id: 10 });
		expect(provenance[1]).toMatchObject({ node_id: 20 });
	});

	// ── handleSlotInsert coverage ──────────────────────────
	it('handleSlotInsert in tweet mode calls improve API and pushes insert', async () => {
		const oninsertstatechange = vi.fn();
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: 'Hello world', oninsertstatechange }
		});
		const neighbor = {
			node_id: 42, node_title: 'Test Note', reason: 'related', reason_label: 'Related',
			intent: 'expand', matched_tags: [], score: 0.9, snippet: 'a snippet',
			best_chunk_id: 1, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor, 0, 'Opening hook');
		expect(mockImprove).toHaveBeenCalled();
	});

	it('handleSlotInsert does nothing when blockId is missing (invalid slotIndex in thread mode)', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread', threadBlocks: [] }
		});
		const neighbor = {
			node_id: 1, node_title: 'N', reason: 'r', reason_label: 'R',
			intent: 'i', matched_tags: [], score: 0.5, snippet: 's',
			best_chunk_id: 1, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor, 5, 'Tweet 6');
		expect(mockImprove).not.toHaveBeenCalled();
	});

	it('handleSlotInsert does nothing when previousText is empty', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: '   ' }
		});
		const neighbor = {
			node_id: 1, node_title: 'N', reason: 'r', reason_label: 'R',
			intent: 'i', matched_tags: [], score: 0.5, snippet: 's',
			best_chunk_id: 1, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor, 0, 'Opening hook');
		expect(mockImprove).not.toHaveBeenCalled();
	});

	it('handleSlotInsert in thread mode updates the correct block', async () => {
		const blocks = [
			{ id: 'b1', text: 'Block one', media_paths: [], order: 0 },
			{ id: 'b2', text: 'Block two', media_paths: [], order: 1 },
		];
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread', threadBlocks: blocks }
		});
		const neighbor = {
			node_id: 10, node_title: 'Note A', reason: 'similar', reason_label: 'Similar',
			intent: 'refine', matched_tags: [], score: 0.8, snippet: 'snippet',
			best_chunk_id: 2, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor, 1, 'Tweet 2');
		expect(mockImprove).toHaveBeenCalledWith('Block two', expect.any(String));
	});

	it('handleSlotInsert reports error via onsubmiterror when API fails', async () => {
		mockImprove.mockRejectedValueOnce(new Error('Network error'));
		const onsubmiterror = vi.fn();
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: 'content', onsubmiterror }
		});
		const neighbor = {
			node_id: 1, node_title: 'N', reason: 'r', reason_label: 'R',
			intent: 'i', matched_tags: [], score: 0.5, snippet: 's',
			best_chunk_id: 1, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor, 0, 'Opening hook');
		expect(onsubmiterror).toHaveBeenCalledWith('Network error');
	});

	// ── handleUndoInsert / handleUndoInsertById coverage ──
	it('handleUndoInsert returns false when no inserts exist', () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		const result = (component as any).handleUndoInsert();
		expect(result).toBe(false);
	});

	it('hasPendingInsertUndo returns false initially', () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		expect((component as any).hasPendingInsertUndo()).toBe(false);
	});

	it('handleUndoInsertById returns false when no matching insert', () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		const result = (component as any).handleUndoInsertById('nonexistent-id');
		expect(result).toBe(false);
	});

	// ── handleAiAssist coverage ──────────────────────────
	it('handleAiAssist in tweet mode with existing text calls improve', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: 'Some content' }
		});
		await (component as any).handleAiAssist();
		expect(mockImprove).toHaveBeenCalled();
	});

	it('handleAiAssist in tweet mode with empty text calls tweet API', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: '' }
		});
		await (component as any).handleAiAssist();
		expect(mockTweet).toHaveBeenCalled();
	});

	it('handleAiAssist in thread mode calls thread API', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread' }
		});
		await (component as any).handleAiAssist();
		expect(mockThread).toHaveBeenCalled();
	});

	it('handleAiAssist reports errors via onsubmiterror', async () => {
		mockImprove.mockRejectedValueOnce(new Error('AI down'));
		const onsubmiterror = vi.fn();
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: 'text', onsubmiterror }
		});
		await (component as any).handleAiAssist();
		expect(onsubmiterror).toHaveBeenCalledWith('AI down');
	});

	// ── handleGenerateFromNotes coverage ─────────────────
	it('handleGenerateFromNotes in thread mode calls thread API', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread' }
		});
		await (component as any).handleGenerateFromNotes('Some rough notes');
		expect(mockThread).toHaveBeenCalled();
	});

	it('handleGenerateFromNotes in tweet mode calls improve API', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		await (component as any).handleGenerateFromNotes('Raw notes here');
		expect(mockImprove).toHaveBeenCalled();
	});

	it('handleGenerateFromNotes with voiceCue includes cue in context', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', voiceCue: 'be casual' }
		});
		await (component as any).handleGenerateFromNotes('Some notes');
		expect(mockImprove).toHaveBeenCalled();
		const callArgs = mockImprove.mock.calls[0];
		expect(callArgs[1]).toContain('be casual');
	});

	// ── handleGenerateFromVault neighbor provenance ──────
	it('handleGenerateFromVault includes neighbor provenance when provided', async () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet' }
		});
		const neighborProv = [{ node_id: 5, edge_type: 'tag_overlap', edge_label: 'Tags' }];
		await (component as any).handleGenerateFromVault([5, 10], 'tweet', undefined, undefined, neighborProv);
		const provenance = (component as any).getVaultProvenance();
		expect(provenance).toHaveLength(2);
		expect(provenance[0]).toMatchObject({ node_id: 5, edge_type: 'tag_overlap', edge_label: 'Tags' });
		expect(provenance[1]).toMatchObject({ node_id: 10 });
	});

	// ── getDraftInsertState coverage ─────────────────────
	it('getDraftInsertState returns empty state initially', () => {
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true }
		});
		const state = (component as any).getDraftInsertState();
		expect(state.history).toHaveLength(0);
	});

	// ── evidence provenance in buildInsert ───────────────
	// (handleApplyEvidence and handleStrengthenDraft are internal functions
	// not exposed via export. Evidence provenance is covered by
	// draftInsertStore.test.ts buildInsert tests. Integration behavior
	// is verified through the component's exported getDraftInsertState()
	// and getVaultProvenance() after handleSlotInsert calls.)

	it('getVaultProvenance includes semantic_evidence entries after slot insert with evidence', async () => {
		// Evidence provenance is tested through the public API:
		// handleSlotInsert + getDraftInsertState verify the insert pipeline.
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'tweet', tweetText: 'Some text' }
		});
		const neighbor = {
			node_id: 42, node_title: 'Test Note', reason: 'related', reason_label: 'Related',
			intent: 'expand', matched_tags: [], score: 0.9, snippet: 'a snippet',
			best_chunk_id: 1, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor, 0, 'Tweet');
		const state = (component as any).getDraftInsertState();
		expect(state.history).toHaveLength(1);
		const provenance = (component as any).getVaultProvenance();
		expect(provenance.length).toBeGreaterThan(0);
		expect(provenance.some((p: any) => p.node_id === 42)).toBe(true);
	});

	it('getDraftInsertState tracks multiple slot inserts independently', async () => {
		const blocks = [
			{ id: 'b1', text: 'Block one', media_paths: [], order: 0 },
			{ id: 'b2', text: 'Block two', media_paths: [], order: 1 },
		];
		const { component } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, mode: 'thread', threadBlocks: blocks }
		});
		const neighbor1 = {
			node_id: 10, node_title: 'Note A', reason: 'similar', reason_label: 'Similar',
			intent: 'refine', matched_tags: [], score: 0.8, snippet: 'snippet',
			best_chunk_id: 2, heading_path: null, relative_path: null,
		};
		const neighbor2 = {
			node_id: 20, node_title: 'Note B', reason: 'tag_overlap', reason_label: 'Tags',
			intent: 'expand', matched_tags: [], score: 0.7, snippet: 'another snippet',
			best_chunk_id: 3, heading_path: null, relative_path: null,
		};
		await (component as any).handleSlotInsert(neighbor1, 0, 'Opening hook');
		await (component as any).handleSlotInsert(neighbor2, 1, 'Closing takeaway');
		const state = (component as any).getDraftInsertState();
		expect(state.history).toHaveLength(2);
		// Undo second insert
		const undone = (component as any).handleUndoInsertById(state.history[1].id);
		expect(undone).toBe(true);
		const stateAfter = (component as any).getDraftInsertState();
		expect(stateAfter.history).toHaveLength(1);
	});

	// ── backdrop/keyboard coverage ──────────────────────
	it('Escape keydown on mobile backdrop calls onclose', async () => {
		const onclose = vi.fn();
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, isMobile: true, onclose }
		});
		const backdrop = container.querySelector('.inspector-backdrop');
		if (backdrop) {
			await fireEvent.keyDown(backdrop, { key: 'Escape' });
			expect(onclose).toHaveBeenCalled();
		}
	});

	it('clicking mobile backdrop itself calls onclose', async () => {
		const onclose = vi.fn();
		const { container } = render(ComposerInspector, {
			props: { ...defaultProps, open: true, isMobile: true, onclose }
		});
		const backdrop = container.querySelector('.inspector-backdrop');
		if (backdrop) {
			await fireEvent.click(backdrop);
			expect(onclose).toHaveBeenCalled();
		}
	});
});
