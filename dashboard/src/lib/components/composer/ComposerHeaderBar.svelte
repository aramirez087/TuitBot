<script lang="ts">
	import { X, Maximize2, Minimize2 } from 'lucide-svelte';

	let {
		focusMode,
		ontogglefocus,
		onclose
	}: {
		focusMode: boolean;
		ontogglefocus: () => void;
		onclose: () => void;
	} = $props();
</script>

<div class="header-bar">
	<button
		class="header-btn close-btn"
		onclick={onclose}
		aria-label="Close compose modal"
	>
		<X size={16} />
	</button>

	<div class="spacer"></div>

	<button
		class="header-btn focus-btn"
		onclick={ontogglefocus}
		aria-label={focusMode ? 'Exit focus mode' : 'Enter focus mode'}
		title={focusMode ? 'Exit focus mode (\u2318\u21E7F)' : 'Focus mode (\u2318\u21E7F)'}
	>
		{#if focusMode}
			<Minimize2 size={14} />
		{:else}
			<Maximize2 size={14} />
		{/if}
	</button>
</div>

<style>
	.header-bar {
		display: flex;
		align-items: center;
		padding: 8px 16px;
		flex-shrink: 0;
	}

	.spacer {
		flex: 1;
	}

	.header-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.header-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	@media (pointer: coarse) {
		.header-btn {
			min-width: 44px;
			min-height: 44px;
		}
	}
</style>
