<script lang="ts">
	import { ArrowLeft } from 'lucide-svelte';

	interface Highlight {
		text: string;
		enabled: boolean;
	}

	let {
		highlights,
		outputFormat = 'tweet',
		generating = false,
		ongenerate,
		onback,
		onformatchange,
	}: {
		highlights: Highlight[];
		outputFormat: 'tweet' | 'thread';
		generating?: boolean;
		ongenerate: (enabledHighlights: string[]) => void;
		onback: () => void;
		onformatchange: (format: 'tweet' | 'thread') => void;
	} = $props();

	function toggleHighlight(index: number) {
		highlights[index].enabled = !highlights[index].enabled;
	}

	const enabledCount = $derived(highlights.filter((h) => h.enabled).length);

	function handleGenerate() {
		const enabled = highlights.filter((h) => h.enabled).map((h) => h.text);
		if (enabled.length > 0) {
			ongenerate(enabled);
		}
	}
</script>

<div class="highlights-panel">
	<div class="highlights-header">
		<button class="highlights-back" onclick={onback} aria-label="Back to notes">
			<ArrowLeft size={14} />
		</button>
		<span class="highlights-label">Key Highlights</span>
	</div>

	<div class="highlights-list" role="group" aria-label="Select highlights to include">
		{#each highlights as highlight, i}
			<label class="highlight-item" class:disabled={!highlight.enabled}>
				<input
					type="checkbox"
					checked={highlight.enabled}
					onchange={() => toggleHighlight(i)}
					class="highlight-check"
				/>
				<span class="highlight-text">{highlight.text}</span>
			</label>
		{/each}
	</div>

	<div class="highlights-footer">
		<div class="highlights-footer-row">
			<span class="highlights-count">
				{enabledCount} of {highlights.length} selected
			</span>
			<div class="highlights-format-toggle" role="radiogroup" aria-label="Output format">
				<button
					class="highlights-format-opt"
					class:active={outputFormat === 'tweet'}
					role="radio"
					aria-checked={outputFormat === 'tweet'}
					onclick={() => onformatchange('tweet')}
				>Tweet</button>
				<button
					class="highlights-format-opt"
					class:active={outputFormat === 'thread'}
					role="radio"
					aria-checked={outputFormat === 'thread'}
					onclick={() => onformatchange('thread')}
				>Thread</button>
			</div>
		</div>

		<button
			class="highlights-generate-btn"
			onclick={handleGenerate}
			disabled={enabledCount === 0 || generating}
		>
			{generating ? 'Generating...' : 'Find hooks'}
		</button>
	</div>
</div>

<style>
	.highlights-panel {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.highlights-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 10px;
	}

	.highlights-back {
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

	.highlights-back:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.highlights-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.highlights-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 200px;
		overflow-y: auto;
		overscroll-behavior: contain;
		margin-bottom: 8px;
	}

	.highlight-item {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 6px;
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.highlight-item:hover {
		border-color: var(--color-accent);
	}

	.highlight-item.disabled {
		opacity: 0.45;
	}

	.highlight-check {
		flex-shrink: 0;
		margin-top: 2px;
		accent-color: var(--color-accent);
	}

	.highlight-text {
		font-size: 13px;
		line-height: 1.45;
		color: var(--color-text);
	}

	.highlights-footer {
		margin-top: 2px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.highlights-footer-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.highlights-count {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.highlights-format-toggle {
		display: flex;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		overflow: hidden;
	}

	.highlights-format-opt {
		padding: 3px 10px;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.highlights-format-opt:first-child {
		border-right: 1px solid var(--color-border);
	}

	.highlights-format-opt.active {
		background: var(--color-accent);
		color: #fff;
	}

	.highlights-format-opt:hover:not(.active) {
		background: var(--color-surface-hover);
	}

	.highlights-generate-btn {
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

	.highlights-generate-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.highlights-generate-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	@media (pointer: coarse) {
		.highlights-back {
			min-width: 44px;
			min-height: 44px;
		}

		.highlights-generate-btn {
			min-height: 44px;
			padding: 10px 14px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.highlights-back,
		.highlight-item,
		.highlights-generate-btn {
			transition: none;
		}
	}
</style>
