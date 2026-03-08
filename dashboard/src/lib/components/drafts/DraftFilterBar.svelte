<script lang="ts">
	import { Search, X, ChevronDown } from 'lucide-svelte';
	import type { ContentTag } from '$lib/api/types';

	let {
		searchQuery,
		sortBy,
		tagFilter,
		tags,
		onsearch,
		onsort,
		ontagfilter
	}: {
		searchQuery: string;
		sortBy: string;
		tagFilter: number | null;
		tags: ContentTag[];
		onsearch: (q: string) => void;
		onsort: (by: string) => void;
		ontagfilter: (tagId: number | null) => void;
	} = $props();

	let searchEl: HTMLInputElement | undefined = $state();
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	let sortOpen = $state(false);
	let tagOpen = $state(false);

	const sortOptions = [
		{ value: 'updated', label: 'Last edited' },
		{ value: 'created', label: 'Created' },
		{ value: 'title', label: 'Title A-Z' },
		{ value: 'scheduled', label: 'Scheduled' }
	];

	const currentSortLabel = $derived(
		sortOptions.find((o) => o.value === sortBy)?.label ?? 'Sort'
	);

	const activeTagName = $derived(
		tagFilter !== null ? tags.find((t) => t.id === tagFilter)?.name ?? 'Tag' : null
	);

	const hasFilters = $derived(searchQuery !== '' || tagFilter !== null || sortBy !== 'updated');

	function handleInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => onsearch(value), 300);
	}

	function clearSearch() {
		if (searchEl) searchEl.value = '';
		onsearch('');
	}

	function clearAll() {
		clearSearch();
		onsort('updated');
		ontagfilter(null);
	}

	function handleSortSelect(value: string) {
		onsort(value);
		sortOpen = false;
	}

	function handleTagSelect(tagId: number | null) {
		ontagfilter(tagId);
		tagOpen = false;
	}

	function handleSortKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') sortOpen = false;
	}

	function handleTagKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') tagOpen = false;
	}

	export function focusSearch() {
		searchEl?.focus();
	}
</script>

<div class="filter-bar">
	<div class="search-row">
		<div class="search-field">
			<Search size={13} class="search-icon" />
			<input
				bind:this={searchEl}
				type="text"
				class="search-input"
				placeholder="Search drafts..."
				value={searchQuery}
				oninput={handleInput}
			/>
			{#if searchQuery}
				<button class="search-clear" type="button" onclick={clearSearch} title="Clear search">
					<X size={12} />
				</button>
			{/if}
		</div>

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="dropdown" onkeydown={handleSortKeydown}>
			<button
				class="dropdown-btn"
				type="button"
				title="Sort by"
				onclick={() => { sortOpen = !sortOpen; tagOpen = false; }}
			>
				<span class="dropdown-label">{currentSortLabel}</span>
				<ChevronDown size={12} />
			</button>
			{#if sortOpen}
				<div class="dropdown-menu" role="listbox" aria-label="Sort options">
					{#each sortOptions as opt}
						<button
							class="dropdown-item"
							class:active={sortBy === opt.value}
							type="button"
							role="option"
							aria-selected={sortBy === opt.value}
							onclick={() => handleSortSelect(opt.value)}
						>
							{opt.label}
						</button>
					{/each}
				</div>
			{/if}
		</div>
	</div>

	{#if tags.length > 0 || hasFilters}
		<div class="filter-row">
			{#if tags.length > 0}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="dropdown tag-dropdown" onkeydown={handleTagKeydown}>
					<button
						class="chip-btn"
						class:active={tagFilter !== null}
						type="button"
						onclick={() => { tagOpen = !tagOpen; sortOpen = false; }}
					>
						{activeTagName ?? 'Tag'}
						<ChevronDown size={11} />
					</button>
					{#if tagOpen}
						<div class="dropdown-menu" role="listbox" aria-label="Tag filter">
							<button
								class="dropdown-item"
								class:active={tagFilter === null}
								type="button"
								role="option"
								aria-selected={tagFilter === null}
								onclick={() => handleTagSelect(null)}
							>
								All tags
							</button>
							{#each tags as tag (tag.id)}
								<button
									class="dropdown-item"
									class:active={tagFilter === tag.id}
									type="button"
									role="option"
									aria-selected={tagFilter === tag.id}
									onclick={() => handleTagSelect(tag.id)}
								>
									{#if tag.color}
										<span class="tag-dot" style="background: {tag.color}"></span>
									{/if}
									{tag.name}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}

			{#if hasFilters}
				<button class="clear-btn" type="button" onclick={clearAll}>
					Clear
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.filter-bar {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px 8px 6px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.search-row {
		display: flex;
		gap: 4px;
		align-items: center;
	}

	.search-field {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 5px;
		background: var(--color-base);
		transition: border-color 0.15s ease;
	}

	.search-field:focus-within {
		border-color: var(--color-accent);
	}

	.search-field :global(.search-icon) {
		flex-shrink: 0;
		color: var(--color-text-subtle);
	}

	.search-input {
		flex: 1;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		outline: none;
		min-width: 0;
	}

	.search-input::placeholder {
		color: var(--color-text-subtle);
	}

	.search-clear {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		padding: 0;
	}

	.search-clear:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.dropdown {
		position: relative;
	}

	.dropdown-btn {
		display: flex;
		align-items: center;
		gap: 3px;
		padding: 5px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 5px;
		background: var(--color-base);
		color: var(--color-text-muted);
		font-size: 11px;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.12s ease;
	}

	.dropdown-btn:hover {
		border-color: var(--color-text-subtle);
		color: var(--color-text);
	}

	.dropdown-label {
		max-width: 80px;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.dropdown-menu {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		min-width: 140px;
		padding: 4px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		z-index: 20;
	}

	.dropdown-item {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 6px 10px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		transition: background 0.1s ease;
	}

	.dropdown-item:hover {
		background: var(--color-surface-hover);
	}

	.dropdown-item.active {
		color: var(--color-accent);
		font-weight: 600;
	}

	.filter-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.tag-dropdown {
		position: relative;
	}

	.chip-btn {
		display: flex;
		align-items: center;
		gap: 3px;
		padding: 3px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 12px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.chip-btn:hover {
		border-color: var(--color-text-subtle);
		color: var(--color-text);
	}

	.chip-btn.active {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.tag-dot {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.clear-btn {
		margin-left: auto;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		padding: 3px 6px;
		border-radius: 4px;
		transition: all 0.1s ease;
	}

	.clear-btn:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}
</style>
