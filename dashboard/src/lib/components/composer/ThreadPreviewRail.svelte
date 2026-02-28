<script lang="ts">
	import TweetPreview from '../TweetPreview.svelte';

	let {
		mode,
		tweetText = '',
		tweetMediaPaths = [],
		tweetLocalPreviews,
		blocks = [],
		handle = '@you'
	}: {
		mode: 'tweet' | 'thread';
		tweetText?: string;
		tweetMediaPaths?: string[];
		tweetLocalPreviews?: Map<string, string>;
		blocks?: Array<{ id: string; text: string; media_paths: string[] }>;
		handle?: string;
	} = $props();

	const hasTweetContent = $derived(
		(tweetText?.trim().length ?? 0) > 0 || (tweetMediaPaths?.length ?? 0) > 0
	);

	const visibleBlocks = $derived(
		blocks.filter((b) => b.text.trim().length > 0)
	);
</script>

<div class="preview-rail">
	<div class="preview-header-label">Preview</div>
	<div class="preview-scroll">
		{#if mode === 'tweet'}
			{#if hasTweetContent}
				<TweetPreview
					text={tweetText ?? ''}
					mediaPaths={tweetMediaPaths ?? []}
					localPreviews={tweetLocalPreviews}
					index={0}
					total={1}
					{handle}
				/>
			{:else}
				<div class="preview-empty">Type to see preview...</div>
			{/if}
		{:else}
			{#if visibleBlocks.length > 0}
				{#each visibleBlocks as block, i (block.id)}
					<TweetPreview
						text={block.text}
						mediaPaths={block.media_paths}
						index={i}
						total={visibleBlocks.length}
						{handle}
					/>
				{/each}
			{:else}
				<div class="preview-empty">Start typing to see preview...</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	.preview-rail {
		min-width: 0;
	}

	.preview-header-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		margin-bottom: 8px;
	}

	.preview-scroll {
		max-height: 400px;
		overflow-y: auto;
	}

	.preview-empty {
		padding: 24px 0;
		text-align: center;
		font-size: 13px;
		color: var(--color-text-subtle);
		font-style: italic;
	}
</style>
