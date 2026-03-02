<script lang="ts">
	import { onMount } from 'svelte';
	import type { ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import { GripVertical, Merge, Trash2, Plus } from 'lucide-svelte';
	import MediaSlot from '../MediaSlot.svelte';

	let {
		block,
		index,
		total,
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
		onpaste,
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
		onpaste: (text: string) => void;
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

	function handlePaste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text/plain');
		if (text && text.includes('\n\n') && block.text.trim() === '') {
			e.preventDefault();
			onpaste(text);
		}
	}

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
		<div class="spine-dot" class:focused class:over-limit={overLimit} aria-hidden="true"></div>
		<textarea
			bind:this={textareaEl}
			class="flow-textarea"
			placeholder={index === 0 ? 'Start writing...' : 'Continue...'}
			value={block.text}
			oninput={handleInput}
			onfocus={() => onfocus()}
			onblur={() => onblur()}
			onkeydown={handleKeydownGuarded}
			onpaste={handlePaste}
			aria-label={`Post ${index + 1} of ${total}`}
		></textarea>
		<MediaSlot mediaPaths={block.media_paths} onmediachange={(paths) => onmedia(paths)} />
	</div>

	<div class="card-separator" class:last={isLast}>
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
		transition: border-color 0.15s ease;
	}

	/* Drop target visual */
	.flow-card.drop-target .card-writing-area {
		background: color-mix(in srgb, var(--color-accent) 3%, transparent);
		border-radius: 4px;
	}

	/* Spine dot marker — aligned to the lane spine */
	.spine-dot {
		position: absolute;
		left: -21px;
		top: 14px;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		border: 1.5px solid var(--color-border-subtle);
		background: var(--color-surface);
		transition: border-color 0.15s ease, background 0.15s ease;
		z-index: 1;
	}

	.spine-dot.focused {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 15%, var(--color-surface));
	}

	.spine-dot.over-limit {
		border-color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
	}

	.flow-textarea {
		width: 100%;
		padding: 10px 0;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 15px;
		font-family: var(--font-sans);
		line-height: 1.4;
		resize: none;
		outline: none;
		box-sizing: border-box;
		min-height: 60px;
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

	/* Between-block "+" affordance */
	.between-zone {
		position: relative;
		height: 12px;
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
		.spine-dot,
		.sep-tools,
		.between-plus,
		.sep-handle,
		.sep-action-btn {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.flow-textarea {
			font-size: 16px;
		}

		/* Hide spine dot on mobile, show thin accent bar instead */
		.spine-dot {
			display: none;
		}

		.card-writing-area {
			border-left: 1px solid transparent;
			padding-left: 12px;
		}

		.card-writing-area.focused {
			border-left-color: var(--color-accent);
		}

		.card-writing-area.over-limit {
			border-left-color: var(--color-danger);
		}

		.between-plus {
			margin-left: 0;
		}
	}
</style>
