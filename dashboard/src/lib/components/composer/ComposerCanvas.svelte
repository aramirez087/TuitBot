<script lang="ts">
	import { Send } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	let {
		canSubmit,
		submitting,
		selectedTime,
		submitError,
		onsubmit,
		children
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		submitError: string | null;
		onsubmit: () => void;
		children: Snippet;
	} = $props();
</script>

<div class="canvas">
	<div class="canvas-main">
		{@render children()}

		{#if submitError}
			<div class="error-msg" role="alert">{submitError}</div>
		{/if}
	</div>

	<div class="submit-anchor">
		<button
			class="submit-pill"
			onclick={onsubmit}
			disabled={!canSubmit || submitting}
		>
			<Send size={14} />
			{submitting ? 'Submitting...' : selectedTime ? 'Schedule' : 'Post now'}
		</button>
	</div>
</div>

<style>
	.canvas {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		position: relative;
	}

	.canvas-main {
		padding: 4px 20px 20px;
		flex: 1;
	}

	.error-msg {
		margin-top: 12px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 12px;
	}

	.submit-anchor {
		position: sticky;
		bottom: 0;
		display: flex;
		justify-content: flex-end;
		padding: 0 20px 16px;
		pointer-events: none;
	}

	.submit-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 40px;
		padding: 0 24px;
		border: none;
		border-radius: 20px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		pointer-events: auto;
		transition: all 0.15s ease;
		box-shadow: 0 2px 12px rgba(0, 0, 0, 0.3);
	}

	.submit-pill:hover:not(:disabled) {
		background: var(--color-accent-hover);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
	}

	.submit-pill:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	@media (pointer: coarse) {
		.submit-pill {
			min-height: 44px;
		}
	}

	@media (max-width: 640px) {
		.canvas-main {
			padding: 4px 16px 16px;
		}

		.submit-anchor {
			padding: 0 16px 16px;
			padding-bottom: calc(16px + env(safe-area-inset-bottom, 0px));
		}

		.submit-pill {
			width: 100%;
			justify-content: center;
		}
	}
</style>
