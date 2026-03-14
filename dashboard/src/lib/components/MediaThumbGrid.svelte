<script lang="ts">
	import { api } from '$lib/api';
	import { X, Film, Users, Captions } from 'lucide-svelte';
	import MediaAltBadge from './composer/MediaAltBadge.svelte';

	interface Props {
		mediaPaths: string[];
		localPreviews: Map<string, { url: string; type: string }>;
		draggingPath: string | null;
		altTexts: Record<string, string>;
		onaltchange?: (path: string, altText: string) => void;
		onRemove: (path: string) => void;
		onThumbMouseDown: (e: MouseEvent, path: string) => void;
	}

	const {
		mediaPaths,
		localPreviews,
		draggingPath,
		altTexts,
		onaltchange,
		onRemove,
		onThumbMouseDown,
	}: Props = $props();

	const mediaCount = $derived(mediaPaths.length);

	function getPreviewUrl(path: string): string {
		return localPreviews.get(path)?.url ?? api.media.fileUrl(path);
	}

	function isVideo(path: string): boolean {
		const preview = localPreviews.get(path);
		return preview?.type === 'video/mp4' || path.endsWith('.mp4');
	}
</script>

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
			onmousedown={(e) => onThumbMouseDown(e, path)}
		>
			{#if isVideo(path)}
				<video src={getPreviewUrl(path)} class="thumb-img" muted></video>
				<span class="media-badge"><Film size={12} /> Video</span>
			{:else}
				<img src={getPreviewUrl(path)} alt="" class="thumb-img" />
			{/if}
			<button
				class="remove-btn"
				onclick={() => onRemove(path)}
				aria-label="Remove media attachment {mediaPaths.indexOf(path) + 1}"
			>
				<X size={12} />
			</button>
			{#if onaltchange && !isVideo(path)}
				<MediaAltBadge
					altText={altTexts[path] ?? ''}
					onchange={(alt: string) => onaltchange(path, alt)}
				/>
			{/if}
		</div>
	{/each}
</div>

<div class="media-links">
	<button
		class="media-link disabled"
		title="Coming soon"
		aria-label="Tag people (coming soon)"
	>
		<Users size={12} />
		<span>Tag People</span>
	</button>
	{#if onaltchange && mediaPaths.some((p) => !isVideo(p))}
		<button
			class="media-link"
			onclick={() => {
				const badge = document.querySelector<HTMLButtonElement>(
					'.media-slot [aria-label^="Add alt text"], .media-slot [aria-label^="Edit alt text"]',
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

<style>
	.media-thumbs {
		display: grid;
		gap: 2px;
		border-radius: 12px;
		overflow: hidden;
		margin: 8px 0;
		border: 1px solid var(--color-border-subtle);
	}

	.media-thumbs.single {
		grid-template-columns: 1fr;
	}

	.media-thumbs.single .thumb-img {
		height: auto;
		max-height: 600px;
	}

	.media-thumbs.double {
		grid-template-columns: 1fr 1fr;
		aspect-ratio: 3 / 2;
	}

	.media-thumbs.triple {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
		aspect-ratio: 3 / 2;
	}

	.media-thumbs.triple .thumb:first-child {
		grid-row: 1 / 3;
	}

	.media-thumbs.quad {
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
		aspect-ratio: 3 / 2;
	}

	.thumb {
		position: relative;
		overflow: hidden;
		background: var(--color-surface-active);
		cursor: grab;
		transition: opacity 0.15s ease;
		user-select: none;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
	}

	.thumb:active {
		cursor: grabbing;
	}

	.thumb.thumb-dragging {
		opacity: 0.4;
	}

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

	@media (pointer: coarse) {
		.remove-btn {
			width: 32px;
			height: 32px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.thumb,
		.remove-btn {
			transition: none;
		}
	}
</style>
