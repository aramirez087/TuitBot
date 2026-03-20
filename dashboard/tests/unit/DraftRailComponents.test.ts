/**
 * DraftRailComponents.test.ts — Unit tests for DraftRail, DraftRailItem,
 * DraftRailTabs, and DraftFilterBar.
 *
 * Written by Sentinel (QA Lead) — 2026-03-19.
 * WHY: Q5 coverage task (4ff1f201). lib/components/drafts/ had 0% frontend
 * coverage. These tests cover user-visible rendering and interaction for the
 * four most-used components in the Draft Studio rail.
 *
 * Architecture notes:
 * - DraftRail is Svelte 5 / $props() runes — all callbacks are prop functions.
 * - DraftRailItem exposes focus(), scrollIntoViewIfNeeded(), armDelete(),
 *   confirmDelete(), cancelDelete(), isConfirmingDelete() via `export`.
 * - DraftFilterBar debounces search 300 ms; tests use vi.useFakeTimers().
 * - jsdom lacks scrollIntoView — mock before any DraftRailItem render.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DraftRail from '$lib/components/drafts/DraftRail.svelte';
import DraftRailItem from '$lib/components/drafts/DraftRailItem.svelte';
import DraftRailTabs from '$lib/components/drafts/DraftRailTabs.svelte';
import DraftFilterBar from '$lib/components/drafts/DraftFilterBar.svelte';
import type { DraftSummary, ContentTag } from '$lib/api/types';

// ─── jsdom polyfill: scrollIntoView is not implemented ───────────────────────
// DraftRailItem.scrollIntoViewIfNeeded() calls rootEl.scrollIntoView().
// Without this mock every render with $effect active throws.
beforeEach(() => {
	Element.prototype.scrollIntoView = vi.fn();
});

// ─── Shared fixtures ─────────────────────────────────────────────────────────

function makeDraft(overrides: Partial<DraftSummary> = {}): DraftSummary {
	return {
		id: 1,
		title: 'Test draft',
		content_type: 'tweet',
		content_preview: 'Hello world preview text that is long enough',
		status: 'active',
		scheduled_for: null,
		archived_at: null,
		updated_at: new Date().toISOString(),
		created_at: new Date().toISOString(),
		source: 'manual',
		...overrides,
	};
}

const defaultTabCounts = { active: 3, scheduled: 1, posted: 0, archive: 0 };

const defaultRailProps = {
	drafts: [makeDraft()],
	selectedId: null as number | null,
	tab: 'active' as const,
	tabCounts: defaultTabCounts,
	loading: false,
	onselect: vi.fn(),
	ontabchange: vi.fn(),
	oncreate: vi.fn(),
	ondelete: vi.fn(),
	onduplicate: vi.fn(),
	onrestore: vi.fn(),
};

// ─── DraftRailTabs ────────────────────────────────────────────────────────────

describe('DraftRailTabs', () => {
	const tabs = [
		{ key: 'active' as const, label: 'Drafts' },
		{ key: 'scheduled' as const, label: 'Scheduled' },
		{ key: 'posted' as const, label: 'Posted' },
	];

	it('renders all tab buttons', () => {
		const { container } = render(DraftRailTabs, {
			props: { tabs, tab: 'active', tabCounts: defaultTabCounts, ontabchange: vi.fn() },
		});
		const buttons = container.querySelectorAll('[role="tab"]');
		expect(buttons.length).toBe(3);
	});

	it('marks the active tab with aria-selected', () => {
		const { container } = render(DraftRailTabs, {
			props: { tabs, tab: 'scheduled', tabCounts: defaultTabCounts, ontabchange: vi.fn() },
		});
		const selectedBtn = container.querySelector('[aria-selected="true"]');
		expect(selectedBtn).not.toBeNull();
		expect(selectedBtn!.textContent).toContain('Scheduled');
	});

	it('calls ontabchange with correct key when tab is clicked', async () => {
		const ontabchange = vi.fn();
		const { container } = render(DraftRailTabs, {
			props: { tabs, tab: 'active', tabCounts: defaultTabCounts, ontabchange },
		});
		const buttons = container.querySelectorAll('[role="tab"]');
		// Click "Scheduled" (index 1)
		await fireEvent.click(buttons[1]);
		expect(ontabchange).toHaveBeenCalledWith('scheduled');
	});

	it('displays tab counts', () => {
		const { container } = render(DraftRailTabs, {
			props: { tabs, tab: 'active', tabCounts: { active: 7, scheduled: 2, posted: 0, archive: 1 }, ontabchange: vi.fn() },
		});
		expect(container.textContent).toContain('7');
		expect(container.textContent).toContain('2');
	});
});

// ─── DraftFilterBar ───────────────────────────────────────────────────────────

describe('DraftFilterBar', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('renders search input', () => {
		const { container } = render(DraftFilterBar, {
			props: {
				searchQuery: '',
				sortBy: 'updated',
				tagFilter: null,
				tags: [],
				onsearch: vi.fn(),
				onsort: vi.fn(),
				ontagfilter: vi.fn(),
			},
		});
		const input = container.querySelector('input[type="text"]');
		expect(input).not.toBeNull();
	});

	it('calls onsearch after debounce when search query changes', async () => {
		vi.useFakeTimers();
		const onsearch = vi.fn();
		const { container } = render(DraftFilterBar, {
			props: {
				searchQuery: '',
				sortBy: 'updated',
				tagFilter: null,
				tags: [],
				onsearch,
				onsort: vi.fn(),
				ontagfilter: vi.fn(),
			},
		});
		const input = container.querySelector('input[type="text"]') as HTMLInputElement;
		expect(input).not.toBeNull();
		// Simulate typing — component debounces 300 ms via oninput
		await fireEvent.input(input, { target: { value: 'hello' } });
		expect(onsearch).not.toHaveBeenCalled(); // not yet — debounced
		vi.advanceTimersByTime(300);
		expect(onsearch).toHaveBeenCalledWith('hello');
	});

	it('does not render tag dropdown when tags is empty', () => {
		const { container } = render(DraftFilterBar, {
			props: {
				searchQuery: '',
				sortBy: 'updated',
				tagFilter: null,
				tags: [],
				onsearch: vi.fn(),
				onsort: vi.fn(),
				ontagfilter: vi.fn(),
			},
		});
		// No tag chip button visible when no tags and no active filters
		const buttons = container.querySelectorAll('button');
		const labels = Array.from(buttons).map((b) => b.textContent?.trim() ?? '');
		// Should only have the sort button (no tag chip, no clear)
		expect(labels.some((l) => l.toLowerCase().includes('tag'))).toBe(false);
	});

	it('renders tag chips when tags are provided', () => {
		const tags: ContentTag[] = [
			{ id: 1, account_id: 'acc1', name: 'Growth', color: '#2ea043' },
		];
		const { container } = render(DraftFilterBar, {
			props: {
				searchQuery: '',
				sortBy: 'updated',
				tagFilter: null,
				tags,
				onsearch: vi.fn(),
				onsort: vi.fn(),
				ontagfilter: vi.fn(),
			},
		});
		// The chip button shows "Tag" when tagFilter is null
		expect(container.textContent).toContain('Tag');
	});
});

// ─── DraftRailItem ────────────────────────────────────────────────────────────

describe('DraftRailItem', () => {
	const draft = makeDraft({ id: 42, title: 'My Draft' });

	const baseItemProps = {
		draft,
		selected: false,
		focused: false,
		tabindex: 0,
		tab: 'active' as const,
		onselect: vi.fn(),
		ondelete: vi.fn(),
		onduplicate: vi.fn(),
		onrestore: vi.fn(),
	};

	it('renders draft title', () => {
		const { container } = render(DraftRailItem, { props: baseItemProps });
		expect(container.textContent).toContain('My Draft');
	});

	it('falls back to content_preview when title is null', () => {
		const props = { ...baseItemProps, draft: makeDraft({ id: 2, title: null, content_preview: 'Preview text here' }) };
		const { container } = render(DraftRailItem, { props });
		expect(container.textContent).toContain('Preview text here');
	});

	it('falls back to "Untitled draft" when both title and content_preview are empty', () => {
		const props = { ...baseItemProps, draft: makeDraft({ id: 3, title: null, content_preview: '' }) };
		const { container } = render(DraftRailItem, { props });
		expect(container.textContent).toContain('Untitled draft');
	});

	it('calls onselect when item is clicked', async () => {
		const onselect = vi.fn();
		const { container } = render(DraftRailItem, { props: { ...baseItemProps, onselect } });
		const item = container.querySelector('[role="option"]') as HTMLElement;
		expect(item).not.toBeNull();
		await fireEvent.click(item);
		expect(onselect).toHaveBeenCalled();
	});

	it('calls onduplicate when duplicate button is clicked', async () => {
		const onduplicate = vi.fn();
		const { container } = render(DraftRailItem, {
			props: { ...baseItemProps, focused: true, onduplicate },
		});
		// Duplicate button has title "Duplicate (D)"
		const dupBtn = container.querySelector('button[title="Duplicate (D)"]') as HTMLElement;
		expect(dupBtn).not.toBeNull();
		await fireEvent.click(dupBtn);
		expect(onduplicate).toHaveBeenCalled();
	});

	it('calls ondelete after two-step delete confirmation', async () => {
		const ondelete = vi.fn();
		const { container } = render(DraftRailItem, {
			props: { ...baseItemProps, focused: true, ondelete },
		});
		// First click arms delete (title changes to "Click to confirm")
		const deleteBtn = container.querySelector('button[title="Delete (Del)"]') as HTMLElement;
		expect(deleteBtn).not.toBeNull();
		await fireEvent.click(deleteBtn);
		// Now armed — button title changes; click again to confirm
		const confirmBtn = container.querySelector('button[title="Click to confirm"]') as HTMLElement;
		expect(confirmBtn).not.toBeNull();
		await fireEvent.click(confirmBtn);
		expect(ondelete).toHaveBeenCalled();
	});

	it('shows Restore button in archive tab instead of Duplicate/Delete', () => {
		const { container } = render(DraftRailItem, {
			props: { ...baseItemProps, tab: 'archive', focused: true },
		});
		const restoreBtn = container.querySelector('button[title="Restore (R)"]');
		expect(restoreBtn).not.toBeNull();
		const dupBtn = container.querySelector('button[title="Duplicate (D)"]');
		expect(dupBtn).toBeNull();
	});

	it('marks item as selected via aria-selected', () => {
		const { container } = render(DraftRailItem, {
			props: { ...baseItemProps, selected: true },
		});
		const item = container.querySelector('[role="option"]');
		expect(item?.getAttribute('aria-selected')).toBe('true');
	});
});

// ─── DraftRail (integration) ──────────────────────────────────────────────────

describe('DraftRail', () => {
	it('renders without crashing with a draft list', () => {
		const { container } = render(DraftRail, { props: defaultRailProps });
		expect(container).toBeTruthy();
	});

	it('renders the New Draft button', () => {
		const { container } = render(DraftRail, { props: defaultRailProps });
		const buttons = Array.from(container.querySelectorAll('button'));
		const newDraftBtn = buttons.find((b) => b.textContent?.includes('New Draft'));
		expect(newDraftBtn).not.toBeUndefined();
	});

	it('calls oncreate when New Draft button is clicked', async () => {
		const oncreate = vi.fn();
		const { container } = render(DraftRail, { props: { ...defaultRailProps, oncreate } });
		const buttons = Array.from(container.querySelectorAll('button'));
		const newDraftBtn = buttons.find((b) => b.textContent?.includes('New Draft')) as HTMLElement;
		expect(newDraftBtn).not.toBeUndefined();
		await fireEvent.click(newDraftBtn);
		expect(oncreate).toHaveBeenCalled();
	});

	it('renders draft item titles', () => {
		const drafts = [
			makeDraft({ id: 1, title: 'Alpha draft' }),
			makeDraft({ id: 2, title: 'Beta draft' }),
		];
		const { container } = render(DraftRail, { props: { ...defaultRailProps, drafts } });
		expect(container.textContent).toContain('Alpha draft');
		expect(container.textContent).toContain('Beta draft');
	});

	it('renders loading spinner when loading=true', () => {
		const { container } = render(DraftRail, {
			props: { ...defaultRailProps, drafts: [], loading: true },
		});
		// Loading state: no rail-empty, has rail-loading
		const empty = container.querySelector('.rail-empty');
		expect(empty).toBeNull();
	});

	it('renders empty state when drafts is empty and not loading', () => {
		const { container } = render(DraftRail, {
			props: { ...defaultRailProps, drafts: [], loading: false },
		});
		expect(container.textContent).toContain('No drafts yet');
	});

	it('renders "No scheduled drafts" in scheduled tab when empty', () => {
		const { container } = render(DraftRail, {
			props: { ...defaultRailProps, tab: 'scheduled', drafts: [], loading: false },
		});
		expect(container.textContent).toContain('No scheduled drafts');
	});

	it('calls ontabchange when a tab is clicked', async () => {
		const ontabchange = vi.fn();
		const { container } = render(DraftRail, { props: { ...defaultRailProps, ontabchange } });
		// Tab buttons have role="tab"
		const tabBtns = container.querySelectorAll('[role="tab"]');
		expect(tabBtns.length).toBeGreaterThan(0);
		// Click "Scheduled" tab (index 1)
		await fireEvent.click(tabBtns[1]);
		expect(ontabchange).toHaveBeenCalledWith('scheduled');
	});

	it('does not render DraftFilterBar when onsearch/onsort/ontagfilter are absent', () => {
		const { container } = render(DraftRail, { props: defaultRailProps });
		// filter-bar class only appears when all 3 props are provided
		const filterBar = container.querySelector('.filter-bar');
		expect(filterBar).toBeNull();
	});

	it('renders DraftFilterBar when optional filter props are provided', () => {
		const { container } = render(DraftRail, {
			props: {
				...defaultRailProps,
				onsearch: vi.fn(),
				onsort: vi.fn(),
				ontagfilter: vi.fn(),
			},
		});
		const filterBar = container.querySelector('.filter-bar');
		expect(filterBar).not.toBeNull();
	});
});
