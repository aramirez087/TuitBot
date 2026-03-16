<script lang="ts">
	import { ChevronLeft, ChevronRight } from 'lucide-svelte';

	interface Props {
		currentPage: number;
		totalPages: number;
		totalCount: number;
		pageSize: number;
		onPageChange: (page: number) => void;
	}

	const { currentPage, totalPages, totalCount, pageSize, onPageChange }: Props = $props();

	// $derived ensures reactive recompute when props change (avoids state_referenced_locally warning)
	const start = $derived((currentPage - 1) * pageSize + 1);
	const end = $derived(Math.min(currentPage * pageSize, totalCount));

	function handlePrev() {
		if (currentPage > 1) onPageChange(currentPage - 1);
	}

	function handleNext() {
		if (currentPage < totalPages) onPageChange(currentPage + 1);
	}

	function handlePageInput(e: Event) {
		const val = parseInt((e.target as HTMLInputElement).value, 10);
		if (!isNaN(val) && val >= 1 && val <= totalPages) {
			onPageChange(val);
		}
	}
</script>

<div class="paginator">
	<div class="paginator-info">
		{#if totalCount === 0}
			<span>No items</span>
		{:else}
			<span>
				Showing {start}–{end} of {totalCount}
			</span>
		{/if}
	</div>

	<div class="paginator-controls">
		<button
			class="paginator-btn"
			onclick={handlePrev}
			disabled={currentPage === 1}
			aria-label="Previous page"
		>
			<ChevronLeft size={16} />
		</button>

		<div class="paginator-input-group">
			<input
				type="number"
				class="paginator-input"
				value={totalPages === 0 ? 0 : currentPage}
				min="1"
				max={totalPages}
				disabled={totalPages === 0}
				onchange={handlePageInput}
				aria-label="Go to page"
			/>
			<span class="paginator-divider">/</span>
			<span class="paginator-total">{totalPages}</span>
		</div>

		<button
			class="paginator-btn"
			onclick={handleNext}
			disabled={currentPage === totalPages || totalPages === 0}
			aria-label="Next page"
		>
			<ChevronRight size={16} />
		</button>
	</div>
</div>

<style>
	.paginator {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding: 12px 0;
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.paginator-info {
		flex: 1;
	}

	.paginator-controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.paginator-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		padding: 0;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.paginator-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		border-color: var(--color-border-strong);
	}

	.paginator-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.paginator-input-group {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.paginator-input {
		width: 48px;
		padding: 6px 8px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background: var(--color-surface);
		color: var(--color-text);
		font-size: 13px;
		text-align: center;
	}

	.paginator-input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.paginator-divider {
		color: var(--color-text-muted);
	}

	.paginator-total {
		color: var(--color-text-muted);
		min-width: 30px;
	}

	@media (max-width: 640px) {
		.paginator {
			flex-direction: column;
			gap: 12px;
		}

		.paginator-info {
			width: 100%;
			text-align: center;
		}

		.paginator-controls {
			width: 100%;
			justify-content: center;
		}
	}
</style>
