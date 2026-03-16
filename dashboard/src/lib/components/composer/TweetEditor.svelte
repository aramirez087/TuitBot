<script lang="ts">
	import { api } from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import CharRing from './CharRing.svelte';
	import TweetMediaPreview from './TweetMediaPreview.svelte';

	export interface AttachedMedia {
		path: string;
		file?: File;
		previewUrl: string;
		mediaType: string;
		altText?: string;
	}

	let {
		text,
		onchange,
		attachedMedia,
		onmediachange,
		onerror,
		avatarUrl = null,
		displayName = null,
		handle = null,
	}: {
		text: string;
		onchange: (text: string) => void;
		attachedMedia: AttachedMedia[];
		onmediachange: (media: AttachedMedia[]) => void;
		onerror: (msg: string) => void;
		avatarUrl?: string | null;
		displayName?: string | null;
		handle?: string | null;
	} = $props();

	let uploading = $state(false);
	let fileInput: HTMLInputElement | undefined = $state();

	const ACCEPTED_TYPES = 'image/jpeg,image/png,image/webp,image/gif,video/mp4';
	const MAX_IMAGES = 4;
	const MAX_IMAGE_SIZE = 5 * 1024 * 1024;
	const MAX_GIF_SIZE = 15 * 1024 * 1024;
	const MAX_VIDEO_SIZE = 512 * 1024 * 1024;
	const TWEET_MAX = 280;

	const tweetChars = $derived(tweetWeightedLen(text));
	const tweetOverLimit = $derived(tweetChars > TWEET_MAX);

	const hasGifOrVideo = $derived(
		attachedMedia.some((m) => m.mediaType === 'image/gif' || m.mediaType === 'video/mp4'),
	);
	const canAttachMore = $derived(!hasGifOrVideo && attachedMedia.length < MAX_IMAGES);

	function getMaxSize(type: string): number {
		if (type === 'video/mp4') return MAX_VIDEO_SIZE;
		if (type === 'image/gif') return MAX_GIF_SIZE;
		return MAX_IMAGE_SIZE;
	}

	async function processFiles(files: FileList | File[]) {
		for (const file of files) {
			const accepted = ACCEPTED_TYPES.split(',');
			if (!accepted.includes(file.type)) {
				onerror(`Unsupported file type: ${file.type || 'unknown'}`);
				break;
			}
			if (!canAttachMore && !hasGifOrVideo) {
				onerror(`Maximum ${MAX_IMAGES} images allowed per tweet.`);
				break;
			}
			const isGifOrVideo = file.type === 'image/gif' || file.type === 'video/mp4';
			if (isGifOrVideo && attachedMedia.length > 0) {
				onerror('GIF/video cannot be combined with other media.');
				break;
			}
			if (!isGifOrVideo && hasGifOrVideo) {
				onerror('Cannot add images when GIF/video is attached.');
				break;
			}
			const maxSize = getMaxSize(file.type);
			if (file.size > maxSize) {
				onerror(`File "${file.name}" exceeds maximum size of ${Math.round(maxSize / 1024 / 1024)}MB.`);
				break;
			}
			uploading = true;
			try {
				const result = await api.media.upload(file);
				onmediachange([
					...attachedMedia,
					{
						path: result.path,
						file,
						previewUrl: api.media.fileUrl(result.path),
						mediaType: result.media_type,
					},
				]);
			} catch (err) {
				onerror(err instanceof Error ? err.message : 'Failed to upload media');
				break;
			} finally {
				uploading = false;
			}
		}
	}

	async function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const files = input.files;
		if (!files || files.length === 0) return;
		await processFiles(files);
		input.value = '';
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		const files = e.dataTransfer?.files;
		if (files && files.length > 0) processFiles(files);
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
	}

	async function handlePaste(e: ClipboardEvent) {
		const files = e.clipboardData?.files;
		if (files && files.length > 0) {
			e.preventDefault();
			processFiles(files);
			return;
		}
		if (e.clipboardData?.getData('text/plain')) return;
		// WKWebView fallback: handled by ComposeWorkspace Cmd+V in Tauri
	}

	function removeMedia(index: number) {
		onmediachange(attachedMedia.filter((_, i) => i !== index));
	}

	export function triggerFileSelect() {
		fileInput?.click();
	}
</script>

<div class="tweet-compose" class:has-avatar={!!avatarUrl}>
	{#if avatarUrl}
		<img src={avatarUrl} alt="" class="compose-avatar" />
	{/if}
	<div class="compose-main">
		{#if displayName || handle}
			<div class="compose-identity">
				{#if displayName}<span class="compose-display-name">{displayName}</span>{/if}
				{#if handle}<span class="compose-handle">@{handle}</span>{/if}
			</div>
		{/if}
		<textarea
			class="compose-input"
			class:over-limit={tweetOverLimit}
			placeholder="What's on your mind?"
			value={text}
			oninput={(e) => onchange(e.currentTarget.value)}
			ondrop={handleDrop}
			ondragover={handleDragOver}
			onpaste={handlePaste}
			rows={4}
			aria-label="Tweet content"
		></textarea>
		<div class="char-ring-row">
			<CharRing current={tweetChars} max={TWEET_MAX} />
		</div>
	</div>
</div>

{#if attachedMedia.length > 0}
	<TweetMediaPreview
		{attachedMedia}
		onRemove={removeMedia}
		onMediaChange={onmediachange}
	/>
{/if}

<!-- File input kept for triggerFileSelect() called from toolbar -->
<input
	bind:this={fileInput}
	type="file"
	accept={ACCEPTED_TYPES}
	multiple
	class="hidden-file-input"
	onchange={handleFileSelect}
/>

<style>
	.tweet-compose {
		position: relative;
	}

	.tweet-compose.has-avatar {
		display: flex;
		gap: 12px;
		align-items: flex-start;
	}

	.compose-main {
		flex: 1;
		min-width: 0;
	}

	.compose-identity {
		display: flex;
		align-items: baseline;
		gap: 6px;
		padding-top: 2px;
		margin-bottom: 1px;
	}

	.compose-display-name {
		font-size: 14px;
		font-weight: 700;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 200px;
	}

	.compose-handle {
		font-size: 13px;
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.compose-avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
		margin-top: 16px;
	}

	.compose-input {
		width: 100%;
		padding: 16px 0;
		border: none;
		border-radius: 0;
		background: transparent;
		color: var(--color-text);
		font-size: 15px;
		font-family: var(--font-sans);
		line-height: 1.5;
		letter-spacing: -0.01em;
		caret-color: var(--color-accent);
		text-rendering: optimizeLegibility;
		-webkit-font-smoothing: antialiased;
		resize: vertical;
		box-sizing: border-box;
	}

	.compose-input::placeholder {
		color: var(--color-text-subtle);
		opacity: 0.35;
		font-style: italic;
	}

	.compose-input:focus {
		outline: none;
	}

	.char-ring-row {
		display: flex;
		justify-content: flex-end;
		margin-top: 4px;
	}

	.hidden-file-input {
		display: none;
	}

	@media (max-width: 640px) {
		.compose-input {
			font-size: 16px;
		}
	}
</style>
