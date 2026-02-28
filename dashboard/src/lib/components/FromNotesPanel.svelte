<script lang="ts">
	import { X } from 'lucide-svelte';

	let {
		mode,
		hasExistingContent = false,
		ongenerate,
		onclose,
		onundo,
		showUndo = false
	}: {
		mode: 'tweet' | 'thread';
		hasExistingContent?: boolean;
		ongenerate: (text: string) => Promise<void>;
		onclose: () => void;
		onundo?: () => void;
		showUndo?: boolean;
	} = $props();

	let notesText = $state('');
	let generating = $state(false);
	let error = $state<string | null>(null);
	let confirmReplace = $state(false);

	async function handleGenerate() {
		if (!notesText.trim() || generating) return;

		if (hasExistingContent && !confirmReplace) {
			confirmReplace = true;
			return;
		}

		generating = true;
		error = null;
		confirmReplace = false;
		try {
			await ongenerate(notesText.trim());
			notesText = '';
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to generate from notes';
		} finally {
			generating = false;
		}
	}

	function cancelReplace() {
		confirmReplace = false;
	}
</script>

<div class="from-notes-section">
	<div class="notes-header">
		<span class="notes-label">From Notes</span>
		<button class="notes-close" onclick={onclose} aria-label="Close notes panel">
			<X size={12} />
		</button>
	</div>

	<div class="notes-textarea-wrapper" class:loading={generating}>
		<textarea
			class="notes-input"
			placeholder="Paste rough notes, ideas, or an outline..."
			bind:value={notesText}
			rows={4}
			disabled={generating}
			aria-label="Notes to transform into content"
		></textarea>
		{#if generating}
			<div class="loading-overlay" aria-label="Generating content">
				<div class="loading-shimmer"></div>
			</div>
		{/if}
	</div>

	{#if error}
		<div class="notes-error" role="alert">{error}</div>
	{/if}

	{#if confirmReplace}
		<div class="replace-banner" role="alert">
			<span>This will replace your current content.</span>
			<div class="replace-actions">
				<button class="replace-confirm-btn" onclick={handleGenerate}>Replace</button>
				<button class="replace-cancel-btn" onclick={cancelReplace}>Cancel</button>
			</div>
		</div>
	{:else}
		<button
			class="notes-generate-btn"
			onclick={handleGenerate}
			disabled={!notesText.trim() || generating}
		>
			{generating
				? 'Generating...'
				: mode === 'thread'
					? 'Generate thread from notes'
					: 'Generate tweet from notes'}
		</button>
	{/if}

	{#if showUndo && onundo}
		<button class="notes-undo-btn" onclick={onundo}>Undo replacement</button>
	{/if}
</div>

<style>
	.from-notes-section {
		margin-top: 12px;
		padding: 12px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: var(--color-base);
	}

	.notes-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.notes-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.notes-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.notes-close:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.notes-textarea-wrapper {
		position: relative;
	}

	.notes-input {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		line-height: 1.5;
		resize: vertical;
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.notes-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.notes-input::placeholder {
		color: var(--color-text-subtle);
	}

	.notes-input:disabled {
		opacity: 0.5;
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		border-radius: 6px;
		overflow: hidden;
		pointer-events: none;
	}

	.loading-shimmer {
		width: 100%;
		height: 100%;
		background: linear-gradient(
			90deg,
			transparent 25%,
			color-mix(in srgb, var(--color-accent) 8%, transparent) 50%,
			transparent 75%
		);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	@keyframes shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	.notes-error {
		margin-top: 6px;
		font-size: 12px;
		color: var(--color-danger);
	}

	.replace-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-top: 8px;
		padding: 8px 10px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		font-size: 12px;
		color: var(--color-warning);
	}

	.replace-actions {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
	}

	.replace-confirm-btn {
		padding: 4px 10px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: var(--color-warning);
		color: #000;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
	}

	.replace-cancel-btn {
		padding: 4px 10px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: transparent;
		color: var(--color-warning);
		font-size: 11px;
		cursor: pointer;
	}

	.notes-generate-btn {
		margin-top: 8px;
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

	.notes-generate-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.notes-generate-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.notes-undo-btn {
		margin-top: 6px;
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

	.notes-undo-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	@media (max-width: 640px) {
		.from-notes-section {
			padding: 10px;
		}

		.notes-input {
			font-size: 16px;
		}
	}

	@media (pointer: coarse) {
		.notes-close {
			min-width: 44px;
			min-height: 44px;
		}

		.notes-generate-btn {
			min-height: 44px;
			padding: 10px 14px;
		}
	}
</style>
