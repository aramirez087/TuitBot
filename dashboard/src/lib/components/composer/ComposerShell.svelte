<script lang="ts">
	import { focusTrap } from '$lib/actions/focusTrap';
	import type { Snippet } from 'svelte';

	let {
		open,
		focusMode,
		inspectorOpen = false,
		onclose,
		children
	}: {
		open: boolean;
		focusMode: boolean;
		inspectorOpen?: boolean;
		onclose: () => void;
		children: Snippet;
	} = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onclose();
	}
</script>

<div
	class="backdrop"
	onclick={handleBackdropClick}
	role="presentation"
>
	<div
		class="modal"
		class:focus-mode={focusMode}
		class:with-inspector={inspectorOpen}
		role="dialog"
		aria-modal="true"
		aria-label="Compose content"
		use:focusTrap
	>
		{@render children()}
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		width: 640px;
		max-width: 90vw;
		max-height: 85vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
		transition: width 0.2s ease;
	}

	.modal.with-inspector {
		width: 900px;
	}

	/* Focus mode */
	.modal.focus-mode {
		width: 100vw;
		max-width: 100vw;
		height: 100vh;
		max-height: 100vh;
		border-radius: 0;
	}

	/* Mobile responsive */
	@media (max-width: 640px) {
		.modal {
			width: 100vw;
			max-width: 100vw;
			height: 100vh;
			max-height: 100vh;
			border-radius: 0;
		}
	}
</style>
