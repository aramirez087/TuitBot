<script lang="ts">
	import type { ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import { GripVertical, Merge, Trash2 } from 'lucide-svelte';
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
		<textarea
			class="flow-textarea"
			placeholder={index === 0 ? 'Start your thread...' : `Tweet ${index + 1}...`}
			value={block.text}
			oninput={(e) => ontext(e.currentTarget.value)}
			onfocus={() => onfocus()}
			onblur={() => onblur()}
			onkeydown={(e) => onkeydown(e)}
			rows={3}
			aria-label={`Tweet ${index + 1} of ${total}`}
		></textarea>
		<MediaSlot mediaPaths={block.media_paths} onmediachange={(paths) => onmedia(paths)} />
	</div>

	{#if !isLast}
		<div class="card-separator">
			<span class="sep-char-count" class:over-limit={overLimit} class:warning>
				{charCount}/{MAX_TWEET_CHARS}
			</span>
			<div class="sep-tools">
				<div
					class="sep-handle"
					draggable="true"
					role="button"
					tabindex="-1"
					title="Drag to reorder"
					aria-label={`Reorder tweet ${index + 1}. Use Alt+Up or Alt+Down to move.`}
					ondragstart={(e) => ondragstart(e)}
					ondragend={() => ondragend()}
				>
					<GripVertical size={12} />
				</div>
				{#if total > 2}
					<button class="sep-action-btn" onclick={() => onmerge()} title="Merge with next (⌘⇧M)">
						<Merge size={12} />
					</button>
					<button class="sep-action-btn sep-remove" onclick={() => onremove()} title="Remove tweet">
						<Trash2 size={12} />
					</button>
				{/if}
			</div>
		</div>
	{:else}
		<div class="last-card-meta">
			<span class="sep-char-count" class:over-limit={overLimit} class:warning>
				{charCount}/{MAX_TWEET_CHARS}
			</span>
		</div>
	{/if}
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
		border-left: 2px solid transparent;
		padding-left: 14px;
		transition: border-color 0.15s ease;
	}

	.card-writing-area.focused {
		border-left-color: var(--color-accent);
	}

	.card-writing-area.over-limit {
		border-left-color: var(--color-danger);
	}

	.flow-card.drop-target .card-writing-area {
		border-left: 2px dashed var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 3%, transparent);
		border-radius: 4px;
	}

	.flow-textarea {
		width: 100%;
		padding: 10px 0;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 14px;
		font-family: var(--font-sans);
		line-height: 1.6;
		resize: none;
		outline: none;
		box-sizing: border-box;
	}

	.flow-textarea::placeholder {
		color: var(--color-text-subtle);
		opacity: 0.6;
	}

	.card-separator {
		display: flex;
		align-items: center;
		height: 28px;
		padding: 0 0 0 14px;
		margin: 0 0 4px;
		border-bottom: 1px solid var(--color-border-subtle);
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

	.last-card-meta {
		display: flex;
		align-items: center;
		height: 24px;
		padding: 0 0 0 14px;
		margin-top: 2px;
	}

	/* Touch: always show separator tools */
	@media (hover: none) {
		.sep-tools {
			opacity: 1;
		}
	}

	@media (pointer: coarse) {
		.sep-handle,
		.sep-action-btn {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (max-width: 640px) {
		.flow-textarea {
			font-size: 16px;
		}

		.card-writing-area {
			padding-left: 12px;
		}
	}
</style>
