<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open,
		mobile = false,
		onclose,
		children
	}: {
		open: boolean;
		mobile?: boolean;
		onclose: () => void;
		children: Snippet;
	} = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onclose();
	}

	function handleBackdropKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			onclose();
		}
	}
</script>

{#if mobile && open}
	<div
		class="inspector-backdrop"
		onclick={handleBackdropClick}
		onkeydown={handleBackdropKeydown}
		role="presentation"
	>
		<div
			class="inspector-drawer"
			role="complementary"
			aria-label="Composer inspector"
		>
			<div class="drawer-handle-area">
				<div class="drawer-handle"></div>
			</div>
			<div class="inspector-scroll">
				{@render children()}
			</div>
		</div>
	</div>
{/if}

<style>
	.inspector-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		z-index: 1099;
		animation: fade-in 0.15s ease;
	}

	.inspector-drawer {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		max-height: 60vh;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border);
		border-radius: 12px 12px 0 0;
		z-index: 1100;
		display: flex;
		flex-direction: column;
		box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.3);
		animation: slide-up 0.2s ease;
	}

	.drawer-handle-area {
		display: flex;
		justify-content: center;
		padding: 8px 0 4px;
		flex-shrink: 0;
		cursor: grab;
	}

	.drawer-handle {
		width: 36px;
		height: 4px;
		border-radius: 2px;
		background: var(--color-border);
	}

	.inspector-scroll {
		overflow-y: auto;
		padding: 4px 16px 16px;
		padding-bottom: calc(16px + env(safe-area-inset-bottom, 0px));
		flex: 1;
		min-height: 0;
	}

	@keyframes fade-in {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	@keyframes slide-up {
		from { transform: translateY(100%); }
		to { transform: translateY(0); }
	}

	@media (prefers-reduced-motion: reduce) {
		.inspector-backdrop,
		.inspector-drawer {
			animation: none;
		}
	}
</style>
