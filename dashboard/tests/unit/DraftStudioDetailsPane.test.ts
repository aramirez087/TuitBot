/**
 * DraftStudioDetailsPane.test.ts — Unit tests for DraftStudioDetailsPane.svelte
 *
 * Tests: panel switcher tabs, CitationChips visibility, provenance mapping,
 * panel switching callback, active tab state.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DraftStudioDetailsPane from '$lib/components/drafts/DraftStudioDetailsPane.svelte';
import type { ProvenanceLink } from '$lib/api/types';

// ─── Store mocks ─────────────────────────────────────────────────────────────

vi.mock('$lib/stores/calendar', () => ({
	schedule: { subscribe: vi.fn((cb) => { cb({ timezone: 'UTC', preferred_times: [] }); return () => {}; }) },
}));

vi.mock('$lib/stores/draftStudio.svelte', () => ({
	getFullDraft: vi.fn().mockReturnValue(null),
	getSelectedDraft: vi.fn().mockReturnValue(null),
	getSelectedDraftTags: vi.fn().mockReturnValue([]),
	getAccountTags: vi.fn().mockReturnValue([]),
	getRevisions: vi.fn().mockReturnValue([]),
	getActivity: vi.fn().mockReturnValue([]),
	loadRevisions: vi.fn().mockResolvedValue(undefined),
	loadActivity: vi.fn().mockResolvedValue(undefined),
}));

// ─── Fixtures ─────────────────────────────────────────────────────────────────

function makeProvenance(overrides: Partial<ProvenanceLink> = {}): ProvenanceLink {
	return {
		id: 1,
		account_id: 'acc1',
		entity_type: 'content',
		entity_id: 100,
		chunk_id: 10,
		node_id: 5,
		seed_id: null,
		heading_path: 'Overview > Strategy',
		source_path: 'notes/ideas.md',
		snippet: 'Relevant text snippet',
		...overrides,
	};
}

const defaultProps = {
	activePanel: 'details' as const,
	prefillSchedule: null,
	provenance: [] as ProvenanceLink[],
	onActivePanel: vi.fn(),
	onUpdateMeta: vi.fn(),
	onAssignTag: vi.fn(),
	onUnassignTag: vi.fn(),
	onCreateTag: vi.fn(),
	onSchedule: vi.fn(),
	onUnschedule: vi.fn(),
	onReschedule: vi.fn(),
	onDuplicate: vi.fn(),
	onRestoreFromRevision: vi.fn(),
	onClose: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('DraftStudioDetailsPane', () => {
	it('renders without crashing', () => {
		const { container } = render(DraftStudioDetailsPane, { props: defaultProps });
		expect(container.querySelector('.details-zone')).toBeTruthy();
	});

	it('renders panel switcher with Details and History tabs', () => {
		const { container } = render(DraftStudioDetailsPane, { props: defaultProps });
		const tabs = container.querySelectorAll('.panel-tab');
		expect(tabs.length).toBe(2);
		expect(tabs[0]?.textContent?.trim()).toBe('Details');
		expect(tabs[1]?.textContent?.trim()).toBe('History');
	});

	it('Details tab is active when activePanel is "details"', () => {
		const { container } = render(DraftStudioDetailsPane, { props: defaultProps });
		const tabs = container.querySelectorAll('.panel-tab');
		expect(tabs[0]?.classList.contains('active')).toBe(true);
		expect(tabs[1]?.classList.contains('active')).toBe(false);
	});

	it('History tab is active when activePanel is "history"', () => {
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, activePanel: 'history' as const },
		});
		const tabs = container.querySelectorAll('.panel-tab');
		expect(tabs[0]?.classList.contains('active')).toBe(false);
		expect(tabs[1]?.classList.contains('active')).toBe(true);
	});

	it('clicking Details tab calls onActivePanel with "details"', async () => {
		const onActivePanel = vi.fn();
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, activePanel: 'history' as const, onActivePanel },
		});
		const tabs = container.querySelectorAll('.panel-tab');
		await fireEvent.click(tabs[0]);
		expect(onActivePanel).toHaveBeenCalledWith('details');
	});

	it('clicking History tab calls onActivePanel with "history"', async () => {
		const onActivePanel = vi.fn();
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, onActivePanel },
		});
		const tabs = container.querySelectorAll('.panel-tab');
		await fireEvent.click(tabs[1]);
		expect(onActivePanel).toHaveBeenCalledWith('history');
	});

	it('also calls loadRevisions and loadActivity when History tab is clicked', async () => {
		const { loadRevisions, loadActivity } = await import('$lib/stores/draftStudio.svelte');
		const { container } = render(DraftStudioDetailsPane, { props: defaultProps });
		const tabs = container.querySelectorAll('.panel-tab');
		await fireEvent.click(tabs[1]);
		expect(loadRevisions).toHaveBeenCalled();
		expect(loadActivity).toHaveBeenCalled();
	});

	it('does not render CitationChips when provenance is empty', () => {
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, provenance: [] },
		});
		const strip = container.querySelector('.citation-strip');
		expect(strip).toBeNull();
	});

	it('renders CitationChips when provenance has source_path entries', () => {
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, provenance: [makeProvenance()] },
		});
		const strip = container.querySelector('.citation-strip');
		expect(strip).toBeTruthy();
	});

	it('filters out provenance entries without source_path', () => {
		const provenance: ProvenanceLink[] = [
			makeProvenance({ source_path: 'notes/valid.md' }),
			makeProvenance({ id: 2, chunk_id: 20, source_path: undefined }),
		];
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, provenance },
		});
		// Only one chip should be rendered — the one with source_path
		const chips = container.querySelectorAll('.citation-chip');
		expect(chips.length).toBe(1);
	});

	it('maps provenance fields to VaultCitation correctly', () => {
		const prov = makeProvenance({
			chunk_id: 99,
			node_id: 7,
			heading_path: 'Section > Subsection',
			source_path: 'notes/mapped.md',
		});
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, provenance: [prov] },
		});
		const chip = container.querySelector('.citation-chip');
		// heading_path leaf: "Subsection" — source_path filename: "mapped.md"
		// chipLabel: "mapped.md › Subsection" (no source_title)
		expect(chip?.getAttribute('title')).toBe('Section > Subsection');
	});

	it('uses chunk_id from provenance in chip aria-expanded', () => {
		const prov = makeProvenance({ chunk_id: 42 });
		const { container } = render(DraftStudioDetailsPane, {
			props: { ...defaultProps, provenance: [prov] },
		});
		const chip = container.querySelector('.citation-chip');
		expect(chip?.getAttribute('aria-expanded')).toBe('false');
	});
});
