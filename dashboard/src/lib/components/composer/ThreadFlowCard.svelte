<script lang="ts">
	import { onMount } from "svelte";
	import type { ThreadBlock } from "$lib/api";
	import { tweetWeightedLen, MAX_TWEET_CHARS } from "$lib/utils/tweetLength";
	import { GripVertical, Merge, Trash2, Image } from "lucide-svelte";
	import MediaSlot from "../MediaSlot.svelte";

	let {
		block,
		index,
		total,
		avatarUrl = null,
		displayName = null,
		handle = null,
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
		displayName?: string | null;
		handle?: string | null;
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
			<div class="card-meta">
				{#if displayName}
					<span class="meta-name">{displayName}</span>
				{/if}
				{#if handle}
					<span class="meta-handle">@{handle}</span>
				{/if}
			</div>
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

		<!-- Card footer: post number + char count + actions -->
		<div class="card-footer">
			<span class="footer-badge">#{index + 1}</span>
			<div class="footer-center">
				{#if charCount > 240 || overLimit}
					<span
						class="footer-char-count"
						class:over-limit={overLimit}
						class:warning
					>
						{charCount}/{MAX_TWEET_CHARS}
					</span>
				{/if}
			</div>
			<div class="footer-actions">
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

<style>
	.flow-card {
		position: relative;
		display: flex;
		gap: 12px;
		transition: opacity 0.15s ease;
	}

	.flow-card.dragging {
		opacity: 0.4;
	}

	.flow-card.assisting {
		opacity: 0.7;
		pointer-events: none;
	}

	/* ── Gutter (avatar) ───────────────────── */
	.card-gutter {
		display: flex;
		align-items: flex-start;
		justify-content: center;
		width: 36px;
		flex-shrink: 0;
		padding-top: 4px;
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

	.flow-card.focused .gutter-avatar {
		border-color: var(--color-accent);
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

	.flow-card.focused .gutter-avatar-placeholder {
		border-color: var(--color-accent);
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
		padding: 4px 0;
		transition: border-color 0.15s ease;
	}

	.card-writing-area.over-limit {
		border-left: 2px solid var(--color-danger);
		padding-left: 12px;
	}

	.flow-card.drop-target .card-writing-area {
		background: color-mix(in srgb, var(--color-accent) 4%, transparent);
		border-radius: 8px;
		padding-left: 12px;
		padding-right: 12px;
	}

	.card-meta {
		display: flex;
		align-items: baseline;
		gap: 5px;
		min-width: 0;
		margin-bottom: 2px;
	}

	.meta-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 160px;
		line-height: 1;
	}

	.meta-handle {
		font-size: 12px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 120px;
		line-height: 1;
	}

	.flow-textarea {
		width: 100%;
		padding: 4px 0;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 16px;
		font-family: var(--font-sans);
		line-height: 1.55;
		resize: none;
		outline: none;
		box-sizing: border-box;
		min-height: 56px;
		overflow: hidden;
	}

	.flow-textarea::placeholder {
		color: var(--color-text-subtle);
		opacity: 0.5;
	}

	/* ── Card Footer ───────────────────────── */
	.card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 0 0;
		margin-top: 8px;
		border-top: 1px solid color-mix(in srgb, var(--color-border-subtle) 30%, transparent);
		min-height: 28px;
	}

	.footer-badge {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		opacity: 0.5;
		padding: 1px 6px;
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-surface-active) 50%, transparent);
	}

	.footer-center {
		flex: 1;
		display: flex;
		justify-content: center;
	}

	.footer-char-count {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		letter-spacing: -0.02em;
	}

	.footer-char-count.warning {
		color: var(--color-warning);
	}

	.footer-char-count.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	.footer-actions {
		display: flex;
		align-items: center;
		gap: 2px;
		opacity: 0.35;
		transition: opacity 0.15s ease;
	}

	.card-footer:hover .footer-actions,
	.flow-card.focused .footer-actions {
		opacity: 1;
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
		.footer-actions {
			opacity: 1;
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
		.footer-actions,
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

		.flow-card {
			gap: 8px;
		}

		.card-meta {
			display: none;
		}
	}
</style>
