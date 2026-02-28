<script lang="ts">
	import MediaCropPreview from './composer/MediaCropPreview.svelte';

	let {
		text,
		mediaPaths = [],
		localPreviews,
		index,
		total,
		handle = '@you'
	}: {
		text: string;
		mediaPaths: string[];
		localPreviews?: Map<string, string>;
		index: number;
		total: number;
		handle?: string;
	} = $props();

	const displayText = $derived(text.trim() || '');
	const hasMedia = $derived(mediaPaths.length > 0);
	const showConnector = $derived(index < total - 1);
</script>

<article class="tweet-preview" aria-label="Tweet {index + 1} of {total}">
	<div class="preview-gutter">
		<div class="avatar-placeholder" aria-hidden="true"></div>
		{#if showConnector}
			<div class="thread-connector" aria-hidden="true"></div>
		{/if}
	</div>
	<div class="preview-body">
		<div class="preview-header">
			<span class="preview-handle">{handle}</span>
			<span class="preview-index">{index + 1}/{total}</span>
		</div>
		{#if displayText}
			<div class="preview-text">{displayText}</div>
		{:else}
			<div class="preview-text placeholder">Empty tweet</div>
		{/if}
		{#if hasMedia}
			<MediaCropPreview {mediaPaths} {localPreviews} />
		{/if}
	</div>
</article>

<style>
	.tweet-preview {
		display: flex;
		gap: 10px;
		padding: 12px 0;
	}

	.preview-gutter {
		display: flex;
		flex-direction: column;
		align-items: center;
		flex-shrink: 0;
		width: 36px;
	}

	.avatar-placeholder {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.thread-connector {
		width: 2px;
		flex: 1;
		min-height: 12px;
		background: var(--color-border-subtle);
		margin-top: 4px;
	}

	.preview-body {
		flex: 1;
		min-width: 0;
	}

	.preview-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 2px;
	}

	.preview-handle {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
	}

	.preview-index {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
	}

	.preview-text {
		font-size: 13px;
		line-height: 1.5;
		color: var(--color-text);
		white-space: pre-wrap;
		word-break: break-word;
	}

	.preview-text.placeholder {
		color: var(--color-text-subtle);
		font-style: italic;
	}

	/* Mobile responsive */
	@media (max-width: 640px) {
		.tweet-preview {
			gap: 8px;
			padding: 10px 0;
		}

		.preview-gutter {
			width: 28px;
		}

		.avatar-placeholder {
			width: 28px;
			height: 28px;
		}

		.preview-text {
			font-size: 14px;
		}
	}
</style>
