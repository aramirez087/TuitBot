<script lang="ts">
	import type { ThreadBlock } from '$lib/api';
	import { Plus } from 'lucide-svelte';
	import ThreadFlowCard from './ThreadFlowCard.svelte';

	let {
		blocks,
		focusedBlockId,
		assistingBlockId,
		draggingBlockId,
		dropTargetBlockId,
		reorderAnnouncement = '',
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
		ondrop,
		onaddcard
	}: {
		blocks: ThreadBlock[];
		focusedBlockId: string | null;
		assistingBlockId: string | null;
		draggingBlockId: string | null;
		dropTargetBlockId: string | null;
		reorderAnnouncement?: string;
		ontext: (id: string, text: string) => void;
		onfocus: (id: string) => void;
		onblur: (id: string) => void;
		onkeydown: (e: KeyboardEvent, id: string) => void;
		onmedia: (id: string, paths: string[]) => void;
		onmerge: (id: string) => void;
		onremove: (id: string) => void;
		ondragstart: (e: DragEvent, id: string) => void;
		ondragend: () => void;
		ondragover: (e: DragEvent, id: string) => void;
		ondragenter: (e: DragEvent, id: string) => void;
		ondragleave: (e: DragEvent, id: string) => void;
		ondrop: (e: DragEvent, id: string) => void;
		onaddcard: () => void;
	} = $props();
</script>

<div class="flow-lane" role="list" aria-label="Thread editor">
	<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{reorderAnnouncement}</div>

	{#each blocks as block, i (block.id)}
		<ThreadFlowCard
			{block}
			index={i}
			total={blocks.length}
			focused={focusedBlockId === block.id}
			assisting={assistingBlockId === block.id}
			dragging={draggingBlockId === block.id}
			dropTarget={dropTargetBlockId === block.id}
			ontext={(text) => ontext(block.id, text)}
			onfocus={() => onfocus(block.id)}
			onblur={() => onblur(block.id)}
			onkeydown={(e) => onkeydown(e, block.id)}
			onmedia={(paths) => onmedia(block.id, paths)}
			onmerge={() => onmerge(block.id)}
			onremove={() => onremove(block.id)}
			ondragstart={(e) => ondragstart(e, block.id)}
			{ondragend}
			ondragover={(e) => ondragover(e, block.id)}
			ondragenter={(e) => ondragenter(e, block.id)}
			ondragleave={(e) => ondragleave(e, block.id)}
			ondrop={(e) => ondrop(e, block.id)}
		/>
	{/each}

	<button class="add-card-btn" onclick={onaddcard} aria-label="Add another tweet to thread">
		<Plus size={14} /> Add tweet
	</button>
</div>

<style>
	.flow-lane {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}

	.add-card-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border: 1px dashed var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
		margin-top: 8px;
		margin-left: 16px;
	}

	.add-card-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
	}

	@media (pointer: coarse) {
		.add-card-btn {
			min-height: 44px;
		}
	}

	@media (max-width: 640px) {
		.add-card-btn {
			margin-left: 12px;
		}
	}
</style>
