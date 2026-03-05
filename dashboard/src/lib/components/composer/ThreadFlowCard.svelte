<script lang="ts">
	import { onMount } from 'svelte';
	import type { ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import { GripVertical, Merge, Trash2, Plus } from 'lucide-svelte';
	import MediaSlot from '../MediaSlot.svelte';
	import ThreadCardHeader from './ThreadCardHeader.svelte';

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
		ondrop
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
	const isLast = $derived(index >= total - 1);

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
	data-block-id={block.id}
	role="listitem"
	ondragover={(e) => ondragover(e)}
	ondragenter={(e) => ondragenter(e)}
	ondragleave={(e) => ondragleave(e)}
	ondrop={(e) => ondrop(e)}
>
	<div class="card-writing-area" class:focused class:over-limit={overLimit}>
		<ThreadCardHeader
			{avatarUrl}
			{displayName}
			{handle}
			{index}
			{total}
			{focused}
			overLimit={overLimit}
		/>
		<textarea
			bind:this={textareaEl}
			class="flow-textarea"
			placeholder={index === 0 ? 'Start writing...' : 'Continue...'}
			value={block.text}
			oninput={handleInput}
			onfocus={() => onfocus()}
			onblur={() => onblur()}
			onkeydown={handleKeydownGuarded}
			aria-label={`Post ${index + 1} of ${total}`}
		></textarea>
		<MediaSlot mediaPaths={block.media_paths} onmediachange={(paths) => onmedia(paths)} />
	</div>

	<div class="card-separator" class:last={isLast}>
		<span class="sep-post-number">#{index + 1}</span>
		{#if charCount > 240 || overLimit}
			<span class="sep-char-count" class:over-limit={overLimit} class:warning>
				{charCount}/{MAX_TWEET_CHARS}
			</span>
		{/if}
		<div class="sep-tools">
			<div
				class="sep-handle"
				draggable="true"
				role="button"
				tabindex="-1"
				title="Drag to reorder"
				aria-label={`Reorder post ${index + 1}. Use Alt+Up or Alt+Down to move.`}
				ondragstart={(e) => ondragstart(e)}
				ondragend={() => ondragend()}
			>
				<GripVertical size={12} />
			</div>
			{#if total > 2}
				<button
					class="sep-action-btn"
					onclick={() => onmerge()}
					title="Merge with next ({'\u2318'}{'\u21e7'}M)"
					aria-label={`Merge post ${index + 1} with post ${index + 2}`}
				>
					<Merge size={12} />
				</button>
				<button
					class="sep-action-btn sep-remove"
					onclick={() => onremove()}
					title="Remove post"
					aria-label={`Remove post ${index + 1}`}
				>
					<Trash2 size={12} />
				</button>
			{/if}
		</div>
	</div>

	<button
		class="between-zone"
		tabindex="-1"
		aria-label={`Add post after post ${index + 1}`}
		onclick={() => onaddafter()}
	>
		<span class="between-plus">
			<Plus size={10} />
		</span>
	</button>
</div>

<style>
	.flow-card {
		position: relative;
		transition: opacity 0.15s ease;
	}

	.flow-card.dragging {
		opacity: 0.4;
	}

	.flow-card.assisting {
		opacity: 0.7;
		pointer-events: none;
	}

	.card-writing-area {
		position: relative;
		background: color-mix(in srgb, var(--color-surface) 60%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-border-subtle) 40%, transparent);
		border-radius: 10px;
		padding: 12px 16px;
		transition: border-color 0.15s ease, background 0.15s ease;
	}

	.card-writing-area.focused {
		border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
		background: color-mix(in srgb, var(--color-surface) 80%, transparent);
	}

	.card-writing-area.over-limit {
		border-color: color-mix(in srgb, var(--color-danger) 30%, transparent);
	}

	.flow-card.drop-target .card-writing-area {
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
		border-color: color-mix(in srgb, var(--color-accent) 20%, transparent);
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
		min-height: 72px;
		overflow: hidden;
	}

	.flow-textarea::placeholder {
		color: var(--color-text-subtle);
		opacity: 0.6;
	}

	.card-separator {
		display: flex;
		align-items: center;
		height: 24px;
		padding: 0;
		margin: 0;
		gap: 8px;
	}

	.sep-post-number {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		opacity: 0.6;
	}

	.sep-char-count {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		min-width: 60px;
	}

	.sep-char-count.warning {
		color: var(--color-warning);
	}

	.sep-char-count.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	.sep-tools {
		display: flex;
		align-items: center;
		gap: 2px;
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	.card-separator:hover .sep-tools {
		opacity: 1;
	}

	.sep-handle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		color: var(--color-text-subtle);
		cursor: grab;
		border: none;
		background: none;
		padding: 0;
		border-radius: 3px;
		transition: color 0.15s ease, background 0.15s ease;
	}

	.sep-handle:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}

	.sep-handle:active {
		cursor: grabbing;
	}

	.sep-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
		padding: 0;
	}

	.sep-action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.sep-action-btn.sep-remove:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	.between-zone {
		position: relative;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: flex-start;
		cursor: pointer;
	}

	.between-plus {
		opacity: 0;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		border: 1px solid var(--color-border-subtle);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		background: var(--color-surface);
		transition: opacity 0.15s ease, border-color 0.15s ease, color 0.15s ease;
		margin-left: -25px;
		z-index: 2;
	}

	.between-zone:hover .between-plus,
	.between-zone:focus-within .between-plus {
		opacity: 1;
	}

	.between-zone:hover .between-plus {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	/* Touch: always show separator tools */
	@media (hover: none) {
		.sep-tools {
			opacity: 1;
		}
		.between-plus {
			opacity: 1;
		}
	}

	@media (pointer: coarse) {
		.sep-handle,
		.sep-action-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.between-zone {
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.flow-card,
		.card-writing-area,
		.sep-tools,
		.between-plus,
		.sep-handle,
		.sep-action-btn {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.card-writing-area { padding: 10px 12px; }
		.between-plus { margin-left: 0; }
	}
</style>
