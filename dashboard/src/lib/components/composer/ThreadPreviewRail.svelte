<script lang="ts">
	import ComposerPreviewSurface from './ComposerPreviewSurface.svelte';

	let {
		mode,
		tweetText = '',
		tweetMediaPaths = [],
		tweetLocalPreviews,
		blocks = [],
		handle = '@you',
		avatarUrl = null,
		previewMode = false,
		onclosepreview,
	}: {
		mode: 'tweet' | 'thread';
		tweetText?: string;
		tweetMediaPaths?: string[];
		tweetLocalPreviews?: Map<string, string>;
		blocks?: Array<{ id: string; text: string; media_paths: string[] }>;
		handle?: string;
		avatarUrl?: string | null;
		previewMode?: boolean;
		onclosepreview?: () => void;
	} = $props();
</script>

{#if previewMode}
	<ComposerPreviewSurface
		{mode}
		{tweetText}
		{blocks}
		tweetMediaPaths={tweetMediaPaths ?? []}
		tweetLocalPreviews={tweetLocalPreviews ?? new Map()}
		{handle}
		{avatarUrl}
		onclose={() => { onclosepreview?.(); }}
	/>
{/if}
