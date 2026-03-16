<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Download } from 'lucide-svelte';
	import { api } from '$lib/api';
	import ApprovalStats from '$lib/components/ApprovalStats.svelte';
	import ApprovalFilters from '$lib/components/ApprovalFilters.svelte';
	import ApprovalBulkActions from '$lib/components/ApprovalBulkActions.svelte';
	import ApprovalQueueFeed from './ApprovalQueueFeed.svelte';
	import { accountTimezone, loadSchedule } from '$lib/stores/calendar';
	import {
		stats,
		loading,
		error,
		selectedStatus,
		selectedType,
		reviewerFilter,
		dateFilter,
		currentPage,
		searchQuery,
		focusedIndex,
		focusedItem,
		pendingCount,
		loadItems,
		loadStats,
		editItem,
		approveItem,
		rejectItem,
		approveAllItems,
		setStatusFilter,
		setTypeFilter,
		setReviewerFilter,
		setDateFilter,
		setCurrentPage,
		setSearchQuery,
		moveFocus,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/approval';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

	let editingId = $state<number | null>(null);
	let exportOpen = $state(false);

	// Sync URL params to store on load and URL change
	$effect(() => {
		const p = $page.url.searchParams;
		const newStatus = p.get('status') ?? 'pending';
		const newType = p.get('type') ?? 'all';
		const newReviewer = p.get('reviewer') ?? '';
		const newDate = p.get('date') ?? 'all';
		const newPage = parseInt(p.get('page') ?? '1', 10);
		const newSearch = p.get('search') ?? '';

		// Update store if URL differs
		if ($selectedStatus !== newStatus) setStatusFilter(newStatus);
		if ($selectedType !== newType) setTypeFilter(newType);
		if ($reviewerFilter !== newReviewer) setReviewerFilter(newReviewer);
		if ($dateFilter !== newDate) setDateFilter(newDate);
		if ($currentPage !== newPage) setCurrentPage(newPage);
		if ($searchQuery !== newSearch) setSearchQuery(newSearch);
	});

	// Sync store changes to URL
	$effect(() => {
		const params = new URLSearchParams();
		if ($selectedStatus !== 'pending') params.set('status', $selectedStatus);
		if ($selectedType !== 'all') params.set('type', $selectedType);
		if ($reviewerFilter) params.set('reviewer', $reviewerFilter);
		if ($dateFilter !== 'all') params.set('date', $dateFilter);
		if ($currentPage !== 1) params.set('page', String($currentPage));
		if ($searchQuery) params.set('search', $searchQuery);

		const newUrl = params.toString() ? `/approval?${params.toString()}` : '/approval';
		const currentUrl = $page.url.pathname + $page.url.search;
		if (currentUrl !== newUrl) {
			goto(newUrl, { replaceState: true });
		}
	});

	function triggerExport(format: 'csv' | 'json') {
		const status = $selectedStatus === 'all' ? undefined : $selectedStatus;
		const type_ = $selectedType === 'all' ? undefined : $selectedType;
		const url = api.approval.exportUrl(format, status, type_);
		const a = document.createElement('a');
		a.href = url;
		a.download = `approval_export.${format}`;
		a.click();
		exportOpen = false;
	}

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
		loadSchedule();
		startAutoRefresh();
		window.addEventListener('keydown', handleKeydown);
		const handler = () => { loadItems(true); loadStats(); };
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
			window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		};
	});

	onDestroy(() => {
		stopAutoRefresh();
	});
</script>

<svelte:head>
	<title>Approval — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<div class="page-header-row">
		<div>
			<h1>Approval</h1>
			<p class="subtitle">Review and approve queued actions. Scheduled items will post at their intended time.</p>
		</div>
		<div class="header-actions">
			<div class="export-wrapper">
				<button class="export-btn" onclick={() => (exportOpen = !exportOpen)}>
					<Download size={14} />
					Export
				</button>
				{#if exportOpen}
					<div class="export-menu">
						<button onclick={() => triggerExport('csv')}>Export CSV</button>
						<button onclick={() => triggerExport('json')}>Export JSON</button>
					</div>
				{/if}
			</div>
			{#if $selectedStatus === 'pending' && $pendingCount > 0}
				<ApprovalBulkActions pendingCount={$pendingCount} maxBatch={25} onApproveAll={approveAllItems} />
			{/if}
		</div>
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
		reviewerFilter={$reviewerFilter}
		dateFilter={$dateFilter}
		onStatusChange={setStatusFilter}
		onTypeChange={setTypeFilter}
		onReviewerChange={setReviewerFilter}
		onDateChange={setDateFilter}
	/>
</div>

<ApprovalQueueFeed
	{editingId}
	timezone={$accountTimezone}
	onStartEdit={(id) => (editingId = id)}
	onSaveEdit={handleSaveEdit}
	onCancelEdit={() => (editingId = null)}
/>

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

	.header-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.export-wrapper {
		position: relative;
	}

	.export-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.export-btn:hover {
		background: var(--color-surface-hover);
	}

	.export-menu {
		position: absolute;
		right: 0;
		top: 100%;
		margin-top: 4px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		overflow: hidden;
		z-index: 10;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}

	.export-menu button {
		display: block;
		width: 100%;
		padding: 8px 16px;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		text-align: left;
		cursor: pointer;
		white-space: nowrap;
	}

	.export-menu button:hover {
		background: var(--color-surface-hover);
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

	@media (max-width: 640px) {
		.page-header-row {
			flex-direction: column;
		}
	}
</style>
