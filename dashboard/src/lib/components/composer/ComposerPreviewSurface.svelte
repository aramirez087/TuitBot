<script lang="ts">
	import { onMount } from "svelte";
	import { X } from "lucide-svelte";
	import { focusTrap } from "$lib/actions/focusTrap";
	import TweetPreview from "../TweetPreview.svelte";

	let {
		mode,
		tweetText,
		blocks,
		tweetMediaPaths,
		tweetLocalPreviews,
		handle,
		avatarUrl = null,
		onclose,
	}: {
		mode: "tweet" | "thread";
		tweetText: string;
		blocks: Array<{ id: string; text: string; media_paths: string[] }>;
		tweetMediaPaths: string[];
		tweetLocalPreviews: Map<string, string>;
		handle: string;
		avatarUrl?: string | null;
		onclose: () => void;
	} = $props();

	const hasTweetContent = $derived(
		tweetText.trim().length > 0 || tweetMediaPaths.length > 0,
	);

	const visibleBlocks = $derived(
		blocks.filter((b) => b.text.trim().length > 0),
	);

	let closeBtn: HTMLButtonElement | undefined = $state();
	let triggerElement: Element | null = null;

	onMount(() => {
		triggerElement = document.activeElement;
		requestAnimationFrame(() => {
			closeBtn?.focus();
		});

		return () => {
			if (triggerElement instanceof HTMLElement) {
				triggerElement.focus();
			}
		};
	});

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onclose();
	}
</script>

<div
	class="preview-overlay"
	role="dialog"
	aria-modal="true"
	aria-label="Post preview"
	use:focusTrap
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="preview-backdrop"
		role="button"
		tabindex="-1"
		onclick={handleBackdropClick}
		onkeydown={(e) => {
			if (e.key === "Enter" || e.key === " ")
				handleBackdropClick(e as any);
		}}
	></div>
	<div class="preview-container">
		<header class="preview-header">
			<h2 class="preview-title">Preview</h2>
			<button
				bind:this={closeBtn}
				class="preview-close"
				onclick={onclose}
				aria-label="Close preview"
			>
				<X size={18} />
			</button>
		</header>
		<div class="preview-scroll">
			{#if mode === "tweet"}
				{#if hasTweetContent}
					<TweetPreview
						text={tweetText}
						mediaPaths={tweetMediaPaths}
						localPreviews={tweetLocalPreviews}
						index={0}
						total={1}
						{handle}
						{avatarUrl}
					/>
				{:else}
					<div class="preview-empty">
						Nothing to preview — start writing
					</div>
				{/if}
			{:else if visibleBlocks.length > 0}
				{#each visibleBlocks as block, i (block.id)}
					<TweetPreview
						text={block.text}
						mediaPaths={block.media_paths}
						index={i}
						total={visibleBlocks.length}
						{handle}
						{avatarUrl}
					/>
				{/each}
			{:else}
				<div class="preview-empty">
					Nothing to preview — start writing
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.preview-overlay {
		position: fixed;
		inset: 0;
		z-index: 2000;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.preview-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.85);
	}

	.preview-container {
		position: relative;
		z-index: 1;
		width: 100%;
		max-width: 600px;
		max-height: calc(100vh - 48px);
		display: flex;
		flex-direction: column;
		background: var(--color-surface);
		border-radius: 12px;
		border: 1px solid var(--color-border);
		box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
		margin: 0 16px;
	}

	.preview-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.preview-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.preview-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.preview-close:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.preview-scroll {
		flex: 1;
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 0 16px 16px;
	}

	.preview-empty {
		padding: 48px 0;
		text-align: center;
		font-size: 14px;
		color: var(--color-text-subtle);
	}

	@media (max-width: 640px) {
		.preview-container {
			max-width: 100%;
			max-height: 100vh;
			border-radius: 0;
			margin: 0;
			height: 100%;
		}
	}

	@media (pointer: coarse) {
		.preview-close {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.preview-close {
			transition: none;
		}
	}
</style>
