<script lang="ts">
	import { api } from '$lib/api';
	import {
		type MediaDimensions,
		X_SLOT_RATIOS,
		calculateCropSeverity,
		isVideoPath,
		CROP_SEVERITY_THRESHOLD
	} from '$lib/utils/mediaDimensions';

	let {
		mediaPaths = [],
		localPreviews
	}: {
		mediaPaths: string[];
		localPreviews?: Map<string, string>;
	} = $props();

	const MAX_MEDIA = 4;

	const displayPaths = $derived(mediaPaths.slice(0, MAX_MEDIA));
	const count = $derived(displayPaths.length);
	const slotRatios = $derived(X_SLOT_RATIOS[count] ?? []);

	let dimensions = $state<Map<string, MediaDimensions>>(new Map());

	function resolveUrl(path: string): string {
		return localPreviews?.get(path) ?? api.media.fileUrl(path);
	}

	function handleImageLoad(path: string, e: Event) {
		const img = e.currentTarget as HTMLImageElement;
		if (img.naturalWidth > 0 && img.naturalHeight > 0) {
			const next = new Map(dimensions);
			next.set(path, { width: img.naturalWidth, height: img.naturalHeight });
			dimensions = next;
		}
	}

	function getCropSeverity(path: string, slotIndex: number): number {
		const dim = dimensions.get(path);
		const ratio = slotRatios[slotIndex];
		if (!dim || ratio === undefined) return 0;
		return calculateCropSeverity(dim, ratio);
	}

	function slotAspectCss(slotIndex: number): string {
		const ratio = slotRatios[slotIndex];
		if (ratio === undefined) return '16 / 9';
		if (ratio === 1) return '1 / 1';
		// Express as approximate integer ratio for CSS
		if (Math.abs(ratio - 16 / 9) < 0.01) return '16 / 9';
		if (Math.abs(ratio - 4 / 5) < 0.01) return '4 / 5';
		if (Math.abs(ratio - 2 / 3) < 0.01) return '2 / 3';
		return `${Math.round(ratio * 100)} / 100`;
	}
</script>

{#if count > 0}
	<div
		class="media-grid"
		class:single={count === 1}
		class:double={count === 2}
		class:triple={count === 3}
		class:quad={count === 4}
	>
		{#each displayPaths as path, i (path)}
			{@const isVideo = isVideoPath(path)}
			{@const severity = getCropSeverity(path, i)}
			{@const showCropBadge = severity > CROP_SEVERITY_THRESHOLD}
			<div class="media-slot" class:slot-first={i === 0} style="aspect-ratio: {slotAspectCss(i)}">
				{#if isVideo}
					<video
						src={resolveUrl(path)}
						preload="metadata"
						muted
						class="media-content"
					></video>
					<div class="play-overlay" aria-hidden="true">
						<svg width="36" height="36" viewBox="0 0 36 36" fill="none">
							<circle cx="18" cy="18" r="17" fill="rgba(0,0,0,0.6)" stroke="white" stroke-width="1.5"/>
							<polygon points="14,11 14,25 26,18" fill="white"/>
						</svg>
					</div>
				{:else}
					<img
						src={resolveUrl(path)}
						alt=""
						loading="lazy"
						class="media-content"
						onload={(e) => handleImageLoad(path, e)}
					/>
				{/if}
				{#if showCropBadge}
					<span class="crop-badge" title="This image will be cropped in this layout">cropped</span>
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.media-grid {
		display: grid;
		gap: 2px;
		margin-top: 8px;
		border-radius: 12px;
		overflow: hidden;
		border: 1px solid var(--color-border-subtle);
	}

	/* 1 image: full width 16:9 */
	.media-grid.single {
		grid-template-columns: 1fr;
	}

	/* 2 images: side by side */
	.media-grid.double {
		grid-template-columns: 1fr 1fr;
	}

	/* 3 images: left tall + right stacked */
	.media-grid.triple {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}

	.media-grid.triple .media-slot.slot-first {
		grid-row: 1 / 3;
	}

	/* 4 images: 2×2 grid */
	.media-grid.quad {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
	}

	.media-slot {
		position: relative;
		overflow: hidden;
		background: var(--color-surface-active);
	}

	.media-content {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.play-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		pointer-events: none;
	}

	.crop-badge {
		position: absolute;
		bottom: 4px;
		right: 4px;
		padding: 2px 5px;
		font-size: 10px;
		font-weight: 500;
		line-height: 1;
		color: #fff;
		background: rgba(0, 0, 0, 0.55);
		border-radius: 3px;
		pointer-events: none;
		letter-spacing: 0.01em;
	}

	/* Fallback for failed loads */
	.media-content:not([src]),
	.media-content[src=''] {
		background: var(--color-surface-active);
	}

	@media (prefers-reduced-motion: reduce) {
		.crop-badge {
			transition: none;
		}
	}
</style>
