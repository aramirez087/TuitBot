<script lang="ts">
	import { Image, Sparkles, MessageSquare, List, Search } from 'lucide-svelte';
	import { formatCombo } from '$lib/utils/shortcuts';

	let {
		mode = 'tweet',
		charCount = 0,
		charMax = 280,
		blockIndex = 0,
		blockCount = 1,
		canAttachMedia = true,
		uploading = false,
		onattachmedia,
		onaiassist,
		onswitchmode,
		onopenpalette
	}: {
		mode: 'tweet' | 'thread';
		charCount: number;
		charMax: number;
		blockIndex: number;
		blockCount: number;
		canAttachMedia: boolean;
		uploading: boolean;
		onattachmedia: () => void;
		onaiassist: () => void;
		onswitchmode: () => void;
		onopenpalette: () => void;
	} = $props();

	const ModeIcon = $derived(mode === 'tweet' ? MessageSquare : List);
	const modeLabel = $derived(mode === 'tweet' ? 'Tweet' : 'Thread');
	const switchLabel = $derived(mode === 'tweet' ? 'Switch to thread' : 'Switch to tweet');
	const showCharCount = $derived(mode === 'tweet' || charCount > 200);
	const charWarning = $derived(charCount > 260 && charCount <= charMax);
	const charOver = $derived(charCount > charMax);
</script>

<div class="composer-toolbar" role="toolbar" aria-label="Composer actions">
	<div class="toolbar-left">
		<button
			class="toolbar-btn"
			onclick={onattachmedia}
			disabled={!canAttachMedia || uploading}
			title="Attach media"
			aria-label={uploading ? 'Uploading media' : 'Attach media'}
		>
			<Image size={15} />
		</button>

		<button
			class="toolbar-btn ai-btn"
			onclick={onaiassist}
			title="AI improve ({formatCombo('cmd+shift+j')})"
			aria-label="AI improve selection or post"
		>
			<Sparkles size={15} />
		</button>

		<span class="toolbar-divider" aria-hidden="true"></span>

		<button
			class="mode-badge"
			onclick={onswitchmode}
			title={switchLabel}
			aria-label={switchLabel}
		>
			<ModeIcon size={13} />
			<span class="mode-label">{modeLabel}</span>
		</button>
	</div>

	<div class="toolbar-right">
		{#if showCharCount}
			<span
				class="char-badge"
				class:warning={charWarning}
				class:over={charOver}
				aria-label="Character count: {charCount} of {charMax}"
			>
				{charCount}/{charMax}
			</span>
		{/if}

		{#if mode === 'thread'}
			<span class="thread-position" aria-label="Post {blockIndex + 1} of {blockCount}">
				#{blockIndex + 1}<span class="thread-total">/{blockCount}</span>
			</span>
		{/if}

		<button
			class="palette-trigger"
			onclick={onopenpalette}
			aria-label="Command palette"
		>
			<Search size={12} />
			<kbd>{formatCombo('cmd+k')}</kbd>
			<span class="palette-label">All shortcuts</span>
		</button>
	</div>
</div>

<style>
	.composer-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 6px 16px;
		border-top: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
		background: color-mix(in srgb, var(--color-accent) 2%, var(--color-surface));
		flex-shrink: 0;
		position: sticky;
		bottom: 0;
		z-index: 2;
	}

	.toolbar-left {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.toolbar-right {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.toolbar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.12s ease;
		padding: 0;
	}

	.toolbar-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
	}

	.toolbar-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.toolbar-btn.ai-btn:hover:not(:disabled) {
		color: var(--color-warning, #d29922);
		background: color-mix(in srgb, var(--color-warning, #d29922) 8%, transparent);
	}

	.toolbar-divider {
		width: 1px;
		height: 16px;
		background: color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
		margin: 0 4px;
	}

	.mode-badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border: 1px solid color-mix(in srgb, var(--color-border-subtle) 60%, transparent);
		border-radius: 12px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.mode-badge:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
	}

	.mode-label {
		letter-spacing: 0.01em;
	}

	.char-badge {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		letter-spacing: -0.02em;
		white-space: nowrap;
	}

	.char-badge.warning {
		color: var(--color-warning);
	}

	.char-badge.over {
		color: var(--color-danger);
		font-weight: 600;
	}

	.thread-position {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		letter-spacing: -0.02em;
	}

	.thread-total {
		opacity: 0.5;
	}

	.palette-trigger {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 2px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		cursor: pointer;
		transition: color 0.12s ease;
	}

	.palette-trigger:hover {
		color: var(--color-accent);
	}

	.palette-trigger kbd {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 500;
		border: 1px solid color-mix(in srgb, var(--color-accent) 12%, transparent);
		line-height: 1.5;
	}

	@media (max-width: 480px) {
		.palette-label {
			display: none;
		}

		.palette-trigger kbd {
			font-size: 9px;
		}

		.mode-label {
			display: none;
		}

		.toolbar-right {
			gap: 6px;
		}
	}

	@media (pointer: coarse) {
		.toolbar-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.mode-badge {
			min-height: 36px;
			padding: 4px 12px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.toolbar-btn,
		.mode-badge,
		.palette-trigger {
			transition: none;
		}
	}
</style>
