<script lang="ts">
	import { api } from '$lib/api';
	import {
		startMediaDrag,
		endMediaDrag,
		isMediaDragActive,
		performTransfer,
	} from '$lib/stores/mediaDrag';
	import MediaThumbGrid from './MediaThumbGrid.svelte';

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
		}),
	);
	const canAttachMore = $derived(
		!disabled && !uploading && !hasGifOrVideo && mediaPaths.length < maxMedia,
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
					type: result.media_type,
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

		if (!draggingPath) {
			const dx = e.clientX - mouseDownInfo.x;
			const dy = e.clientY - mouseDownInfo.y;
			if (Math.abs(dx) < DRAG_THRESHOLD && Math.abs(dy) < DRAG_THRESHOLD) return;
			draggingPath = mouseDownInfo.path;
			startMediaDrag(mouseDownInfo.path, blockId);
			createGhost(e.clientX, e.clientY);
		}

		if (ghostEl) {
			ghostEl.style.left = `${e.clientX - 24}px`;
			ghostEl.style.top = `${e.clientY - 24}px`;
		}

		if (ghostEl) ghostEl.style.pointerEvents = 'none';
		const elUnder = document.elementFromPoint(e.clientX, e.clientY);
		if (ghostEl) ghostEl.style.pointerEvents = '';

		const cardEl = elUnder?.closest('[data-block-id]') as HTMLElement | null;
		const newTargetId = cardEl?.dataset.blockId ?? null;

		if (newTargetId !== currentTargetBlockId) {
			if (currentTargetEl) currentTargetEl.classList.remove('media-transfer-target');
			currentTargetBlockId = newTargetId;
			currentTargetEl = cardEl;
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

		if (currentTargetEl) currentTargetEl.classList.remove('media-transfer-target');
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
		<MediaThumbGrid
			{mediaPaths}
			{localPreviews}
			{draggingPath}
			{altTexts}
			{onaltchange}
			onRemove={removeMedia}
			onThumbMouseDown={handleThumbMouseDown}
		/>
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

	.hidden {
		display: none;
	}

	.slot-error {
		margin-top: 4px;
		font-size: 11px;
		color: var(--color-danger);
	}

	@media (prefers-reduced-motion: reduce) {
		.media-slot {
			transition: none;
		}
	}
</style>
