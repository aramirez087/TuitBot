<script lang="ts">
	import { X, Maximize2, Minimize2, PanelRight, Eye, EyeOff } from 'lucide-svelte';

	let {
		focusMode,
		inspectorOpen = false,
		previewVisible = false,
		ontogglefocus,
		ontoggleinspector,
		ontogglepreview,
		onclose
	}: {
		focusMode: boolean;
		inspectorOpen?: boolean;
		previewVisible?: boolean;
		ontogglefocus: () => void;
		ontoggleinspector?: () => void;
		ontogglepreview?: () => void;
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

	{#if ontogglepreview}
		<button
			class="header-btn"
			class:active={previewVisible}
			onclick={ontogglepreview}
			aria-label={previewVisible ? 'Hide preview' : 'Show preview'}
			title={previewVisible ? 'Hide preview (\u2318\u21E7P)' : 'Show preview (\u2318\u21E7P)'}
		>
			{#if previewVisible}
				<Eye size={14} />
			{:else}
				<EyeOff size={14} />
			{/if}
		</button>
	{/if}

	{#if ontoggleinspector}
		<button
			class="header-btn"
			class:active={inspectorOpen}
			onclick={ontoggleinspector}
			aria-label={inspectorOpen ? 'Close inspector' : 'Open inspector'}
			title={inspectorOpen ? 'Close inspector (\u2318I)' : 'Open inspector (\u2318I)'}
		>
			<PanelRight size={14} />
		</button>
	{/if}

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
		gap: 2px;
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

	.header-btn.active {
		color: var(--color-accent);
	}

	.header-btn.active:hover {
		color: var(--color-accent-hover);
	}

	@media (pointer: coarse) {
		.header-btn {
			min-width: 44px;
			min-height: 44px;
		}
	}
</style>
