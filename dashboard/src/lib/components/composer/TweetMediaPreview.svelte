<script lang="ts">
	import { X, Film, Users, Captions } from 'lucide-svelte';
	import MediaAltBadge from './MediaAltBadge.svelte';
	import type { AttachedMedia } from './TweetEditor.svelte';

	interface Props {
		attachedMedia: AttachedMedia[];
		onRemove: (index: number) => void;
		onMediaChange: (media: AttachedMedia[]) => void;
	}

	const { attachedMedia, onRemove, onMediaChange }: Props = $props();

	const mediaCount = $derived(attachedMedia.length);
</script>

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
			<button class="remove-media-btn" onclick={() => onRemove(i)} aria-label="Remove media">
				<X size={14} />
			</button>
			{#if media.mediaType !== 'video/mp4'}
				<MediaAltBadge
					altText={media.altText ?? ''}
					onchange={(alt: string) => {
						onMediaChange(attachedMedia.map((m, j) => (j === i ? { ...m, altText: alt } : m)));
					}}
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
	{#if attachedMedia.some((m) => m.mediaType !== 'video/mp4')}
		<button
			class="media-link"
			onclick={() => {
				const badge = document.querySelector<HTMLButtonElement>(
					'.media-preview-grid [aria-label^="Add alt text"], .media-preview-grid [aria-label^="Edit alt text"]',
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

	.media-preview-grid.triple .media-thumb:first-child {
		grid-row: 1 / 3;
	}

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
		.remove-media-btn {
			width: 32px;
			height: 32px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.remove-media-btn {
			transition: none;
		}
	}
</style>
