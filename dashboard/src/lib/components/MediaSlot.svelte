<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import { X, Film, Users, Captions } from 'lucide-svelte';
	import MediaAltBadge from './composer/MediaAltBadge.svelte';
	import { startMediaDrag, endMediaDrag, isMediaDragActive, performTransfer } from '$lib/stores/mediaDrag';

	let {
		mediaPaths = [],
		onmediachange,
		maxMedia = 4,
		disabled = false,
		altTexts = {},
		onaltchange,
		blockId = '',
	}: {
		mediaPaths: string[];
		onmediachange: (paths: string[]) => void;
		maxMedia?: number;
		disabled?: boolean;
		altTexts?: Record<string, string>;
		onaltchange?: (path: string, altText: string) => void;
		blockId?: string;
	} = $props();

	const ACCEPTED_TYPES = 'image/jpeg,image/png,image/webp,image/gif,video/mp4';
	const MAX_IMAGE_SIZE = 5 * 1024 * 1024;
	const MAX_GIF_SIZE = 15 * 1024 * 1024;
	const MAX_VIDEO_SIZE = 512 * 1024 * 1024;

	let uploading = $state(false);
	let error = $state<string | null>(null);
	let fileInput: HTMLInputElement | undefined = $state();
	let dragOver = $state(false);
	let draggingPath = $state<string | null>(null);

	// Mouse-based drag state
	let mouseDownInfo = $state<{ path: string; x: number; y: number } | null>(null);
	let ghostEl: HTMLDivElement | null = null;
	let currentTargetBlockId: string | null = null;
	let currentTargetEl: HTMLElement | null = null;

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

	// ── File drop from Finder (HTML5 DnD for external files only) ──
	function handleDragOver(e: DragEvent) {
		// Only handle external file drops, not intra-page media drags
		if (isMediaDragActive()) return;
		e.preventDefault();
		if (canAttachMore) dragOver = true;
	}

	function handleDragLeave() {
		dragOver = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragOver = false;
		if (isMediaDragActive()) return;
		if (!canAttachMore || !e.dataTransfer?.files.length) return;
		handleFiles(e.dataTransfer.files);
	}

	// ── Mouse-based media drag between tweets ──
	const DRAG_THRESHOLD = 5;

	function handleThumbMouseDown(e: MouseEvent, path: string) {
		// Don't start drag if clicking on buttons inside the thumb
		const target = e.target as HTMLElement;
		if (target.closest('button')) return;
		if (!blockId) return;
		e.preventDefault();
		mouseDownInfo = { path, x: e.clientX, y: e.clientY };
		document.addEventListener('mousemove', handleDocMouseMove);
		document.addEventListener('mouseup', handleDocMouseUp);
	}

	function handleDocMouseMove(e: MouseEvent) {
		if (!mouseDownInfo) return;

		// Check threshold before starting drag
		if (!draggingPath) {
			const dx = e.clientX - mouseDownInfo.x;
			const dy = e.clientY - mouseDownInfo.y;
			if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
			// Start the drag
			draggingPath = mouseDownInfo.path;
			startMediaDrag(mouseDownInfo.path, blockId);
			createGhost(e.clientX, e.clientY);
		}

		// Move ghost
		if (ghostEl) {
			ghostEl.style.left = `${e.clientX - 24}px`;
			ghostEl.style.top = `${e.clientY - 24}px`;
		}

		// Detect target card via elementFromPoint (hide ghost first so it doesn't block)
		if (ghostEl) ghostEl.style.pointerEvents = 'none';
		const elUnder = document.elementFromPoint(e.clientX, e.clientY);
		if (ghostEl) ghostEl.style.pointerEvents = '';

		const cardEl = elUnder?.closest('[data-block-id]') as HTMLElement | null;
		const newTargetId = cardEl?.dataset.blockId ?? null;

		if (newTargetId !== currentTargetBlockId) {
			// Remove highlight from previous target
			if (currentTargetEl) {
				currentTargetEl.classList.remove('media-transfer-target');
			}
			currentTargetBlockId = newTargetId;
			currentTargetEl = cardEl;
			// Add highlight to new target (if different from source)
			if (currentTargetEl && currentTargetBlockId && currentTargetBlockId !== blockId) {
				currentTargetEl.classList.add('media-transfer-target');
			}
		}
	}

	function handleDocMouseUp(_e: MouseEvent) {
		document.removeEventListener('mousemove', handleDocMouseMove);
		document.removeEventListener('mouseup', handleDocMouseUp);

		if (draggingPath && currentTargetBlockId) {
			performTransfer(currentTargetBlockId);
		} else {
			endMediaDrag();
		}

		// Cleanup
		if (currentTargetEl) {
			currentTargetEl.classList.remove('media-transfer-target');
		}
		destroyGhost();
		draggingPath = null;
		mouseDownInfo = null;
		currentTargetBlockId = null;
		currentTargetEl = null;
	}

	function createGhost(x: number, y: number) {
		if (!mouseDownInfo) return;
		const el = document.createElement('div');
		el.className = 'media-drag-ghost';
		el.style.cssText = `
			position: fixed;
			left: ${x - 24}px;
			top: ${y - 24}px;
			width: 48px;
			height: 48px;
			border-radius: 8px;
			overflow: hidden;
			box-shadow: 0 4px 16px rgba(0,0,0,0.3);
			z-index: 9999;
			pointer-events: none;
			opacity: 0.85;
			transition: none;
		`;
		const url = getPreviewUrl(mouseDownInfo.path);
		if (isVideo(mouseDownInfo.path)) {
			const vid = document.createElement('video');
			vid.src = url;
			vid.muted = true;
			vid.style.cssText = 'width:100%;height:100%;object-fit:cover;';
			el.appendChild(vid);
		} else {
			const img = document.createElement('img');
			img.src = url;
			img.style.cssText = 'width:100%;height:100%;object-fit:cover;';
			el.appendChild(img);
		}
		document.body.appendChild(el);
		ghostEl = el;
	}

	function destroyGhost() {
		if (ghostEl) {
			ghostEl.remove();
			ghostEl = null;
		}
	}

	function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files?.length) handleFiles(input.files);
		input.value = '';
	}

	export function triggerAttach() {
		fileInput?.click();
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
				<div
					class="thumb"
					class:thumb-dragging={draggingPath === path}
					role="button"
					tabindex="-1"
					onmousedown={(e) => handleThumbMouseDown(e, path)}
				>
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
		<div class="media-links">
			<button class="media-link disabled" title="Coming soon" aria-label="Tag people (coming soon)">
				<Users size={12} />
				<span>Tag People</span>
			</button>
			{#if onaltchange && mediaPaths.some((p) => !isVideo(p))}
				<button
					class="media-link"
					onclick={() => {
						const badge = document.querySelector<HTMLButtonElement>(
							'.media-slot [aria-label^="Add alt text"], .media-slot [aria-label^="Edit alt text"]'
						);
						badge?.click();
					}}
					aria-label="Edit media descriptions"
				>
					<Captions size={12} />
					<span>Descriptions</span>
				</button>
			{/if}
		</div>
	{/if}
	<input
		bind:this={fileInput}
		type="file"
		accept={ACCEPTED_TYPES}
		multiple
		class="hidden"
		onchange={handleFileSelect}
	/>
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
		cursor: grab;
		transition: opacity 0.15s ease;
		user-select: none;
	}

	.thumb:active {
		cursor: grabbing;
	}

	.thumb.thumb-dragging {
		opacity: 0.4;
	}

	.media-thumbs.single .thumb { aspect-ratio: 16 / 9; }
	.media-thumbs.double .thumb { aspect-ratio: 1; }

	.thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
		pointer-events: none;
		user-select: none;
		-webkit-user-drag: none;
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

	.media-links {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 2px 0;
	}

	.media-link {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: var(--color-text-muted);
		background: none;
		border: none;
		padding: 2px 4px;
		border-radius: 4px;
		cursor: pointer;
		transition: color 0.12s ease;
	}

	.media-link:hover {
		color: var(--color-accent);
	}

	.media-link.disabled {
		opacity: 0.4;
		cursor: default;
	}

	.media-link.disabled:hover {
		color: var(--color-text-muted);
	}

	.hidden { display: none; }

	.slot-error {
		margin-top: 4px;
		font-size: 11px;
		color: var(--color-danger);
	}

	@media (pointer: coarse) {
		.remove-btn { width: 32px; height: 32px; }
	}

	@media (prefers-reduced-motion: reduce) {
		.media-slot, .remove-btn { transition: none; }
	}
</style>
