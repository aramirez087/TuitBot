<script lang="ts">
	import { Copy, Scissors, Merge, Trash2 } from 'lucide-svelte';

	let {
		index,
		total,
		onduplicate,
		onsplit,
		onmerge,
		onremove
	}: {
		index: number;
		total: number;
		onduplicate: () => void;
		onsplit: () => void;
		onmerge: () => void;
		onremove: () => void;
	} = $props();
</script>

<div class="card-actions">
	<button
		class="action-btn"
		onclick={onduplicate}
		title="Duplicate (\u2318D)"
		aria-label="Duplicate tweet {index + 1}"
	>
		<Copy size={12} />
	</button>
	<button
		class="action-btn"
		onclick={onsplit}
		title="Split at cursor (\u2318\u21E7S)"
		aria-label="Split tweet {index + 1}"
	>
		<Scissors size={12} />
	</button>
	{#if index < total - 1 && total > 2}
		<button
			class="action-btn"
			onclick={onmerge}
			title="Merge with next (\u2318\u21E7M)"
			aria-label="Merge tweet {index + 1} with tweet {index + 2}"
		>
			<Merge size={12} />
		</button>
	{/if}
	{#if total > 2}
		<button
			class="remove-card-btn"
			onclick={onremove}
			aria-label="Remove tweet {index + 1}"
		>
			<Trash2 size={12} />
		</button>
	{/if}
</div>

<style>
	.card-actions {
		display: flex;
		gap: 2px;
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	:global(.tweet-card:hover) .card-actions,
	:global(.tweet-card.focused) .card-actions {
		opacity: 1;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.remove-card-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.remove-card-btn:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	/* Card actions always visible on touch devices */
	@media (hover: none) {
		.card-actions {
			opacity: 1;
		}
	}

	@media (pointer: coarse) {
		.action-btn,
		.remove-card-btn {
			min-width: 44px;
			min-height: 44px;
		}
	}
</style>
