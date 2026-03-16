<script lang="ts">
	import { type ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import * as threadOps from '$lib/utils/threadOps';
	import { fly, fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { registerTransferHandler } from '$lib/stores/mediaDrag';
	import ThreadFlowCard from './ThreadFlowCard.svelte';
	import {
		handleCardKeydown,
		handlePaletteAction as laneHandlePaletteAction,
		handleInlineAssist as laneHandleInlineAssist,
	} from '$lib/utils/threadLaneActions';

	let {
		blocks: externalBlocks = [],
		avatarUrl = null,
		displayName = null,
		handle = null,
		onchange,
		onvalidchange,
		onfocusindexchange,
	}: {
		blocks?: ThreadBlock[];
		avatarUrl?: string | null;
		displayName?: string | null;
		handle?: string | null;
		onchange: (blocks: ThreadBlock[]) => void;
		onvalidchange: (valid: boolean) => void;
		onfocusindexchange?: (index: number) => void;
	} = $props();

	const fallbackBlocks = threadOps.createDefaultBlocks();
	const blocks = $derived(externalBlocks.length > 0 ? externalBlocks : fallbackBlocks);

	let focusedBlockId = $state<string | null>(null);
	let draggingBlockId = $state<string | null>(null);
	let dropTargetBlockId = $state<string | null>(null);
	let reorderAnnouncement = $state('');
	let mergeError = $state<string | null>(null);
	let assistingBlockId = $state<string | null>(null);

	const sortedBlocks = $derived(threadOps.sortBlocks(blocks));

	const validationErrors = $derived.by(() => threadOps.validateThread(blocks).errors);

	const canSubmit = $derived(
		blocks.filter((b) => b.text.trim().length > 0).length >= 2 &&
			blocks.every((b) => tweetWeightedLen(b.text) <= MAX_TWEET_CHARS) &&
			blocks.every((b) => b.media_paths.length <= 4),
	);

	$effect(() => { onvalidchange(canSubmit); });

	$effect(() => {
		if (!focusedBlockId || !onfocusindexchange) return;
		const idx = sortedBlocks.findIndex((b) => b.id === focusedBlockId);
		if (idx >= 0) onfocusindexchange(idx);
	});

	function focusBlock(blockId: string, cursorPos?: number) {
		requestAnimationFrame(() => {
			const textarea = document.querySelector(
				`[data-block-id="${blockId}"] textarea`,
			) as HTMLTextAreaElement | null;
			if (textarea) {
				textarea.focus();
				if (cursorPos !== undefined) textarea.setSelectionRange(cursorPos, cursorPos);
			}
		});
	}

	function addBlock() {
		const result = threadOps.addBlock(blocks);
		onchange(result.blocks);
		focusBlock(result.newId);
	}

	function addBlockAfter(afterId: string) {
		const result = threadOps.addBlockAfter(blocks, afterId);
		if (!result) return;
		onchange(result.blocks);
		focusBlock(result.newId);
	}

	function removeBlock(id: string) {
		const result = threadOps.removeBlock(blocks, id);
		if (!result) return;
		onchange(result);
	}

	function updateBlockText(id: string, text: string) {
		onchange(threadOps.updateBlockText(blocks, id, text));
	}

	function updateBlockMedia(id: string, paths: string[]) {
		onchange(threadOps.updateBlockMedia(blocks, id, paths));
	}

	function moveBlock(blockId: string, newIndex: number) {
		const result = threadOps.moveBlockToIndex(blocks, blockId, newIndex);
		if (!result) return;
		onchange(result);
		announce(`Post moved to position ${newIndex + 1}`);
	}

	function duplicateBlock(id: string) {
		const result = threadOps.duplicateBlock(blocks, id);
		if (!result) return;
		onchange(result.blocks);
		focusBlock(result.newId);
	}

	function announce(msg: string) {
		reorderAnnouncement = msg;
		setTimeout(() => { reorderAnnouncement = ''; }, 1000);
	}

	function splitBlock(id: string) {
		const textarea = document.querySelector(
			`[data-block-id="${id}"] textarea`,
		) as HTMLTextAreaElement | null;
		const block = blocks.find((b) => b.id === id);
		if (!block) return;
		const cursorPos = textarea?.selectionStart ?? Math.floor(block.text.length / 2);
		const result = threadOps.splitBlockAt(blocks, id, cursorPos);
		if (!result) return;
		onchange(result.blocks);
		focusBlock(result.newId);
		announce(`Post split. Now ${result.blocks.length} posts in thread.`);
	}

	function showMergeMediaError(a: string[], b: string[]) {
		const total = a.length + b.length;
		if (total > 4) {
			mergeError = `Cannot merge: combined media would exceed 4 (has ${total}).`;
			setTimeout(() => { mergeError = null; }, 3000);
		}
	}

	function mergeWithNext(id: string) {
		const result = threadOps.mergeWithNext(blocks, id);
		if (!result) {
			const sorted = threadOps.sortBlocks(blocks);
			const idx = sorted.findIndex((b) => b.id === id);
			if (idx >= sorted.length - 1 || sorted.length <= 1) return;
			showMergeMediaError(sorted[idx].media_paths, sorted[idx + 1].media_paths);
			return;
		}
		onchange(result.blocks);
		focusBlock(id, result.cursorPos);
		announce(`Posts merged. Now ${result.blocks.length} posts in thread.`);
	}

	function mergeWithPrevious(id: string) {
		const result = threadOps.mergeWithPrevious(blocks, id);
		if (!result) {
			const sorted = threadOps.sortBlocks(blocks);
			const idx = sorted.findIndex((b) => b.id === id);
			if (idx <= 0 || sorted.length <= 1) return;
			showMergeMediaError(sorted[idx - 1].media_paths, sorted[idx].media_paths);
			return;
		}
		onchange(result.blocks);
		focusBlock(result.targetId, result.cursorPos);
		announce(`Posts merged. Now ${result.blocks.length} posts in thread.`);
	}

	function handleMediaTransfer(targetBlockId: string, mediaPath: string, sourceBlockId: string) {
		const result = threadOps.moveMediaBetweenBlocks(blocks, sourceBlockId, targetBlockId, mediaPath);
		if (result) onchange(result);
	}

	$effect(() => {
		registerTransferHandler(handleMediaTransfer);
		return () => registerTransferHandler(null);
	});

	function handleDragStart(e: DragEvent, blockId: string) {
		draggingBlockId = blockId;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setData('text/plain', blockId);
		}
	}

	function handleDragEnd() {
		draggingBlockId = null;
		dropTargetBlockId = null;
	}

	function handleCardDragOver(e: DragEvent, blockId: string) {
		e.preventDefault();
		if (draggingBlockId && draggingBlockId !== blockId) dropTargetBlockId = blockId;
	}

	function handleCardDragEnter(e: DragEvent, blockId: string) {
		e.preventDefault();
		if (draggingBlockId && draggingBlockId !== blockId) dropTargetBlockId = blockId;
	}

	function handleCardDragLeave(e: DragEvent, blockId: string) {
		const related = e.relatedTarget as HTMLElement | null;
		const card = e.currentTarget as HTMLElement;
		if (!related || !card.contains(related)) {
			if (dropTargetBlockId === blockId) dropTargetBlockId = null;
		}
	}

	function handleCardDrop(e: DragEvent, targetBlockId: string) {
		e.preventDefault();
		if (!draggingBlockId || draggingBlockId === targetBlockId) return;
		const targetIndex = sortedBlocks.findIndex((b) => b.id === targetBlockId);
		if (targetIndex !== -1) moveBlock(draggingBlockId, targetIndex);
		draggingBlockId = null;
		dropTargetBlockId = null;
	}

	export function addMediaToFocusedBlock(path: string) {
		const targetId = focusedBlockId ?? sortedBlocks[0]?.id;
		if (!targetId) return;
		const block = blocks.find((b) => b.id === targetId);
		if (!block || block.media_paths.length >= 4) return;
		updateBlockMedia(targetId, [...block.media_paths, path]);
	}

	export function getBlocks(): ThreadBlock[] { return [...blocks]; }
	export function setBlocks(newBlocks: ThreadBlock[]) { onchange([...newBlocks]); }

	export async function handleInlineAssist(voiceCue?: string): Promise<void> {
		await laneHandleInlineAssist(voiceCue, {
			getFocusedBlockId: () => focusedBlockId,
			blocks,
			updateBlockText,
			setAssistingBlockId: (id) => { assistingBlockId = id; },
		});
	}

	export function handlePaletteAction(actionId: string) {
		laneHandlePaletteAction(actionId, {
			getFocusedBlockId: () => focusedBlockId,
			setFocusedBlockId: (id) => { focusedBlockId = id; },
			sortedBlocks,
			addBlock,
			duplicateBlock,
			splitBlock,
			mergeWithNext,
			moveBlock,
			focusBlock,
		});
	}
</script>

<div
	class="flow-lane"
	class:has-multiple={sortedBlocks.length > 1}
	role="list"
	aria-label="Thread editor"
>
	<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
		{reorderAnnouncement}
	</div>

	{#each sortedBlocks as block, i (block.id)}
		<div
			class="card-wrapper"
			in:fly={{ y: 20, duration: 200 }}
			out:fade={{ duration: 150 }}
			animate:flip={{ duration: 250 }}
		>
			<ThreadFlowCard
				{block}
				index={i}
				total={sortedBlocks.length}
				{avatarUrl}
				{displayName}
				{handle}
				focused={focusedBlockId === block.id}
				assisting={assistingBlockId === block.id}
				dragging={draggingBlockId === block.id}
				dropTarget={dropTargetBlockId === block.id}
				ontext={(text) => updateBlockText(block.id, text)}
				onfocus={() => { focusedBlockId = block.id; }}
				onblur={() => { if (focusedBlockId === block.id) focusedBlockId = null; }}
				onkeydown={(e) => handleCardKeydown(e, block.id, {
					blocks, sortedBlocks, addBlockAfter, splitBlock,
					mergeWithPrevious, mergeWithNext, duplicateBlock, moveBlock, focusBlock,
				})}
				onmedia={(paths) => updateBlockMedia(block.id, paths)}
				onmerge={() => mergeWithNext(block.id)}
				onremove={() => removeBlock(block.id)}
				onaddafter={() => addBlockAfter(block.id)}
				onmoveup={i > 0 ? () => moveBlock(block.id, i - 1) : undefined}
				onmovedown={i < sortedBlocks.length - 1 ? () => moveBlock(block.id, i + 1) : undefined}
				ondragstart={(e) => handleDragStart(e, block.id)}
				ondragend={handleDragEnd}
				ondragover={(e) => handleCardDragOver(e, block.id)}
				ondragenter={(e) => handleCardDragEnter(e, block.id)}
				ondragleave={(e) => handleCardDragLeave(e, block.id)}
				ondrop={(e) => handleCardDrop(e, block.id)}
			/>
		</div>
	{/each}
</div>

{#if mergeError}<div class="merge-error" role="alert">{mergeError}</div>{/if}

{#if validationErrors.length > 0}
	<div class="validation-summary" role="status" aria-live="polite">
		{#each validationErrors as err}<p class="validation-error">{err}</p>{/each}
	</div>
{/if}

<style>
	.flow-lane {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.card-wrapper {
		position: relative;
		z-index: 1;
		padding-bottom: 16px;
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

	.merge-error {
		margin-top: 8px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
		font-size: 12px;
		color: var(--color-danger);
	}

	.validation-summary {
		margin-top: 8px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
	}

	.validation-error {
		font-size: 12px;
		color: var(--color-danger);
		margin: 0;
		padding: 2px 0;
	}

	@media (max-width: 640px) {
		.flow-lane { padding-left: 0; }
	}
</style>
