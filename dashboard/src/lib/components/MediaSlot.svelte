<script lang="ts">
	import { api } from '$lib/api';
	import { Image, X, Film } from 'lucide-svelte';

	let {
		mediaPaths = [],
		onmediachange,
		maxMedia = 4,
		disabled = false
	}: {
		mediaPaths: string[];
		onmediachange: (paths: string[]) => void;
		maxMedia?: number;
		disabled?: boolean;
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
	{#if mediaPaths.length > 0}
		<div class="media-thumbs">
			{#each mediaPaths as path (path)}
				<div class="thumb">
					{#if isVideo(path)}
						<!-- svelte-ignore a11y_media_has_caption -->
						<video src={getPreviewUrl(path)} class="thumb-img"></video>
						<span class="media-badge"><Film size={10} /> Video</span>
					{:else}
						<img src={getPreviewUrl(path)} alt="" class="thumb-img" />
					{/if}
					<button
						class="remove-btn"
						onclick={() => removeMedia(path)}
						aria-label="Remove media"
					>
						<X size={10} />
					</button>
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
		margin-top: 4px;
		border-radius: 6px;
		transition: all 0.15s ease;
	}

	.media-slot.drag-over {
		border: 1px dashed var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
		padding: 4px;
	}

	.media-thumbs {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
		margin-bottom: 4px;
	}

	.thumb {
		position: relative;
		width: 48px;
		height: 48px;
		border-radius: 6px;
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
		bottom: 2px;
		left: 2px;
		display: flex;
		align-items: center;
		gap: 2px;
		font-size: 8px;
		font-weight: 600;
		padding: 1px 3px;
		border-radius: 2px;
		background: rgba(0, 0, 0, 0.7);
		color: #fff;
	}

	.remove-btn {
		position: absolute;
		top: 2px;
		right: 2px;
		width: 16px;
		height: 16px;
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
	}

	.remove-btn:hover {
		background: rgba(0, 0, 0, 0.85);
	}

	.attach-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
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

	.hidden {
		display: none;
	}

	.slot-error {
		margin-top: 4px;
		font-size: 11px;
		color: var(--color-danger);
	}
</style>
