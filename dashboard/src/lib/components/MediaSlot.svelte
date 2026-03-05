<script lang="ts">
	import { api } from '$lib/api';
	import { Image, X, Film } from 'lucide-svelte';
	import MediaAltBadge from './composer/MediaAltBadge.svelte';

	let {
		mediaPaths = [],
		onmediachange,
		maxMedia = 4,
		disabled = false,
		altTexts = {},
		onaltchange
	}: {
		mediaPaths: string[];
		onmediachange: (paths: string[]) => void;
		maxMedia?: number;
		disabled?: boolean;
		altTexts?: Record<string, string>;
		onaltchange?: (path: string, altText: string) => void;
	} = $props();

	const ACCEPTED_TYPES = 'image/jpeg,image/png,image/webp,image/gif,video/mp4';
	const MAX_IMAGE_SIZE = 5 * 1024 * 1024;
	const MAX_GIF_SIZE = 15 * 1024 * 1024;
	const MAX_VIDEO_SIZE = 512 * 1024 * 1024;

	let uploading = $state(false);
	let error = $state<string | null>(null);
	let fileInput: HTMLInputElement | undefined = $state();
	let dragOver = $state(false);

	let localPreviews = $state<Map<string, { url: string; type: string }>>(new Map());
	const mediaCount = $derived(mediaPaths.length);

	const hasGifOrVideo = $derived(
		mediaPaths.some((p) => {
			const preview = localPreviews.get(p);
			if (preview) return preview.type === 'image/gif' || preview.type === 'video/mp4';
			return p.endsWith('.gif') || p.endsWith('.mp4');
		})
	);
	const canAttachMore = $derived(
		!disabled && !uploading && !hasGifOrVideo && mediaPaths.length < maxMedia
	);

	function getMaxSize(type: string): number {
		if (type === 'video/mp4') return MAX_VIDEO_SIZE;
		if (type === 'image/gif') return MAX_GIF_SIZE;
		return MAX_IMAGE_SIZE;
	}

	async function handleFiles(files: FileList) {
		error = null;
		for (const file of files) {
			if (mediaPaths.length >= maxMedia) {
				error = `Maximum ${maxMedia} media per tweet.`;
				break;
			}
			const isGifOrVideo = file.type === 'image/gif' || file.type === 'video/mp4';
			if (isGifOrVideo && mediaPaths.length > 0) {
				error = 'GIF/video cannot be combined with other media.';
				break;
			}
			if (!isGifOrVideo && hasGifOrVideo) {
				error = 'Cannot add images when GIF/video is attached.';
				break;
			}
			const maxSize = getMaxSize(file.type);
			if (file.size > maxSize) {
				error = `File "${file.name}" exceeds ${Math.round(maxSize / 1024 / 1024)}MB limit.`;
				break;
			}
			uploading = true;
			try {
				const result = await api.media.upload(file);
				localPreviews.set(result.path, {
					url: URL.createObjectURL(file),
					type: result.media_type
				});
				localPreviews = new Map(localPreviews);
				onmediachange([...mediaPaths, result.path]);
			} catch (err) {
				error = err instanceof Error ? err.message : 'Upload failed';
				break;
			} finally {
				uploading = false;
			}
		}
	}

	function removeMedia(path: string) {
		const preview = localPreviews.get(path);
		if (preview) URL.revokeObjectURL(preview.url);
		localPreviews.delete(path);
		localPreviews = new Map(localPreviews);
		onmediachange(mediaPaths.filter((p) => p !== path));
	}

	function getPreviewUrl(path: string): string {
		return localPreviews.get(path)?.url ?? api.media.fileUrl(path);
	}

	function isVideo(path: string): boolean {
		const preview = localPreviews.get(path);
		return preview?.type === 'video/mp4' || path.endsWith('.mp4');
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (canAttachMore) dragOver = true;
	}

	function handleDragLeave() {
		dragOver = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;
		if (!canAttachMore || !e.dataTransfer?.files.length) return;
		handleFiles(e.dataTransfer.files);
	}

	function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files?.length) handleFiles(input.files);
		input.value = '';
	}
</script>

<div
	class="media-slot"
	class:drag-over={dragOver}
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
	role="region"
	aria-label="Media attachment zone"
>
	{#if mediaCount > 0}
		<div
			class="media-thumbs"
			class:single={mediaCount === 1}
			class:double={mediaCount === 2}
			class:triple={mediaCount === 3}
			class:quad={mediaCount >= 4}
		>
			{#each mediaPaths as path (path)}
				<div class="thumb">
					{#if isVideo(path)}
						<video src={getPreviewUrl(path)} class="thumb-img" muted></video>
						<span class="media-badge"><Film size={12} /> Video</span>
					{:else}
						<img src={getPreviewUrl(path)} alt="" class="thumb-img" />
					{/if}
					<button
						class="remove-btn"
						onclick={() => removeMedia(path)}
						aria-label="Remove media attachment {mediaPaths.indexOf(path) + 1}"
					>
						<X size={12} />
					</button>
					{#if onaltchange && !isVideo(path)}
						<MediaAltBadge
							altText={altTexts[path] ?? ''}
							onchange={(alt) => onaltchange(path, alt)}
						/>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
	{#if canAttachMore}
		<button class="attach-btn" onclick={() => fileInput?.click()} disabled={uploading}>
			<Image size={12} />
			{uploading ? 'Uploading...' : mediaPaths.length > 0 ? 'Add more' : 'Attach media'}
		</button>
		<input
			bind:this={fileInput}
			type="file"
			accept={ACCEPTED_TYPES}
			multiple
			class="hidden"
			onchange={handleFileSelect}
		/>
	{/if}
	{#if error}
		<div class="slot-error" role="alert">{error}</div>
	{/if}
</div>

<style>
	.media-slot {
		margin-top: 0;
		border-radius: 6px;
		transition: all 0.15s ease;
	}

	.media-slot.drag-over {
		border: 1px dashed var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
		padding: 4px;
	}

	.media-thumbs {
		display: grid;
		gap: 2px;
		border-radius: 12px;
		overflow: hidden;
		margin-top: 8px;
		margin-bottom: 8px;
		border: 1px solid var(--color-border-subtle);
	}

	.media-thumbs.single { grid-template-columns: 1fr; }
	.media-thumbs.double { grid-template-columns: 1fr 1fr; }
	.media-thumbs.triple {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}
	.media-thumbs.triple .thumb:first-child { grid-row: 1 / 3; }
	.media-thumbs.quad {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}

	.thumb {
		position: relative;
		overflow: hidden;
		min-height: 80px;
		background: var(--color-surface-active);
	}

	.media-thumbs.single .thumb { aspect-ratio: 16 / 9; }
	.media-thumbs.double .thumb { aspect-ratio: 1; }

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

	.remove-btn {
		position: absolute;
		top: 6px;
		right: 6px;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 50%;
		background: rgba(0, 0, 0, 0.6);
		color: #fff;
		cursor: pointer;
		transition: background 0.15s ease;
		padding: 0;
		backdrop-filter: blur(4px);
	}

	.remove-btn:hover {
		background: rgba(0, 0, 0, 0.85);
	}

	.attach-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 6px 12px;
		border: 1px dashed var(--color-border-subtle);
		border-radius: 8px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
		margin-top: 4px;
	}

	.attach-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 4%, transparent);
	}

	.attach-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.hidden { display: none; }

	.slot-error {
		margin-top: 4px;
		font-size: 11px;
		color: var(--color-danger);
	}

	@media (pointer: coarse) {
		.remove-btn { width: 32px; height: 32px; }
		.attach-btn { min-height: 44px; padding: 8px 12px; }
	}

	@media (prefers-reduced-motion: reduce) {
		.media-slot, .remove-btn, .attach-btn { transition: none; }
	}
</style>
