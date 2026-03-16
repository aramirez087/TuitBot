<script lang="ts">
	import { CheckCircle } from 'lucide-svelte';
	import ApprovalCardSelectable from '$lib/components/ApprovalCardSelectable.svelte';
	import ApprovalPaginator from '$lib/components/ApprovalPaginator.svelte';
	import {
		paginatedItems,
		loading,
		isEmpty,
		selectedStatus,
		focusedIndex,
		currentPage,
		totalCount,
		totalPages,
		pageSize,
		approveItem,
		rejectItem,
		setCurrentPage
	} from '$lib/stores/approval';

	interface Props {
		editingId: number | null;
		timezone: string;
		onStartEdit: (id: number) => void;
		onSaveEdit: (id: number, content: string) => void;
		onCancelEdit: () => void;
	}

	const { editingId, timezone, onStartEdit, onSaveEdit, onCancelEdit }: Props = $props();

	let selectedIds = $state<Set<number>>(new Set());

	function handleSelectionChange(itemId: number, selected: boolean) {
		if (selected) {
			selectedIds.add(itemId);
		} else {
			selectedIds.delete(itemId);
		}
		selectedIds = new Set(selectedIds);
	}

	function handleSelectAll(e: Event) {
		const checked = (e.target as HTMLInputElement).checked;
		selectedIds = checked ? new Set($paginatedItems.map((i) => i.id)) : new Set();
	}

	const allSelected = $derived(
		$paginatedItems.length > 0 && $paginatedItems.every((i) => selectedIds.has(i.id))
	);
</script>

<div class="queue-section">
	{#if $loading && $paginatedItems.length === 0}
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
					<p class="empty-hint">
						New items will appear here when automation generates content or you schedule posts with
						approval enabled.
					</p>
				{:else}
					<p class="empty-title">No {$selectedStatus} items</p>
					<p class="empty-hint">Try a different filter to see more items.</p>
				{/if}
			</div>
		</div>
	{:else}
		<div class="feed-container">
			{#if $paginatedItems.length > 0}
				<div class="feed-header">
					<div class="header-checkbox">
						<input
							type="checkbox"
							checked={allSelected}
							onchange={handleSelectAll}
							aria-label="Select all on this page"
						/>
					</div>
					<span class="header-label">
						{selectedIds.size > 0 ? `${selectedIds.size} selected` : 'Select items'}
					</span>
				</div>
			{/if}

			{#each $paginatedItems as item, i (item.id)}
				<div data-approval-index={i}>
					<ApprovalCardSelectable
						{item}
						focused={$focusedIndex === i}
						editing={editingId === item.id}
						{timezone}
						selected={selectedIds.has(item.id)}
						onApprove={approveItem}
						onReject={rejectItem}
						{onStartEdit}
						{onSaveEdit}
						{onCancelEdit}
						onSelectionChange={handleSelectionChange}
					/>
				</div>
			{/each}

			<div class="feed-footer">
				<ApprovalPaginator
					currentPage={$currentPage}
					totalPages={$totalPages}
					totalCount={$totalCount}
					pageSize={$pageSize}
					onPageChange={setCurrentPage}
				/>
			</div>
		</div>
	{/if}
</div>

<style>
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

	.feed-header {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px;
		border-bottom: 1px solid var(--color-border-subtle);
		background: var(--color-surface-active);
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.header-checkbox {
		display: flex;
		align-items: center;
		padding-left: 12px;
	}

	.header-checkbox input {
		width: 18px;
		height: 18px;
		cursor: pointer;
		accent-color: var(--color-accent);
	}

	.header-label {
		color: var(--color-text);
	}

	.feed-footer {
		padding: 12px;
		border-top: 1px solid var(--color-border-subtle);
		background: var(--color-surface-active);
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
</style>
