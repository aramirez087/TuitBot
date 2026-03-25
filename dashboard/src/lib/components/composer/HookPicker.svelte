<script lang="ts">
	import { ArrowLeft, RefreshCw } from 'lucide-svelte';
	import type { HookOption } from '$lib/api/types';
	import { getStyleLabel, getConfidenceBadge } from '$lib/utils/hookStyles';

	let {
		hooks,
		outputFormat = 'tweet',
		loading = false,
		generating = false,
		error = null,
		onselect,
		onregenerate,
		onback,
		onformatchange,
	}: {
		hooks: HookOption[];
		outputFormat: 'tweet' | 'thread';
		loading?: boolean;
		generating?: boolean;
		error?: string | null;
		onselect: (hook: HookOption, format: 'tweet' | 'thread') => void;
		onregenerate: () => void;
		onback: () => void;
		onformatchange: (format: 'tweet' | 'thread') => void;
	} = $props();

	let selectedIndex = $state<number | null>(null);

	function handleCardClick(index: number) {
		selectedIndex = index;
	}

	function handleCardKeydown(e: KeyboardEvent, index: number) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			selectedIndex = index;
		}
	}

	function handleConfirm() {
		if (selectedIndex !== null && hooks[selectedIndex]) {
			onselect(hooks[selectedIndex], outputFormat);
		}
	}

	function handleRegenerate() {
		selectedIndex = null;
		onregenerate();
	}
</script>

<div class="hook-picker">
	<div class="hook-header">
		<button class="hook-back" onclick={onback} aria-label="Back to highlights">
			<ArrowLeft size={14} />
		</button>
		<span class="hook-label">Choose a Hook</span>
	</div>

	{#if error}
		<div class="hook-error" role="alert">
			<span>{error}</span>
			<button class="hook-retry-btn" onclick={handleRegenerate}>Retry</button>
		</div>
	{/if}

	<div class="hook-card-list" role="listbox" aria-label="Hook options">
		{#if loading}
			{#each Array(5) as _}
				<div class="hook-card-shimmer" aria-hidden="true">
					<div class="shimmer-pill"></div>
					<div class="shimmer-line"></div>
					<div class="shimmer-line short"></div>
					<div class="shimmer-footer"></div>
				</div>
			{/each}
		{:else}
			{#each hooks as hook, i}
				<div
					class="hook-card"
					class:selected={selectedIndex === i}
					role="option"
					aria-selected={selectedIndex === i}
					tabindex="0"
					onclick={() => handleCardClick(i)}
					onkeydown={(e) => handleCardKeydown(e, i)}
				>
					<span class="hook-style-pill">{getStyleLabel(hook.style)}</span>
					<p class="hook-text">{hook.text}</p>
					<div class="hook-card-footer">
						<span class="hook-char-count">{hook.char_count} chars</span>
						<span class="hook-confidence {getConfidenceBadge(hook.confidence).cssClass}">{getConfidenceBadge(hook.confidence).label}</span>
					</div>
				</div>
			{/each}
		{/if}
	</div>

	<div class="hook-footer">
		<div class="hook-footer-row">
			<button
				class="hook-regen-btn"
				onclick={handleRegenerate}
				disabled={loading || generating}
			>
				<RefreshCw size={12} />
				Regenerate
			</button>
			<div class="hook-format-toggle" role="radiogroup" aria-label="Output format">
				<button
					class="hook-format-opt"
					class:active={outputFormat === 'tweet'}
					role="radio"
					aria-checked={outputFormat === 'tweet'}
					onclick={() => onformatchange('tweet')}
				>Tweet</button>
				<button
					class="hook-format-opt"
					class:active={outputFormat === 'thread'}
					role="radio"
					aria-checked={outputFormat === 'thread'}
					onclick={() => onformatchange('thread')}
				>Thread</button>
			</div>
		</div>

		<button
			class="hook-confirm-btn"
			onclick={handleConfirm}
			disabled={selectedIndex === null || loading || generating}
		>
			{#if generating}
				<span class="hook-confirm-spinner" aria-hidden="true"></span>
				Generating {outputFormat}…
			{:else}
				Use this hook
			{/if}
		</button>
	</div>
</div>

<style>
	.hook-picker {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.hook-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 10px;
	}

	.hook-back {
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

	.hook-back:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.hook-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.hook-error {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
		font-size: 12px;
		color: var(--color-danger);
		margin-bottom: 8px;
	}

	.hook-retry-btn {
		padding: 3px 10px;
		border: 1px solid var(--color-danger);
		border-radius: 4px;
		background: transparent;
		color: var(--color-danger);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		flex-shrink: 0;
	}

	.hook-retry-btn:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.hook-card-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 280px;
		overflow-y: auto;
		overscroll-behavior: contain;
		margin-bottom: 8px;
	}

	.hook-card {
		padding: 10px 12px;
		border-radius: 6px;
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.hook-card:hover {
		border-color: var(--color-accent);
	}

	.hook-card:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: -2px;
	}

	.hook-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface-active));
	}

	.hook-style-pill {
		display: inline-block;
		padding: 1px 8px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.03em;
		margin-bottom: 6px;
	}

	.hook-text {
		font-size: 13px;
		line-height: 1.45;
		color: var(--color-text);
		margin: 0 0 6px;
		word-break: break-word;
	}

	.hook-card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.hook-char-count {
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	.hook-confidence {
		font-size: 10px;
		font-weight: 500;
		padding: 1px 6px;
		border-radius: 3px;
	}

	.hook-confidence.confidence-high {
		background: color-mix(in srgb, var(--color-success, #22c55e) 12%, transparent);
		color: var(--color-success, #22c55e);
	}

	.hook-confidence.confidence-medium {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	/* Shimmer loading cards */
	.hook-card-shimmer {
		padding: 10px 12px;
		border-radius: 6px;
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.shimmer-pill {
		width: 60px;
		height: 16px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-accent) 8%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	.shimmer-line {
		width: 100%;
		height: 14px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-text-subtle) 8%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	.shimmer-line.short {
		width: 65%;
	}

	.shimmer-footer {
		width: 40%;
		height: 12px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-text-subtle) 6%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	@keyframes shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	.hook-footer {
		margin-top: 2px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.hook-footer-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.hook-regen-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.hook-regen-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.hook-regen-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.hook-format-toggle {
		display: flex;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		overflow: hidden;
	}

	.hook-format-opt {
		padding: 3px 10px;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.hook-format-opt:first-child {
		border-right: 1px solid var(--color-border);
	}

	.hook-format-opt.active {
		background: var(--color-accent);
		color: #fff;
	}

	.hook-format-opt:hover:not(.active) {
		background: var(--color-surface-hover);
	}

	.hook-confirm-btn {
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

	.hook-confirm-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.hook-confirm-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.hook-confirm-spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-top-color: #fff;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
		vertical-align: middle;
		margin-right: 4px;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	@media (pointer: coarse) {
		.hook-back {
			min-width: 44px;
			min-height: 44px;
		}

		.hook-confirm-btn {
			min-height: 44px;
			padding: 10px 14px;
		}

		.hook-card {
			padding: 12px 14px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.hook-back,
		.hook-card,
		.hook-confirm-btn,
		.hook-regen-btn {
			transition: none;
		}

		.shimmer-pill,
		.shimmer-line,
		.shimmer-footer,
		.hook-confirm-spinner {
			animation: none;
		}
	}
</style>
