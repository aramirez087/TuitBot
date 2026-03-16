<script lang="ts">
	import { onMount } from 'svelte';
	import type { ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, wordCount, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import MediaSlot from '../MediaSlot.svelte';
	import ThreadFlowCardFooter from './ThreadFlowCardFooter.svelte';

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
		onmoveup,
		onmovedown,
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
		onmoveup?: () => void;
		onmovedown?: () => void;
		ondragstart: (e: DragEvent) => void;
		ondragend: () => void;
		ondragover: (e: DragEvent) => void;
		ondragenter: (e: DragEvent) => void;
		ondragleave: (e: DragEvent) => void;
		ondrop: (e: DragEvent) => void;
	} = $props();

	const charCount = $derived(tweetWeightedLen(block.text));
	const overLimit = $derived(charCount > MAX_TWEET_CHARS);
	const isFirst = $derived(index === 0);
	const isLast = $derived(index === total - 1);
	const words = $derived(wordCount(block.text));

	function handleKeydownGuarded(e: KeyboardEvent) {
		if (e.isComposing) return;
		onkeydown(e);
	}

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = 'auto';
		el.style.height = el.scrollHeight + 'px';
	}

	function handleInput(e: Event) {
		const textarea = e.currentTarget as HTMLTextAreaElement;
		ontext(textarea.value);
		autoResize(textarea);
	}

	let textareaEl: HTMLTextAreaElement | undefined = $state();
	let mediaSlotRef: MediaSlot | undefined = $state();

	onMount(() => {
		if (textareaEl && block.text) autoResize(textareaEl);
	});

	$effect(() => {
		void block.text;
		if (textareaEl) {
			requestAnimationFrame(() => { if (textareaEl) autoResize(textareaEl); });
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
	<!-- Gutter: avatar + spine -->
	<div class="card-gutter">
		{#if avatarUrl}
			<img src={avatarUrl} alt="" class="gutter-avatar" class:over-limit={overLimit} />
		{:else}
			<div class="gutter-avatar-placeholder" class:over-limit={overLimit}></div>
		{/if}
	</div>

	<!-- Main content -->
	<div class="card-body">
		{#if displayName || handle}
			<div class="card-identity">
				{#if displayName}<span class="card-display-name">{displayName}</span>{/if}
				{#if handle}<span class="card-handle">@{handle}</span>{/if}
			</div>
		{/if}
		<div class="card-writing-area" class:over-limit={overLimit}>
			<textarea
				bind:this={textareaEl}
				class="flow-textarea"
				placeholder={isFirst ? 'Start writing...' : 'Continue...'}
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
				blockId={block.id}
			/>
		</div>

		<ThreadFlowCardFooter
			{index}
			{total}
			{charCount}
			{words}
			{isFirst}
			{isLast}
			onMediaAttach={() => mediaSlotRef?.triggerAttach()}
			onMoveUp={onmoveup}
			onMoveDown={onmovedown}
			onMerge={onmerge}
			onRemove={onremove}
			onDragStart={ondragstart}
			onDragEnd={ondragend}
		/>
	</div>
</div>

<style>
	.flow-card {
		position: relative;
		display: flex;
		gap: 14px;
		border-left: 2px solid transparent;
		transition:
			opacity 0.15s ease,
			border-color 0.15s ease;
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

	.flow-card:global(.media-transfer-target) {
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
		border-left-color: var(--color-accent);
		border-radius: 6px;
	}

	/* ── Gutter ───────────────────────────── */
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
		content: '';
		position: absolute;
		top: 44px;
		bottom: -16px;
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

	/* ── Body ─────────────────────────────── */
	.card-body {
		flex: 1;
		min-width: 0;
	}

	.card-identity {
		display: flex;
		align-items: baseline;
		gap: 6px;
		padding-top: 2px;
		margin-bottom: 1px;
	}

	.card-display-name {
		font-size: 14px;
		font-weight: 700;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 200px;
	}

	.card-handle {
		font-size: 13px;
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.card-writing-area {
		position: relative;
		padding: 2px 0;
		border-radius: 6px;
		transition: opacity 0.2s ease;
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

	@media (max-width: 640px) {
		.card-gutter { width: 28px; }

		.gutter-avatar,
		.gutter-avatar-placeholder {
			width: 28px;
			height: 28px;
		}

		.flow-card:not(.is-last) .card-gutter::after {
			top: 36px;
		}

		.flow-card { gap: 8px; }
	}

	@media (prefers-reduced-motion: reduce) {
		.flow-card,
		.card-writing-area,
		.gutter-avatar,
		.gutter-avatar-placeholder {
			transition: none;
		}
	}
</style>
