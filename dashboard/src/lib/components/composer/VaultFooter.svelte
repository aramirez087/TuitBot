<script lang="ts">
	interface Props {
		selectionCount: number;
		maxSelections: number;
		outputFormat: 'tweet' | 'thread';
		extracting?: boolean;
		generating: boolean;
		confirmReplace: boolean;
		showUndo: boolean;
		selectionMode?: boolean;
		onundo?: () => void;
		onGenerate: () => void;
		onCancelReplace: () => void;
		onformatchange: (format: 'tweet' | 'thread') => void;
	}

	const {
		selectionCount,
		maxSelections,
		outputFormat,
		extracting = false,
		generating,
		confirmReplace,
		showUndo,
		selectionMode = false,
		onundo,
		onGenerate,
		onCancelReplace,
		onformatchange,
	}: Props = $props();

	function buttonLabel(): string {
		if (generating) return 'Generating...';
		if (extracting) return 'Extracting...';
		if (selectionMode) return 'Generate from selection';
		return 'Extract key points';
	}
</script>

<div class="vault-footer">
	<div class="vault-footer-row">
		<span class="vault-selection-count">
			{selectionCount} of {maxSelections} selected
		</span>
		<div class="vault-format-toggle" role="radiogroup" aria-label="Output format">
			<button
				class="vault-format-opt"
				class:active={outputFormat === 'tweet'}
				role="radio"
				aria-checked={outputFormat === 'tweet'}
				onclick={() => onformatchange('tweet')}
			>Tweet</button>
			<button
				class="vault-format-opt"
				class:active={outputFormat === 'thread'}
				role="radio"
				aria-checked={outputFormat === 'thread'}
				onclick={() => onformatchange('thread')}
			>Thread</button>
		</div>
	</div>

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
			disabled={selectionCount === 0 || extracting || generating}
		>
			{buttonLabel()}
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

	.vault-footer-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.vault-selection-count {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.vault-format-toggle {
		display: flex;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		overflow: hidden;
	}

	.vault-format-opt {
		padding: 3px 10px;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.vault-format-opt:first-child {
		border-right: 1px solid var(--color-border);
	}

	.vault-format-opt.active {
		background: var(--color-accent);
		color: #fff;
	}

	.vault-format-opt:hover:not(.active) {
		background: var(--color-surface-hover);
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
