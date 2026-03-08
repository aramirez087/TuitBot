<script lang="ts">
	import { Plus, Loader2, Undo2 } from 'lucide-svelte';
	import type { DraftSummary, ContentTag } from '$lib/api/types';
	import DraftRailItem from './DraftRailItem.svelte';
	import DraftFilterBar from './DraftFilterBar.svelte';

	type TabKey = 'active' | 'scheduled' | 'posted' | 'archive';

	let {
		drafts,
		selectedId,
		tab,
		tabCounts,
		loading,
		searchQuery = '',
		sortBy = 'updated',
		tagFilter = null,
		accountTags = [],
		onselect,
		ontabchange,
		oncreate,
		onarchive,
		onduplicate,
		onrestore,
		onsearch,
		onsort,
		ontagfilter
	}: {
		drafts: DraftSummary[];
		selectedId: number | null;
		tab: TabKey;
		tabCounts: { active: number; scheduled: number; posted: number; archive: number };
		loading: boolean;
		searchQuery?: string;
		sortBy?: string;
		tagFilter?: number | null;
		accountTags?: ContentTag[];
		onselect: (id: number) => void;
		ontabchange: (tab: TabKey) => void;
		oncreate: () => void;
		onarchive: (id: number) => void;
		onduplicate: (id: number) => void;
		onrestore: (id: number) => void;
		onsearch?: (q: string) => void;
		onsort?: (by: string) => void;
		ontagfilter?: (tagId: number | null) => void;
	} = $props();

	let filterBarComponent: DraftFilterBar | undefined = $state();

	const tabs: Array<{ key: TabKey; label: string }> = [
		{ key: 'active', label: 'Drafts' },
		{ key: 'scheduled', label: 'Scheduled' },
		{ key: 'posted', label: 'Posted' },
		{ key: 'archive', label: 'Archive' }
	];

	let focusedIndex = $state(0);
	let lastArchived = $state<{
		id: number;
		title: string;
		timeout: ReturnType<typeof setTimeout>;
	} | null>(null);
	let listEl: HTMLDivElement | undefined = $state();
	let itemEls: DraftRailItem[] = $state([]);
	let newDraftBtnEl: HTMLButtonElement | undefined = $state();

	// Clamp focusedIndex when the list shrinks
	$effect(() => {
		if (drafts.length === 0) return;
		if (focusedIndex >= drafts.length) {
			focusedIndex = Math.max(0, drafts.length - 1);
		}
	});

	// Scroll focused item into view
	$effect(() => {
		const item = itemEls[focusedIndex];
		if (item) item.scrollIntoViewIfNeeded();
	});

	// Reset focusedIndex on tab change
	$effect(() => {
		void tab;
		focusedIndex = 0;
	});

	// Clear undo toast on tab change
	$effect(() => {
		void tab;
		clearUndoToast();
	});

	function clearUndoToast() {
		if (lastArchived) {
			clearTimeout(lastArchived.timeout);
			lastArchived = null;
		}
	}

	function handleArchiveFocused() {
		const draft = drafts[focusedIndex];
		if (!draft) return;
		handleArchiveItem(draft.id);
	}

	function handleArchiveItem(id: number) {
		const draft = drafts.find((d) => d.id === id);
		if (!draft) return;
		const title = draft.title ?? draft.content_preview?.trim() ?? 'Untitled draft';

		clearUndoToast();
		onarchive(id);

		const timeout = setTimeout(() => {
			lastArchived = null;
		}, 5000);
		lastArchived = { id, title, timeout };
	}

	function handleUndo() {
		if (!lastArchived) return;
		onrestore(lastArchived.id);
		clearUndoToast();
	}

	function handleListKeydown(e: KeyboardEvent) {
		// Ignore if focus is inside a button (action buttons handle their own events)
		const target = e.target as HTMLElement;
		if (target.tagName === 'BUTTON' && target.closest('[data-rail-actions]')) return;

		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				focusedIndex = Math.min(focusedIndex + 1, drafts.length - 1);
				itemEls[focusedIndex]?.focus?.();
				break;
			case 'ArrowUp':
				e.preventDefault();
				focusedIndex = Math.max(focusedIndex - 1, 0);
				itemEls[focusedIndex]?.focus?.();
				break;
			case 'Enter':
				e.preventDefault();
				if (drafts[focusedIndex]) onselect(drafts[focusedIndex].id);
				break;
			case 'n':
			case 'N':
				if (!e.metaKey && !e.ctrlKey && !e.altKey) {
					e.preventDefault();
					oncreate();
				}
				break;
			case 'Backspace':
			case 'Delete':
				if (tab !== 'archive' && tab !== 'posted') {
					e.preventDefault();
					handleArchiveFocused();
				}
				break;
			case 'd':
			case 'D':
				if (!e.metaKey && !e.ctrlKey && !e.altKey && drafts[focusedIndex]) {
					e.preventDefault();
					onduplicate(drafts[focusedIndex].id);
				}
				break;
			case 'r':
			case 'R':
				if (tab === 'archive' && !e.metaKey && !e.ctrlKey && !e.altKey && drafts[focusedIndex]) {
					e.preventDefault();
					onrestore(drafts[focusedIndex].id);
				}
				break;
			case '1': ontabchange('active'); break;
			case '2': ontabchange('scheduled'); break;
			case '3': ontabchange('posted'); break;
			case '4': ontabchange('archive'); break;
			case '/':
				if (!e.metaKey && !e.ctrlKey && !e.altKey) {
					e.preventDefault();
					filterBarComponent?.focusSearch();
				}
				break;
		}
	}

	export function focus() {
		if (drafts.length > 0 && itemEls[focusedIndex]) {
			itemEls[focusedIndex].focus?.();
		} else if (newDraftBtnEl) {
			newDraftBtnEl.focus();
		}
	}

	const emptyMessage = $derived(
		tab === 'archive'
			? 'No archived drafts'
			: tab === 'scheduled'
				? 'No scheduled drafts'
				: tab === 'posted'
					? 'Posted drafts appear here'
					: 'No drafts yet'
	);
</script>

<div class="rail">
	<div class="rail-tabs" role="tablist" aria-label="Draft tabs">
		{#each tabs as t}
			<button
				class="tab-btn"
				class:active={tab === t.key}
				type="button"
				role="tab"
				aria-selected={tab === t.key}
				onclick={() => ontabchange(t.key)}
			>
				{t.label}
				<span class="tab-count">{tabCounts[t.key]}</span>
			</button>
		{/each}
	</div>

	{#if onsearch && onsort && ontagfilter}
		<DraftFilterBar
			bind:this={filterBarComponent}
			{searchQuery}
			{sortBy}
			{tagFilter}
			tags={accountTags}
			{onsearch}
			{onsort}
			{ontagfilter}
		/>
	{/if}

	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="rail-list"
		role="listbox"
		tabindex="-1"
		aria-label="{tabs.find(t => t.key === tab)?.label ?? 'Drafts'} list"
		bind:this={listEl}
		onkeydown={handleListKeydown}
	>
		{#if loading}
			<div class="rail-loading">
				<Loader2 size={16} class="spinner" />
			</div>
		{:else if drafts.length === 0}
			<div class="rail-empty">
				<span class="rail-empty-text">{emptyMessage}</span>
			</div>
		{:else}
			{#each drafts as draft, i (draft.id)}
				<DraftRailItem
					{draft}
					selected={selectedId === draft.id}
					focused={focusedIndex === i}
					tabindex={focusedIndex === i ? 0 : -1}
					{tab}
					onselect={() => { focusedIndex = i; onselect(draft.id); }}
					onarchive={() => handleArchiveItem(draft.id)}
					onduplicate={() => onduplicate(draft.id)}
					onrestore={() => onrestore(draft.id)}
					bind:this={itemEls[i]}
				/>
			{/each}
		{/if}
	</div>

	{#if lastArchived}
		<div class="undo-toast" role="status" aria-live="polite">
			<span class="undo-text">Draft archived</span>
			<button class="undo-btn" type="button" onclick={handleUndo}>
				<Undo2 size={13} />
				Undo
			</button>
		</div>
	{/if}

	<div class="rail-footer">
		<button
			class="new-draft-btn"
			type="button"
			onclick={oncreate}
			bind:this={newDraftBtnEl}
		>
			<Plus size={16} />
			<span>New Draft</span>
		</button>
	</div>
</div>

<style>
	.rail {
		display: flex;
		flex-direction: column;
		height: 100%;
		border-right: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
	}

	.rail-tabs {
		display: flex;
		flex-shrink: 0;
		border-bottom: 1px solid var(--color-border-subtle);
		padding: 0 4px;
	}

	.tab-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 10px 4px;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.tab-btn:hover {
		color: var(--color-text);
	}

	.tab-btn.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.tab-count {
		font-size: 10px;
		font-weight: 700;
		font-family: var(--font-mono);
		opacity: 0.7;
	}

	.rail-list {
		flex: 1;
		overflow-y: auto;
		padding: 4px;
		outline: none;
	}

	.rail-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 40px 0;
		color: var(--color-text-subtle);
	}

	.rail-loading :global(.spinner) {
		animation: spin 1s linear infinite;
	}

	.rail-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 40px 12px;
	}

	.rail-empty-text {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.undo-toast {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 12px;
		background: var(--color-surface-active);
		border-top: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.undo-text {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.undo-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		border: none;
		background: transparent;
		color: var(--color-accent);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: 4px;
		transition: background 0.1s ease;
	}

	.undo-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.rail-footer {
		flex-shrink: 0;
		padding: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.new-draft-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		width: 100%;
		padding: 8px 12px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.new-draft-btn:hover {
		background: var(--color-accent-hover);
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
