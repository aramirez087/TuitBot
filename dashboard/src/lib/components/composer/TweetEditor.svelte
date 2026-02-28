<script lang="ts">
	import { api } from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import { X, Image, Film } from 'lucide-svelte';

	export interface AttachedMedia {
		path: string;
		file: File;
		previewUrl: string;
		mediaType: string;
	}

	let {
		text,
		onchange,
		attachedMedia,
		onmediachange,
		onerror
	}: {
		text: string;
		onchange: (text: string) => void;
		attachedMedia: AttachedMedia[];
		onmediachange: (media: AttachedMedia[]) => void;
		onerror: (msg: string) => void;
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
		attachedMedia.some((m) => m.mediaType === 'image/gif' || m.mediaType === 'video/mp4')
	);
	const canAttachMore = $derived(!hasGifOrVideo && attachedMedia.length < MAX_IMAGES);

	function getMaxSize(type: string): number {
		if (type === 'video/mp4') return MAX_VIDEO_SIZE;
		if (type === 'image/gif') return MAX_GIF_SIZE;
		return MAX_IMAGE_SIZE;
	}

	async function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const files = input.files;
		if (!files || files.length === 0) return;

		for (const file of files) {
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
						previewUrl: URL.createObjectURL(file),
						mediaType: result.media_type
					}
				]);
			} catch (err) {
				onerror(err instanceof Error ? err.message : 'Failed to upload media');
				break;
			} finally {
				uploading = false;
			}
		}
		input.value = '';
	}

	function removeMedia(index: number) {
		const removed = attachedMedia[index];
		if (removed) URL.revokeObjectURL(removed.previewUrl);
		onmediachange(attachedMedia.filter((_, i) => i !== index));
	}

	export function triggerFileSelect() {
		fileInput?.click();
	}
</script>

<div class="tweet-compose">
	<textarea
		class="compose-input"
		class:over-limit={tweetOverLimit}
		placeholder="What's on your mind?"
		value={text}
		oninput={(e) => onchange(e.currentTarget.value)}
		rows={4}
		aria-label="Tweet content"
	></textarea>
	<div
		class="char-counter"
		class:over-limit={tweetOverLimit}
		aria-live="polite"
		aria-label="Character count"
	>
		{tweetChars}/{TWEET_MAX}
	</div>
</div>

{#if attachedMedia.length > 0}
	<div class="media-preview-grid">
		{#each attachedMedia as media, i}
			<div class="media-thumb">
				{#if media.mediaType === 'video/mp4'}
					<video src={media.previewUrl} class="thumb-img" muted></video>
					<span class="media-badge"><Film size={10} /> Video</span>
				{:else}
					<img src={media.previewUrl} alt="Attached media" class="thumb-img" />
					{#if media.mediaType === 'image/gif'}
						<span class="media-badge">GIF</span>
					{/if}
				{/if}
				<button class="remove-media-btn" onclick={() => removeMedia(i)} aria-label="Remove media">
					<X size={12} />
				</button>
			</div>
		{/each}
	</div>
{/if}

{#if canAttachMore}
	<div class="media-attach-section">
		<button class="attach-btn" onclick={() => fileInput?.click()} disabled={uploading}>
			<Image size={14} />
			{uploading ? 'Uploading...' : 'Attach media'}
		</button>
		<span class="attach-hint">
			JPEG, PNG, WebP, GIF, MP4 &middot; max 4 images or 1 GIF/video
		</span>
		<input
			bind:this={fileInput}
			type="file"
			accept={ACCEPTED_TYPES}
			multiple
			class="hidden-file-input"
			onchange={handleFileSelect}
		/>
	</div>
{/if}

<style>
	.tweet-compose {
		/* tweet editor wrapper */
	}

	.compose-input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-base);
		color: var(--color-text);
		font-size: 14px;
		font-family: var(--font-sans);
		line-height: 1.5;
		resize: vertical;
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.compose-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.compose-input.over-limit {
		border-color: var(--color-danger);
	}

	.char-counter {
		text-align: right;
		font-size: 11px;
		color: var(--color-text-subtle);
		margin-top: 4px;
		font-family: var(--font-mono);
	}

	.char-counter.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	.media-preview-grid {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-top: 12px;
	}

	.media-thumb {
		position: relative;
		width: 80px;
		height: 80px;
		border-radius: 8px;
		overflow: hidden;
		border: 1px solid var(--color-border);
	}

	.thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.media-badge {
		position: absolute;
		bottom: 4px;
		left: 4px;
		display: flex;
		align-items: center;
		gap: 3px;
		font-size: 9px;
		font-weight: 600;
		padding: 1px 5px;
		border-radius: 3px;
		background: rgba(0, 0, 0, 0.7);
		color: #fff;
	}

	.remove-media-btn {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 50%;
		background: rgba(0, 0, 0, 0.6);
		color: #fff;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.remove-media-btn:hover {
		background: rgba(0, 0, 0, 0.85);
	}

	.media-attach-section {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 12px;
	}

	.attach-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.attach-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.attach-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.attach-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.hidden-file-input {
		display: none;
	}

	@media (pointer: coarse) {
		.remove-media-btn {
			width: 32px;
			height: 32px;
		}
	}

	@media (max-width: 640px) {
		.media-attach-section {
			flex-direction: column;
			align-items: flex-start;
		}

		.compose-input {
			font-size: 16px;
		}
	}
</style>
