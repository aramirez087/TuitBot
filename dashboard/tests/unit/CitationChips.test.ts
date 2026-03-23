/**
 * CitationChips.test.ts — Unit tests for CitationChips.svelte
 *
 * Tests: empty state, chip rendering, label derivation, expand toggle,
 * keyboard interaction, remove button, Obsidian URI button.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import CitationChips from '$lib/components/composer/CitationChips.svelte';
import type { VaultCitation } from '$lib/api/types';

vi.mock('$lib/utils/obsidianUri', () => ({
	buildObsidianUri: vi.fn().mockReturnValue('obsidian://open?vault=myvault&file=notes'),
	openExternalUrl: vi.fn().mockResolvedValue(true),
}));

function makeCitation(overrides: Partial<VaultCitation> = {}): VaultCitation {
	return {
		chunk_id: 1,
		node_id: 10,
		heading_path: 'Overview > Strategy',
		source_path: 'notes/ideas.md',
		source_title: null,
		snippet: 'Some relevant snippet text',
		retrieval_boost: 1.0,
		...overrides,
	};
}

const defaultCitations: VaultCitation[] = [
	makeCitation({ chunk_id: 1, source_title: 'Ideas', heading_path: 'Overview > Strategy' }),
	makeCitation({ chunk_id: 2, source_path: 'folder/another-note.md', heading_path: '', source_title: null }),
];

beforeEach(() => {
	vi.clearAllMocks();
});

describe('CitationChips', () => {
	it('renders nothing when citations array is empty', () => {
		const { container } = render(CitationChips, {
			props: { citations: [] },
		});
		const strip = container.querySelector('.citation-strip');
		expect(strip).toBeNull();
	});

	it('renders the "Based on:" label when citations exist', () => {
		const { container } = render(CitationChips, {
			props: { citations: defaultCitations },
		});
		const label = container.querySelector('.citation-label');
		expect(label?.textContent).toContain('Based on:');
	});

	it('renders one chip per citation', () => {
		const { container } = render(CitationChips, {
			props: { citations: defaultCitations },
		});
		const chips = container.querySelectorAll('.citation-chip');
		expect(chips.length).toBe(2);
	});

	it('chipLabel shows source_title when provided', () => {
		const cit = makeCitation({ chunk_id: 3, source_title: 'My Note', heading_path: '', source_path: 'folder/my-note.md' });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chipText = container.querySelector('.chip-text');
		expect(chipText?.textContent).toBe('My Note');
	});

	it('chipLabel falls back to filename from source_path when source_title is null', () => {
		const cit = makeCitation({ chunk_id: 4, source_title: null, heading_path: '', source_path: 'folder/my-file.md' });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chipText = container.querySelector('.chip-text');
		expect(chipText?.textContent).toBe('my-file.md');
	});

	it('chipLabel shows "title › heading" when heading differs from title', () => {
		const cit = makeCitation({
			chunk_id: 5,
			source_title: 'My Note',
			heading_path: 'Section A > Deep Heading',
			source_path: 'folder/my-note.md',
		});
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chipText = container.querySelector('.chip-text');
		expect(chipText?.textContent).toBe('My Note › Deep Heading');
	});

	it('clicking a chip toggles the expanded detail', async () => {
		const cit = makeCitation({ chunk_id: 1 });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		// Detail not visible yet
		expect(container.querySelector('.chip-detail')).toBeNull();
		const chip = container.querySelector('.citation-chip') as HTMLButtonElement;
		await fireEvent.click(chip);
		expect(container.querySelector('.chip-detail')).toBeTruthy();
	});

	it('clicking an expanded chip collapses the detail', async () => {
		const cit = makeCitation({ chunk_id: 1 });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chip = container.querySelector('.citation-chip') as HTMLButtonElement;
		await fireEvent.click(chip);
		expect(container.querySelector('.chip-detail')).toBeTruthy();
		await fireEvent.click(chip);
		expect(container.querySelector('.chip-detail')).toBeNull();
	});

	it('Enter key toggles expanded state', async () => {
		const cit = makeCitation({ chunk_id: 1 });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chip = container.querySelector('.citation-chip') as HTMLButtonElement;
		await fireEvent.keyDown(chip, { key: 'Enter' });
		expect(container.querySelector('.chip-detail')).toBeTruthy();
	});

	it('Space key toggles expanded state', async () => {
		const cit = makeCitation({ chunk_id: 1 });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chip = container.querySelector('.citation-chip') as HTMLButtonElement;
		await fireEvent.keyDown(chip, { key: ' ' });
		expect(container.querySelector('.chip-detail')).toBeTruthy();
	});

	it('shows remove button when onremove prop is provided', () => {
		const { container } = render(CitationChips, {
			props: { citations: [makeCitation()], onremove: vi.fn() },
		});
		const removeBtn = container.querySelector('.chip-remove');
		expect(removeBtn).toBeTruthy();
	});

	it('does not show remove button when onremove is not provided', () => {
		const { container } = render(CitationChips, {
			props: { citations: [makeCitation()] },
		});
		const removeBtn = container.querySelector('.chip-remove');
		expect(removeBtn).toBeNull();
	});

	it('calls onremove with chunk_id when remove button is clicked', async () => {
		const onremove = vi.fn();
		const cit = makeCitation({ chunk_id: 42 });
		const { container } = render(CitationChips, {
			props: { citations: [cit], onremove },
		});
		const removeBtn = container.querySelector('.chip-remove') as HTMLButtonElement;
		await fireEvent.click(removeBtn);
		expect(onremove).toHaveBeenCalledWith(42);
	});

	it('shows "Open in Obsidian" button when isDesktop and vaultPath are set', () => {
		const { container } = render(CitationChips, {
			props: { citations: [makeCitation()], isDesktop: true, vaultPath: '/Users/alice/vaults/notes' },
		});
		const openBtn = container.querySelector('.chip-open');
		expect(openBtn).toBeTruthy();
	});

	it('hides "Open in Obsidian" button when isDesktop is false', () => {
		const { container } = render(CitationChips, {
			props: { citations: [makeCitation()], isDesktop: false, vaultPath: '/Users/alice/vaults/notes' },
		});
		const openBtn = container.querySelector('.chip-open');
		expect(openBtn).toBeNull();
	});

	it('hides "Open in Obsidian" button when vaultPath is not set', () => {
		const { container } = render(CitationChips, {
			props: { citations: [makeCitation()], isDesktop: true, vaultPath: null },
		});
		const openBtn = container.querySelector('.chip-open');
		expect(openBtn).toBeNull();
	});

	it('citation-strip has role="list" and aria-label', () => {
		const { container } = render(CitationChips, {
			props: { citations: [makeCitation()] },
		});
		const strip = container.querySelector('.citation-strip');
		expect(strip?.getAttribute('role')).toBe('list');
		expect(strip?.getAttribute('aria-label')).toBe('Source citations');
	});

	it('chip wrappers have role="listitem"', () => {
		const { container } = render(CitationChips, {
			props: { citations: defaultCitations },
		});
		const items = container.querySelectorAll('[role="listitem"]');
		expect(items.length).toBe(2);
	});

	// ── Graph inserts strip ─────────────────────────────
	it('renders nothing when graphInserts is empty', () => {
		const { container } = render(CitationChips, {
			props: { citations: [], graphInserts: [] },
		});
		const graphStrip = container.querySelector('.graph-strip');
		expect(graphStrip).toBeNull();
	});

	it('renders graph insert chips with source title and slot label', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'tweet', slotLabel: 'Opening hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'My Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], graphInserts: inserts },
		});
		const graphStrip = container.querySelector('.graph-strip');
		expect(graphStrip).toBeTruthy();
		expect(graphStrip?.textContent).toContain('My Note');
		expect(graphStrip?.textContent).toContain('Opening hook');
	});

	it('renders "Related notes:" label for graph inserts', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'tweet', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], graphInserts: inserts },
		});
		const labels = container.querySelectorAll('.citation-label');
		const relatedLabel = Array.from(labels).find((l) => l.textContent?.includes('Related notes'));
		expect(relatedLabel).toBeTruthy();
	});

	it('shows undo button on graph insert when onundoinsert is provided', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'tweet', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], graphInserts: inserts, onundoinsert: vi.fn() },
		});
		const undoBtn = container.querySelector('.graph-strip .chip-remove');
		expect(undoBtn).toBeTruthy();
	});

	it('calls onundoinsert with insert ID when undo is clicked', async () => {
		const onundoinsert = vi.fn();
		const inserts = [
			{ id: 'ins-42', blockId: 'tweet', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], graphInserts: inserts, onundoinsert },
		});
		const undoBtn = container.querySelector('.graph-strip .chip-remove') as HTMLButtonElement;
		await fireEvent.click(undoBtn);
		expect(onundoinsert).toHaveBeenCalledWith('ins-42');
	});

	it('hides undo button on graph insert when onundoinsert is not provided', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'tweet', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], graphInserts: inserts },
		});
		const undoBtn = container.querySelector('.graph-strip .chip-remove');
		expect(undoBtn).toBeNull();
	});

	// ── Obsidian open handler ───────────────────────────
	it('clicking "Open in Obsidian" calls openExternalUrl', async () => {
		const { openExternalUrl } = await import('$lib/utils/obsidianUri');
		const cit = makeCitation({ chunk_id: 1 });
		const { container } = render(CitationChips, {
			props: { citations: [cit], isDesktop: true, vaultPath: '/my/vault' },
		});
		const openBtn = container.querySelector('.chip-open') as HTMLButtonElement;
		await fireEvent.click(openBtn);
		expect(openExternalUrl).toHaveBeenCalled();
	});

	// ── Evidence inserts strip ──────────────────────────
	it('renders nothing for evidence strip when evidenceInserts is empty', () => {
		const { container } = render(CitationChips, {
			props: { citations: [], evidenceInserts: [] },
		});
		const evidenceStrip = container.querySelector('.evidence-strip');
		expect(evidenceStrip).toBeNull();
	});

	it('renders evidence strip when evidenceInserts provided', () => {
		const inserts = [
			{ id: 'e-1', blockId: 'tweet', slotLabel: 'Tweet', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: '3 evidence items', provenance: { node_id: 1, source_role: 'semantic_evidence' }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], evidenceInserts: inserts },
		});
		const evidenceStrip = container.querySelector('.evidence-strip');
		expect(evidenceStrip).toBeTruthy();
	});

	it('evidence strip shows "Evidence used:" label', () => {
		const inserts = [
			{ id: 'e-1', blockId: 'tweet', slotLabel: 'Tweet', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1, source_role: 'semantic_evidence' }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], evidenceInserts: inserts },
		});
		const labels = container.querySelectorAll('.citation-label');
		const evidenceLabel = Array.from(labels).find((l) => l.textContent?.includes('Evidence used'));
		expect(evidenceLabel).toBeTruthy();
	});

	it('evidence chip shows sourceTitle and slotLabel', () => {
		const inserts = [
			{ id: 'e-1', blockId: 'tweet', slotLabel: 'Opening hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Research Note', provenance: { node_id: 1, source_role: 'semantic_evidence' }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], evidenceInserts: inserts },
		});
		const evidenceStrip = container.querySelector('.evidence-strip');
		expect(evidenceStrip?.textContent).toContain('Research Note');
		expect(evidenceStrip?.textContent).toContain('Opening hook');
	});

	it('evidence undo button calls onundoinsert', async () => {
		const onundoinsert = vi.fn();
		const inserts = [
			{ id: 'e-42', blockId: 'tweet', slotLabel: 'Tweet', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1, source_role: 'semantic_evidence' }, timestamp: Date.now() },
		];
		const { container } = render(CitationChips, {
			props: { citations: [], evidenceInserts: inserts, onundoinsert },
		});
		const undoBtn = container.querySelector('.evidence-strip .chip-remove') as HTMLButtonElement;
		await fireEvent.click(undoBtn);
		expect(onundoinsert).toHaveBeenCalledWith('e-42');
	});

	it('evidence strip hidden when evidenceInserts empty', () => {
		const { container } = render(CitationChips, {
			props: { citations: [], evidenceInserts: [] },
		});
		expect(container.querySelector('.evidence-strip')).toBeNull();
	});

	// ── expanded chip detail content ────────────────────
	it('expanded chip shows heading path and snippet', async () => {
		const cit = makeCitation({ chunk_id: 1, heading_path: 'Overview > Strategy', snippet: 'My snippet text' });
		const { container } = render(CitationChips, {
			props: { citations: [cit] },
		});
		const chip = container.querySelector('.citation-chip') as HTMLButtonElement;
		await fireEvent.click(chip);
		const detail = container.querySelector('.chip-detail');
		expect(detail?.textContent).toContain('Overview > Strategy');
		expect(detail?.textContent).toContain('My snippet text');
	});
});
