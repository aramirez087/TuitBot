<script lang="ts">
	import { Plus } from 'lucide-svelte';

	let {
		onclick,
		disabled = false
	}: {
		onclick: () => void;
		disabled?: boolean;
	} = $props();
</script>

<div class="divider-row" class:muted={disabled}>
	<div class="divider-line"></div>
	<button
		class="divider-btn"
		{disabled}
		{onclick}
		aria-label="Continue as thread"
		title="Continue as thread"
	>
		<span class="divider-icon"><Plus size={14} /></span>
		<span class="divider-label">Continue thread</span>
	</button>
	<div class="divider-line"></div>
</div>

<style>
	.divider-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 0 4px;
		transition: opacity 0.15s ease;
	}

	.divider-row.muted {
		opacity: 0.35;
	}

	.divider-line {
		flex: 1;
		height: 1px;
		background: color-mix(in srgb, var(--color-border-subtle) 40%, transparent);
	}

	.divider-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
		border-radius: 20px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.divider-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
	}

	.divider-btn:disabled {
		cursor: not-allowed;
	}

	.divider-icon {
		display: flex;
		align-items: center;
		justify-content: center;
	}

	@media (prefers-reduced-motion: reduce) {
		.divider-row,
		.divider-btn {
			transition: none;
		}
	}
</style>
