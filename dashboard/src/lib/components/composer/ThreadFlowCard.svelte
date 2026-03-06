<script lang="ts">
	import { onMount } from "svelte";
	import type { ThreadBlock } from "$lib/api";
	import { tweetWeightedLen, wordCount, MAX_TWEET_CHARS } from "$lib/utils/tweetLength";
	import { GripVertical, Merge, Trash2, Image } from "lucide-svelte";
	import MediaSlot from "../MediaSlot.svelte";
	import CharRing from "./CharRing.svelte";

	let {
		block,
		index,
		total,
		avatarUrl = null,
		focused = false,
		assisting = false,
		dragging = false,
		dropTarget = false,
		ontext,
		onfocus,
		onblur,
		onkeydown,
		onmedia,
		onmerge,
		onremove,
		onaddafter,
		ondragstart,
		ondragend,
		ondragover,
		ondragenter,
		ondragleave,
		ondrop,
	}: {
		block: ThreadBlock;
		index: number;
		total: number;
		avatarUrl?: string | null;
		focused?: boolean;
		assisting?: boolean;
		dragging?: boolean;
		dropTarget?: boolean;
		ontext: (text: string) => void;
		onfocus: () => void;
		onblur: () => void;
		onkeydown: (e: KeyboardEvent) => void;
		onmedia: (paths: string[]) => void;
		onmerge: () => void;
		onremove: () => void;
		onaddafter: () => void;
		ondragstart: (e: DragEvent) => void;
		ondragend: () => void;
		ondragover: (e: DragEvent) => void;
		ondragenter: (e: DragEvent) => void;
		ondragleave: (e: DragEvent) => void;
		ondrop: (e: DragEvent) => void;
	} = $props();

	const charCount = $derived(tweetWeightedLen(block.text));
	const overLimit = $derived(charCount > MAX_TWEET_CHARS);
	const warning = $derived(charCount > 260 && !overLimit);
	const isFirst = $derived(index === 0);
	const isLast = $derived(index === total - 1);
	const words = $derived(wordCount(block.text));

	function handleKeydownGuarded(e: KeyboardEvent) {
		if (e.isComposing) return;
		onkeydown(e);
	}

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = "auto";
		el.style.height = el.scrollHeight + "px";
	}

	function handleInput(e: Event) {
		const textarea = e.currentTarget as HTMLTextAreaElement;
		ontext(textarea.value);
		autoResize(textarea);
	}

	let textareaEl: HTMLTextAreaElement | undefined = $state();
	let mediaSlotRef: MediaSlot | undefined = $state();

	onMount(() => {
		if (textareaEl && block.text) {
			autoResize(textareaEl);
		}
	});

	$effect(() => {
		void block.text;
		if (textareaEl) {
			requestAnimationFrame(() => {
				if (textareaEl) autoResize(textareaEl);
			});
		}
	});
</script>

<div
	class="flow-card"
	class:dragging
	class:drop-target={dropTarget}
	class:assisting
	class:focused
	class:is-last={isLast}
	data-block-id={block.id}
	role="listitem"
	ondragover={(e) => ondragover(e)}
	ondragenter={(e) => ondragenter(e)}
	ondragleave={(e) => ondragleave(e)}
	ondrop={(e) => ondrop(e)}
>
	<!-- Gutter: avatar -->
	<div class="card-gutter">
		{#if avatarUrl}
			<img
				src={avatarUrl}
				alt=""
				class="gutter-avatar"
				class:over-limit={overLimit}
			/>
		{:else}
			<div
				class="gutter-avatar-placeholder"
				class:over-limit={overLimit}
			></div>
		{/if}
	</div>

	<!-- Main content area -->
	<div class="card-body">
		<div class="card-writing-area" class:over-limit={overLimit}>
			<textarea
				bind:this={textareaEl}
				class="flow-textarea"
				placeholder={isFirst ? "Start writing..." : "Continue..."}
				value={block.text}
				oninput={handleInput}
				onfocus={() => onfocus()}
				onblur={() => onblur()}
				onkeydown={handleKeydownGuarded}
				aria-label={`Post ${index + 1} of ${total}`}
			></textarea>
			<MediaSlot
				bind:this={mediaSlotRef}
				mediaPaths={block.media_paths}
				onmediachange={(paths) => onmedia(paths)}
			/>
		</div>

		<!-- Card footer: post number + actions -->
		<div class="card-footer">
			<div class="footer-left">
				<span class="footer-badge">#{index + 1}{#if words > 0} &middot; {words} {words === 1 ? "word" : "words"}{/if}</span>
			</div>
			<div class="footer-actions">
				<CharRing current={charCount} />
				<div class="footer-action-group">
					<button
						class="footer-action-btn"
						onclick={() => mediaSlotRef?.triggerAttach()}
						title="Attach media"
						aria-label={`Attach media to post ${index + 1}`}
					>
						<Image size={13} />
					</button>
					<div
						class="footer-handle"
						draggable="true"
						role="button"
						tabindex="-1"
						title="Drag to reorder"
						aria-label={`Reorder post ${index + 1}. Use Alt+Up or Alt+Down to move.`}
						ondragstart={(e) => ondragstart(e)}
						ondragend={() => ondragend()}
					>
						<GripVertical size={13} />
					</div>
					{#if total > 2}
						<button
							class="footer-action-btn"
							onclick={() => onmerge()}
							title="Merge with next ({'\u2318'}{'\u21e7'}M)"
							aria-label={`Merge post ${index + 1} with post ${index + 2}`}
						>
							<Merge size={13} />
						</button>
						<button
							class="footer-action-btn footer-remove"
							onclick={() => onremove()}
							title="Remove post"
							aria-label={`Remove post ${index + 1}`}
						>
							<Trash2 size={13} />
						</button>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	.flow-card {
		position: relative;
		display: flex;
		gap: 14px;
		border-left: 2px solid transparent;
		transition: opacity 0.15s ease, border-color 0.15s ease;
	}

	.flow-card.focused {
		border-left-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
	}

	.flow-card.dragging {
		opacity: 0.4;
	}

	.flow-card.assisting {
		opacity: 0.7;
		pointer-events: none;
	}

	/* ── Gutter (avatar + spine) ──────────── */
	.card-gutter {
		position: relative;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		width: 36px;
		flex-shrink: 0;
		padding-top: 4px;
	}

	.flow-card:not(.is-last) .card-gutter::after {
		content: "";
		position: absolute;
		top: 44px; /* 4px padding-top + 36px avatar + 4px below */
		bottom: -16px; /* extend through the inter-card gap */
		left: 50%;
		transform: translateX(-50%);
		width: 2px;
		background: color-mix(in srgb, var(--color-border-subtle) 40%, transparent);
		border-radius: 1px;
	}

	.gutter-avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
		border: 2px solid transparent;
		transition: border-color 0.15s ease;
	}

	.gutter-avatar.over-limit {
		border-color: var(--color-danger);
	}

	.gutter-avatar-placeholder {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: var(--color-surface-active);
		border: 2px solid transparent;
		flex-shrink: 0;
		transition: border-color 0.15s ease;
	}

	.gutter-avatar-placeholder.over-limit {
		border-color: var(--color-danger);
	}

	/* ── Card body ──────────────────────────── */
	.card-body {
		flex: 1;
		min-width: 0;
	}

	.card-writing-area {
		position: relative;
		padding: 2px 0;
		border-radius: 6px;
		transition: opacity 0.2s ease;
	}

	/* De-emphasis on unfocused cards */
	.flow-card:not(.focused) .card-writing-area {
		opacity: 0.7;
	}

	.flow-card:not(.focused):hover .card-writing-area {
		opacity: 1;
	}

	.flow-card.drop-target .card-writing-area {
		background: color-mix(in srgb, var(--color-accent) 4%, transparent);
		border-radius: 8px;
		padding-left: 12px;
		padding-right: 12px;
	}

	.flow-textarea {
		width: 100%;
		padding: 2px 0;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 15px;
		font-family: var(--font-sans);
		line-height: 1.5;
		letter-spacing: -0.01em;
		caret-color: var(--color-accent);
		text-rendering: optimizeLegibility;
		-webkit-font-smoothing: antialiased;
		resize: none;
		outline: none;
		box-sizing: border-box;
		min-height: 48px;
		overflow: hidden;
	}

	.flow-textarea::placeholder {
		color: var(--color-text-subtle);
		opacity: 0.35;
		font-style: italic;
	}

	/* ── Card Footer ───────────────────────── */
	.card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 0 0;
		margin-top: 6px;
		min-height: 28px;
	}

	.footer-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.footer-badge {
		font-size: 10px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		opacity: 0.6;
		padding: 1px 6px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-surface-active) 50%, transparent);
		letter-spacing: 0.02em;
	}

	.footer-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.card-footer {
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	.flow-card:hover .card-footer,
	.flow-card.focused .card-footer,
	.flow-card:focus-within .card-footer {
		opacity: 1;
	}

	.footer-action-group {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.footer-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 0;
		transition:
			color 0.15s ease,
			background 0.15s ease;
	}

	.footer-action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.footer-remove:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	.footer-handle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		color: var(--color-text-subtle);
		cursor: grab;
		border: none;
		background: none;
		padding: 0;
		border-radius: 4px;
		transition:
			color 0.15s ease,
			background 0.15s ease;
	}

	.footer-handle:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}

	.footer-handle:active {
		cursor: grabbing;
	}

	/* Touch: always show tools */
	@media (hover: none) {
		.card-footer {
			opacity: 0.7;
		}
	}

	@media (pointer: coarse) {
		.footer-action-btn,
		.footer-handle {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.flow-card,
		.card-writing-area,
		.card-writing-area::before,
		.card-footer,
		.footer-action-btn,
		.footer-handle,
		.gutter-avatar,
		.gutter-avatar-placeholder {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.card-gutter {
			width: 28px;
		}

		.gutter-avatar,
		.gutter-avatar-placeholder {
			width: 28px;
			height: 28px;
		}

		.flow-card:not(.is-last) .card-gutter::after {
			top: 36px; /* 4px padding-top + 28px avatar + 4px below */
		}

		.flow-card {
			gap: 8px;
		}
	}
</style>
