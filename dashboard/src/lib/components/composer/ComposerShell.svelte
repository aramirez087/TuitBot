<script lang="ts">
	import { focusTrap } from '$lib/actions/focusTrap';
	import type { Snippet } from 'svelte';

	let {
		open,
		focusMode,
		showRecovery,
		onclose,
		onrecover,
		ondismissrecovery,
		children
	}: {
		open: boolean;
		focusMode: boolean;
		showRecovery: boolean;
		onclose: () => void;
		onrecover: () => void;
		ondismissrecovery: () => void;
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
		role="dialog"
		aria-modal="true"
		aria-label="Compose content"
		use:focusTrap
	>
		{#if showRecovery}
			<div class="recovery-banner" role="alert">
				<span>Unsaved draft found. Recover it?</span>
				<div class="recovery-actions">
					<button class="recovery-btn" onclick={onrecover}>Recover</button>
					<button class="recovery-dismiss" onclick={ondismissrecovery}>Discard</button>
				</div>
			</div>
		{/if}

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
	}

	.recovery-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 20px;
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		border-bottom: 1px solid color-mix(in srgb, var(--color-warning) 25%, transparent);
		font-size: 13px;
		color: var(--color-warning);
		border-radius: 12px 12px 0 0;
		flex-shrink: 0;
	}

	.recovery-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}

	.recovery-btn {
		padding: 4px 12px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: var(--color-warning);
		color: #000;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
	}

	.recovery-dismiss {
		padding: 4px 12px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: transparent;
		color: var(--color-warning);
		font-size: 12px;
		cursor: pointer;
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

		.recovery-banner {
			border-radius: 0;
		}
	}
</style>
