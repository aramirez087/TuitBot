<script lang="ts">
	interface Props {
		selectionCount: number;
		maxSelections: number;
		mode: 'tweet' | 'thread';
		generating: boolean;
		confirmReplace: boolean;
		showUndo: boolean;
		onundo?: () => void;
		onGenerate: () => void;
		onCancelReplace: () => void;
	}

	const {
		selectionCount,
		maxSelections,
		mode,
		generating,
		confirmReplace,
		showUndo,
		onundo,
		onGenerate,
		onCancelReplace,
	}: Props = $props();
</script>

<div class="vault-footer">
	<span class="vault-selection-count">
		{selectionCount} of {maxSelections} selected
	</span>

	{#if confirmReplace}
		<div class="vault-replace-banner" role="alert">
			<span>This will replace your current content.</span>
			<div class="vault-replace-actions">
				<button class="vault-replace-confirm" onclick={onGenerate}>Replace</button>
				<button class="vault-replace-cancel" onclick={onCancelReplace}>Cancel</button>
			</div>
		</div>
	{:else}
		<button
			class="vault-generate-btn"
			onclick={onGenerate}
			disabled={selectionCount === 0 || generating}
		>
			{generating
				? 'Generating...'
				: mode === 'thread'
					? 'Generate thread from vault'
					: 'Generate tweet from vault'}
		</button>
	{/if}

	{#if showUndo && onundo}
		<button class="vault-undo-btn" onclick={onundo}>Undo replacement</button>
	{/if}
</div>

<style>
	.vault-footer {
		margin-top: 6px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.vault-selection-count {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.vault-generate-btn {
		padding: 6px 14px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.vault-generate-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.vault-generate-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.vault-replace-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		font-size: 12px;
		color: var(--color-warning);
	}

	.vault-replace-actions {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
	}

	.vault-replace-confirm {
		padding: 4px 10px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: var(--color-warning);
		color: #000;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
	}

	.vault-replace-cancel {
		padding: 4px 10px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: transparent;
		color: var(--color-warning);
		font-size: 11px;
		cursor: pointer;
	}

	.vault-undo-btn {
		padding: 4px 10px;
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		background: transparent;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.vault-undo-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	@media (pointer: coarse) {
		.vault-generate-btn {
			min-height: 44px;
			padding: 10px 14px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.vault-generate-btn,
		.vault-undo-btn {
			transition: none;
		}
	}
</style>
