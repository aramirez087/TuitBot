<script lang="ts">
	import { CheckCircle } from 'lucide-svelte';
	import ApprovalCard from '$lib/components/ApprovalCard.svelte';
	import {
		items,
		loading,
		isEmpty,
		selectedStatus,
		focusedIndex,
		approveItem,
		rejectItem
	} from '$lib/stores/approval';

	interface Props {
		editingId: number | null;
		timezone: string;
		onStartEdit: (id: number) => void;
		onSaveEdit: (id: number, content: string) => void;
		onCancelEdit: () => void;
	}

	const { editingId, timezone, onStartEdit, onSaveEdit, onCancelEdit }: Props = $props();
</script>

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
					<p class="empty-hint">
						New items will appear here when automation generates content or you schedule
						posts with approval enabled.
					</p>
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
						{timezone}
						onApprove={approveItem}
						onReject={rejectItem}
						{onStartEdit}
						{onSaveEdit}
						{onCancelEdit}
					/>
				</div>
			{/each}
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
