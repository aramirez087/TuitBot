<script lang="ts">
	import { api } from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import { X, Image, Film } from 'lucide-svelte';

	export interface AttachedMedia {
		path: string;
		file?: File;
		previewUrl: string;
		mediaType: string;
	}

	let {
		text,
		onchange,
		attachedMedia,
		onmediachange,
		onerror,
		avatarUrl = null
	}: {
		text: string;
		onchange: (text: string) => void;
		attachedMedia: AttachedMedia[];
		onmediachange: (media: AttachedMedia[]) => void;
		onerror: (msg: string) => void;
		avatarUrl?: string | null;
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
	const mediaCount = $derived(attachedMedia.length);

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

<div class="tweet-compose" class:has-avatar={!!avatarUrl}>
	{#if avatarUrl}
		<img src={avatarUrl} alt="" class="compose-avatar" />
	{/if}
	<textarea
		class="compose-input"
		class:over-limit={tweetOverLimit}
		placeholder="What's on your mind?"
		value={text}
		oninput={(e) => onchange(e.currentTarget.value)}
		rows={4}
		aria-label="Tweet content"
	></textarea>
	{#if tweetChars > 240 || tweetOverLimit}
		<div
			class="char-counter"
			class:over-limit={tweetOverLimit}
			aria-live="polite"
			aria-label="Character count"
		>
			{tweetChars}/{TWEET_MAX}
		</div>
	{/if}
</div>

{#if mediaCount > 0}
	<div
		class="media-preview-grid"
		class:single={mediaCount === 1}
		class:double={mediaCount === 2}
		class:triple={mediaCount === 3}
		class:quad={mediaCount >= 4}
	>
		{#each attachedMedia as media, i}
			<div class="media-thumb">
				{#if media.mediaType === 'video/mp4'}
					<video src={media.previewUrl} class="thumb-img" muted></video>
					<span class="media-badge"><Film size={12} /> Video</span>
				{:else}
					<img src={media.previewUrl} alt="Attached media" class="thumb-img" />
					{#if media.mediaType === 'image/gif'}
						<span class="media-badge">GIF</span>
					{/if}
				{/if}
				<button class="remove-media-btn" onclick={() => removeMedia(i)} aria-label="Remove media">
					<X size={14} />
				</button>
			</div>
		{/each}
	</div>
{/if}

{#if canAttachMore}
	<div class="media-attach-section">
		<button
			class="attach-icon-btn"
			onclick={() => fileInput?.click()}
			disabled={uploading}
			title="Attach media (JPEG, PNG, WebP, GIF, MP4)"
			aria-label={uploading ? 'Uploading media' : 'Attach media'}
		>
			<Image size={16} />
		</button>
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
		position: relative;
	}

	.tweet-compose.has-avatar {
		display: flex;
		gap: 12px;
		align-items: flex-start;
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
		font-size: 17px;
		font-family: var(--font-sans);
		line-height: 1.55;
		resize: vertical;
		box-sizing: border-box;
	}

	.compose-input:focus {
		outline: none;
	}

	.compose-input.over-limit {
		box-shadow: inset 2px 0 0 var(--color-danger);
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
		display: grid;
		gap: 2px;
		border-radius: 12px;
		overflow: hidden;
		margin-top: 12px;
		border: 1px solid var(--color-border-subtle);
	}

	.media-preview-grid.single { grid-template-columns: 1fr; }
	.media-preview-grid.double { grid-template-columns: 1fr 1fr; }
	.media-preview-grid.triple {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}
	.media-preview-grid.triple .media-thumb:first-child { grid-row: 1 / 3; }
	.media-preview-grid.quad {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}

	.media-thumb {
		position: relative;
		overflow: hidden;
		min-height: 100px;
		background: var(--color-surface-active);
	}

	.media-preview-grid.single .media-thumb { aspect-ratio: 16 / 9; }
	.media-preview-grid.double .media-thumb { aspect-ratio: 1; }

	.thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.media-badge {
		position: absolute;
		bottom: 6px;
		left: 6px;
		display: flex;
		align-items: center;
		gap: 3px;
		font-size: 10px;
		font-weight: 600;
		padding: 2px 6px;
		border-radius: 4px;
		background: rgba(0, 0, 0, 0.7);
		color: #fff;
		backdrop-filter: blur(4px);
	}

	.remove-media-btn {
		position: absolute;
		top: 6px;
		right: 6px;
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 50%;
		background: rgba(0, 0, 0, 0.6);
		color: #fff;
		cursor: pointer;
		transition: background 0.15s ease;
		backdrop-filter: blur(4px);
	}

	.remove-media-btn:hover {
		background: rgba(0, 0, 0, 0.85);
	}

	.media-attach-section {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 8px;
	}

	.attach-icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border: 1px dashed transparent;
		border-radius: 8px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.attach-icon-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
		border-color: color-mix(in srgb, var(--color-accent) 25%, transparent);
		color: var(--color-accent);
	}

	.attach-icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.hidden-file-input {
		display: none;
	}

	@media (pointer: coarse) {
		.remove-media-btn {
			width: 32px;
			height: 32px;
		}

		.attach-icon-btn {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.remove-media-btn,
		.attach-icon-btn {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.compose-input {
			font-size: 16px;
		}
	}
</style>
