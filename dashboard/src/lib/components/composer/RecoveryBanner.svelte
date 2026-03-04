<script lang="ts">
	let {
		onrecover,
		ondismiss
	}: {
		onrecover: () => void;
		ondismiss: () => void;
	} = $props();
</script>

<div class="recovery-banner" role="alert">
	<span>Unsaved draft found. Recover it?</span>
	<div class="recovery-actions">
		<button class="recovery-btn" onclick={onrecover}>Recover</button>
		<button class="recovery-dismiss" onclick={ondismiss}>Discard</button>
	</div>
</div>

<style>
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

	@media (pointer: coarse) {
		.recovery-btn,
		.recovery-dismiss {
			min-height: 44px;
			display: flex;
			align-items: center;
		}
	}
</style>
