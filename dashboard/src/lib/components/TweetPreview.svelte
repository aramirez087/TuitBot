<script lang="ts">
	import MediaCropPreview from "./composer/MediaCropPreview.svelte";
	import TweetActionBar from "./composer/TweetActionBar.svelte";

	let {
		text,
		mediaPaths = [],
		localPreviews,
		index,
		total,
		handle = "@you",
		displayName = "",
		avatarUrl = null,
		deviceMode = 'desktop' as 'mobile' | 'desktop',
	}: {
		text: string;
		mediaPaths: string[];
		localPreviews?: Map<string, string>;
		index: number;
		total: number;
		handle?: string;
		displayName?: string;
		avatarUrl?: string | null;
		deviceMode?: 'mobile' | 'desktop';
	} = $props();

	const displayText = $derived(text.trim() || "");
	const hasMedia = $derived(mediaPaths.length > 0);
	const showConnector = $derived(index < total - 1);
	const compact = $derived(deviceMode === 'mobile');
	const effectiveDisplayName = $derived(displayName || handle.replace('@', ''));
	const timeAgo = $derived('1m');
</script>

<article class="tweet-preview" aria-label="Tweet {index + 1} of {total}">
	<div class="preview-gutter">
		{#if avatarUrl}
			<img class="avatar-img" src={avatarUrl} alt="" aria-hidden="true" />
		{:else}
			<div class="avatar-placeholder" aria-hidden="true"></div>
		{/if}
		{#if showConnector}
			<div class="thread-connector" aria-hidden="true"></div>
		{/if}
	</div>
	<div class="preview-body">
		<div class="preview-header">
			<div class="preview-author">
				<span class="preview-display-name">{effectiveDisplayName}</span>
				<span class="preview-handle-subtle">{handle}</span>
				<span class="preview-dot">&middot;</span>
				<span class="preview-timestamp">{timeAgo}</span>
			</div>
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
		<TweetActionBar {compact} />
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
		width: 40px;
	}

	.avatar-placeholder,
	.avatar-img {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.avatar-placeholder {
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
	}

	.avatar-img {
		object-fit: cover;
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

	.preview-author {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
		overflow: hidden;
	}

	.preview-display-name {
		font-size: 15px;
		font-weight: 700;
		color: var(--color-text);
	}

	.preview-handle-subtle {
		font-size: 15px;
		color: var(--color-text-subtle);
		font-weight: 400;
	}

	.preview-dot {
		font-size: 15px;
		color: var(--color-text-subtle);
		padding: 0 4px;
	}

	.preview-timestamp {
		font-size: 15px;
		color: var(--color-text-subtle);
	}

	.preview-index {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
	}

	.preview-text {
		font-size: 15px;
		line-height: 20px;
		font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
		color: var(--color-text);
		white-space: pre-wrap;
		word-break: break-word;
	}

	.preview-text.placeholder {
		color: var(--color-text-subtle);
		font-style: italic;
	}
</style>
