<script lang="ts">
	import { Send } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	let {
		canSubmit,
		submitting,
		selectedTime,
		submitError,
		inspectorOpen = false,
		onsubmit,
		children,
		inspector
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		submitError: string | null;
		inspectorOpen?: boolean;
		onsubmit: () => void;
		children: Snippet;
		inspector?: Snippet;
	} = $props();
</script>

<div class="canvas" class:with-inspector={inspectorOpen && inspector}>
	<div class="canvas-main">
		{@render children()}

		{#if submitError}
			<div class="error-msg" role="alert">{submitError}</div>
		{/if}

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

	{#if inspectorOpen && inspector}
		<div class="canvas-inspector">
			{@render inspector()}
		</div>
	{/if}
</div>

<style>
	.canvas {
		display: flex;
		flex: 1;
		min-height: 0;
		position: relative;
	}

	.canvas.with-inspector {
		display: flex;
	}

	.canvas-main {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow-y: auto;
	}

	.canvas-main > :global(:first-child) {
		padding-top: 4px;
	}

	.canvas-main {
		padding: 0 20px 20px;
	}

	.canvas-inspector {
		width: 260px;
		flex-shrink: 0;
		border-left: 1px solid var(--color-border-subtle);
		overflow-y: auto;
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-base) 50%, var(--color-surface));
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
		padding: 12px 0 0;
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

	@media (max-width: 768px) {
		.canvas-inspector {
			display: none;
		}
	}

	@media (max-width: 640px) {
		.canvas-main {
			padding: 0 16px 16px;
		}

		.submit-anchor {
			padding-bottom: env(safe-area-inset-bottom, 0px);
		}

		.submit-pill {
			width: 100%;
			justify-content: center;
		}
	}
</style>
