<script lang="ts">
	import { CheckCircle } from 'lucide-svelte';

	interface Props {
		pendingCount: number;
		maxBatch: number;
		onApproveAll: () => void;
	}

	let { pendingCount, maxBatch = 25, onApproveAll }: Props = $props();
	let showConfirm = $state(false);

	const approveCount = $derived(Math.min(pendingCount, maxBatch));
	const isCapped = $derived(pendingCount > maxBatch);

	function handleConfirm() {
		showConfirm = false;
		onApproveAll();
	}
</script>

<div class="bulk-actions">
	{#if !showConfirm}
		<button
			class="approve-all-btn"
			onclick={() => (showConfirm = true)}
			disabled={pendingCount === 0}
		>
			<CheckCircle size={16} />
			{#if isCapped}
				Approve {approveCount} of {pendingCount}
			{:else}
				Approve All ({pendingCount})
			{/if}
		</button>
	{:else}
		<div class="confirm-dialog">
			<span class="confirm-text">
				{#if isCapped}
					Approve {approveCount} of {pendingCount} pending items? (max batch: {maxBatch})
				{:else}
					Approve all {pendingCount} pending items?
				{/if}
			</span>
			<div class="confirm-actions">
				<button class="confirm-btn" onclick={handleConfirm}>Confirm</button>
				<button class="cancel-btn" onclick={() => (showConfirm = false)}>Cancel</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.bulk-actions {
		display: flex;
		align-items: center;
	}

	.approve-all-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border: 1px solid var(--color-success);
		border-radius: 6px;
		background-color: transparent;
		color: var(--color-success);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.approve-all-btn:hover:not(:disabled) {
		background-color: color-mix(in srgb, var(--color-success) 10%, transparent);
	}

	.approve-all-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.confirm-dialog {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		border: 1px solid var(--color-warning);
		border-radius: 6px;
		background-color: color-mix(in srgb, var(--color-warning) 8%, transparent);
	}

	.confirm-text {
		font-size: 13px;
		color: var(--color-text);
		white-space: nowrap;
	}

	.confirm-actions {
		display: flex;
		gap: 6px;
	}

	.confirm-btn {
		padding: 4px 12px;
		border: 1px solid var(--color-success);
		border-radius: 4px;
		background-color: var(--color-success);
		color: white;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: opacity 0.15s ease;
	}

	.confirm-btn:hover {
		opacity: 0.9;
	}

	.cancel-btn {
		padding: 4px 12px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background-color: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.cancel-btn:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}
</style>
