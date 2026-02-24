<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { CheckCircle } from 'lucide-svelte';
	import ApprovalStats from '$lib/components/ApprovalStats.svelte';
	import ApprovalFilters from '$lib/components/ApprovalFilters.svelte';
	import ApprovalCard from '$lib/components/ApprovalCard.svelte';
	import BulkActions from '$lib/components/BulkActions.svelte';
	import {
		items,
		stats,
		loading,
		error,
		selectedStatus,
		selectedType,
		focusedIndex,
		focusedItem,
		isEmpty,
		pendingCount,
		loadItems,
		loadStats,
		approveItem,
		rejectItem,
		editItem,
		approveAllItems,
		setStatusFilter,
		setTypeFilter,
		moveFocus,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/approval';

	let editingId = $state<number | null>(null);

	async function handleSaveEdit(id: number, content: string) {
		await editItem(id, content);
		editingId = null;
	}

	function handleKeydown(e: KeyboardEvent) {
		// Don't intercept when editing.
		if (editingId !== null) {
			if (e.key === 'Escape') {
				editingId = null;
			}
			return;
		}
		// Don't intercept when focus is in input/textarea/select.
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

		switch (e.key) {
			case 'j':
			case 'ArrowDown':
				e.preventDefault();
				moveFocus(1);
				break;
			case 'k':
			case 'ArrowUp':
				e.preventDefault();
				moveFocus(-1);
				break;
			case 'a': {
				const item = $focusedItem;
				if (item && item.status === 'pending') approveItem(item.id);
				break;
			}
			case 'r': {
				const item = $focusedItem;
				if (item && item.status === 'pending') rejectItem(item.id);
				break;
			}
			case 'e': {
				const item = $focusedItem;
				if (item && item.status === 'pending') editingId = item.id;
				break;
			}
		}
	}

	$effect(() => {
		// Scroll focused card into view.
		const idx = $focusedIndex;
		const el = document.querySelector(`[data-approval-index="${idx}"]`);
		el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
	});

	onMount(() => {
		loadItems(true);
		loadStats();
		startAutoRefresh();
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		stopAutoRefresh();
		window.removeEventListener('keydown', handleKeydown);
	});
</script>

<svelte:head>
	<title>Approval — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<div class="page-header-row">
		<div>
			<h1>Approval</h1>
			<p class="subtitle">Review and approve queued actions</p>
		</div>
		{#if $selectedStatus === 'pending' && $pendingCount > 0}
			<BulkActions pendingCount={$pendingCount} onApproveAll={approveAllItems} />
		{/if}
	</div>
	<div class="page-header-stats">
		<ApprovalStats stats={$stats} />
	</div>
</div>

{#if $error}
	<div class="error-banner">
		<span>{$error}</span>
		<button onclick={() => loadItems(true)}>Retry</button>
	</div>
{/if}

<div class="filters-section">
	<ApprovalFilters
		selectedStatus={$selectedStatus}
		selectedType={$selectedType}
		onStatusChange={setStatusFilter}
		onTypeChange={setTypeFilter}
	/>
</div>

<div class="queue-section">
	{#if $loading && $items.length === 0}
		<div class="feed-container">
			{#each { length: 5 } as _}
				<div class="skeleton-item"></div>
			{/each}
		</div>
	{:else if $isEmpty}
		<div class="feed-container">
			<div class="empty-state">
				<div class="empty-icon">
					<CheckCircle size={32} />
				</div>
				{#if $selectedStatus === 'pending'}
					<p class="empty-title">No pending items — you're all caught up!</p>
					<p class="empty-hint">New items will appear here when the automation generates content.</p>
				{:else}
					<p class="empty-title">No {$selectedStatus} items</p>
					<p class="empty-hint">Try a different filter to see more items.</p>
				{/if}
			</div>
		</div>
	{:else}
		<div class="feed-container">
			{#each $items as item, i (item.id)}
				<div data-approval-index={i}>
					<ApprovalCard
						{item}
						focused={$focusedIndex === i}
						editing={editingId === item.id}
						onApprove={approveItem}
						onReject={rejectItem}
						onStartEdit={(id) => (editingId = id)}
						onSaveEdit={handleSaveEdit}
						onCancelEdit={() => (editingId = null)}
					/>
				</div>
			{/each}
		</div>
	{/if}
</div>

<div class="keyboard-hints">
	<kbd>j</kbd><kbd>k</kbd> navigate
	<span class="hint-sep">&middot;</span>
	<kbd>a</kbd> approve
	<span class="hint-sep">&middot;</span>
	<kbd>r</kbd> reject
	<span class="hint-sep">&middot;</span>
	<kbd>e</kbd> edit
	<span class="hint-sep">&middot;</span>
	<kbd>esc</kbd> cancel
</div>

<style>
	.page-header {
		margin-bottom: 24px;
	}

	.page-header-row {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 16px;
		margin-bottom: 8px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.page-header-stats {
		margin-top: 4px;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid var(--color-danger);
		border-radius: 8px;
		margin-bottom: 20px;
		color: var(--color-danger);
		font-size: 13px;
	}

	.error-banner button {
		padding: 4px 12px;
		border: 1px solid var(--color-danger);
		border-radius: 4px;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
	}

	.error-banner button:hover {
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.filters-section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 16px 20px;
		margin-bottom: 20px;
	}

	.queue-section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 0;
		overflow: hidden;
	}

	.feed-container {
		background-color: var(--color-base);
		overflow: hidden;
	}

	.empty-state {
		padding: 60px 20px;
		text-align: center;
	}

	.empty-icon {
		color: var(--color-success);
		margin-bottom: 12px;
		opacity: 0.6;
	}

	.empty-title {
		margin: 0 0 6px;
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.empty-hint {
		margin: 0;
		font-size: 13px;
		color: var(--color-text-subtle);
	}

	.keyboard-hints {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		margin-top: 16px;
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.keyboard-hints kbd {
		display: inline-block;
		padding: 1px 5px;
		border: 1px solid var(--color-border);
		border-radius: 3px;
		background-color: var(--color-surface);
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-text-muted);
	}

	.hint-sep {
		color: var(--color-border);
	}

	.skeleton-item {
		height: 120px;
		border-bottom: 1px solid var(--color-border-subtle);
		background-color: var(--color-surface-active);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-item:last-child {
		border-bottom: none;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.4;
		}
	}

	@media (max-width: 640px) {
		.page-header-row {
			flex-direction: column;
		}
	}
</style>
