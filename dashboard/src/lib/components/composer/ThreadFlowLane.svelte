<script lang="ts">
	import { api, type ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import * as threadOps from '$lib/utils/threadOps';
	import ThreadFlowCard from './ThreadFlowCard.svelte';

	let {
		blocks: externalBlocks = [],
		onchange,
		onvalidchange
	}: {
		blocks?: ThreadBlock[];
		onchange: (blocks: ThreadBlock[]) => void;
		onvalidchange: (valid: boolean) => void;
	} = $props();

	// ── State ──────────────────────────────────────────────
	// Stable fallback for when parent has no blocks yet
	const fallbackBlocks = threadOps.createDefaultBlocks();
	// Single source of truth: always derived from parent prop
	const blocks = $derived(externalBlocks.length > 0 ? externalBlocks : fallbackBlocks);

	let focusedBlockId = $state<string | null>(null);
	let draggingBlockId = $state<string | null>(null);
	let dropTargetBlockId = $state<string | null>(null);
	let reorderAnnouncement = $state('');
	let mergeError = $state<string | null>(null);
	let assistingBlockId = $state<string | null>(null);

	// ── Derived ────────────────────────────────────────────
	const sortedBlocks = $derived(threadOps.sortBlocks(blocks));

	const validationErrors = $derived.by(() => {
		const result = threadOps.validateThread(blocks);
		return result.errors;
	});

	const canSubmit = $derived(
		blocks.filter((b) => b.text.trim().length > 0).length >= 2 &&
			blocks.every((b) => tweetWeightedLen(b.text) <= MAX_TWEET_CHARS) &&
			blocks.every((b) => b.media_paths.length <= 4)
	);

	$effect(() => { onvalidchange(canSubmit); });

	// ── Helpers ────────────────────────────────────────────
	function focusBlock(blockId: string, cursorPos?: number) {
		requestAnimationFrame(() => {
			const textarea = document.querySelector(
				`[data-block-id="${blockId}"] textarea`
			) as HTMLTextAreaElement | null;
			if (textarea) {
				textarea.focus();
				if (cursorPos !== undefined) {
					textarea.setSelectionRange(cursorPos, cursorPos);
				}
			}
		});
	}

	// ── Block operations ───────────────────────────────────
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
			`[data-block-id="${id}"] textarea`
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

	function mergeWithNext(id: string) {
		const result = threadOps.mergeWithNext(blocks, id);
		if (!result) {
			const sorted = threadOps.sortBlocks(blocks);
			const idx = sorted.findIndex((b) => b.id === id);
			if (idx >= sorted.length - 1 || sorted.length <= 2) return;
			const current = sorted[idx];
			const next = sorted[idx + 1];
			const combinedMedia = [...current.media_paths, ...next.media_paths];
			if (combinedMedia.length > 4) {
				mergeError = `Cannot merge: combined media would exceed 4 (has ${combinedMedia.length}).`;
				setTimeout(() => { mergeError = null; }, 3000);
			}
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
			if (idx <= 0 || sorted.length <= 2) return;
			const prev = sorted[idx - 1];
			const current = sorted[idx];
			const combinedMedia = [...prev.media_paths, ...current.media_paths];
			if (combinedMedia.length > 4) {
				mergeError = `Cannot merge: combined media would exceed 4 (has ${combinedMedia.length}).`;
				setTimeout(() => { mergeError = null; }, 3000);
			}
			return;
		}
		onchange(result.blocks);
		focusBlock(result.targetId, result.cursorPos);
		announce(`Posts merged. Now ${result.blocks.length} posts in thread.`);
	}

	// ── Paste handler ──────────────────────────────────────
	function handlePaste(blockId: string, pastedText: string) {
		const block = blocks.find((b) => b.id === blockId);
		if (!block || block.text.trim() !== '') return;
		const result = threadOps.splitFromPaste(blocks, blockId, pastedText);
		if (!result) return;
		onchange(result.blocks);
		focusBlock(result.lastNewId);
	}

	// ── Drag-and-drop ──────────────────────────────────────
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
		const sorted = threadOps.sortBlocks(blocks);
		const targetIndex = sorted.findIndex((b) => b.id === targetBlockId);
		if (targetIndex !== -1) moveBlock(draggingBlockId, targetIndex);
		draggingBlockId = null;
		dropTargetBlockId = null;
	}

	// ── Keyboard handling ──────────────────────────────────
	function handleCardKeydown(e: KeyboardEvent, blockId: string) {
		// Cmd+Enter: split at cursor (fast path for thread mode)
		if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'Enter') {
			e.preventDefault();
			e.stopPropagation();
			const textarea = e.target as HTMLTextAreaElement;
			const block = blocks.find((b) => b.id === blockId);
			if (!block) return;
			if (textarea.selectionStart >= block.text.length || block.text.trim() === '') {
				addBlockAfter(blockId);
			} else {
				splitBlock(blockId);
			}
			return;
		}

		// Backspace at position 0: merge with previous
		if (e.key === 'Backspace' && !e.metaKey && !e.ctrlKey && !e.shiftKey && !e.altKey) {
			const textarea = e.target as HTMLTextAreaElement;
			if (textarea.selectionStart === 0 && textarea.selectionEnd === 0) {
				const sorted = threadOps.sortBlocks(blocks);
				const idx = sorted.findIndex((b) => b.id === blockId);
				if (idx > 0 && sorted.length > 2) {
					e.preventDefault();
					mergeWithPrevious(blockId);
					return;
				}
			}
		}

		// Tab / Shift+Tab: navigate between blocks
		if (e.key === 'Tab' && !e.altKey && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			const sorted = threadOps.sortBlocks(blocks);
			const idx = sorted.findIndex((b) => b.id === blockId);
			if (e.shiftKey) { if (idx > 0) focusBlock(sorted[idx - 1].id); }
			else { if (idx < sorted.length - 1) focusBlock(sorted[idx + 1].id); }
			return;
		}

		// Alt+Arrow: reorder
		if (e.altKey && e.key === 'ArrowUp') {
			e.preventDefault();
			const sorted = threadOps.sortBlocks(blocks);
			const idx = sorted.findIndex((b) => b.id === blockId);
			if (idx > 0) { moveBlock(blockId, idx - 1); focusBlock(blockId); }
			return;
		}
		if (e.altKey && e.key === 'ArrowDown') {
			e.preventDefault();
			const sorted = threadOps.sortBlocks(blocks);
			const idx = sorted.findIndex((b) => b.id === blockId);
			if (idx < sorted.length - 1) { moveBlock(blockId, idx + 1); focusBlock(blockId); }
			return;
		}

		// Cmd+D: duplicate
		if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'd') {
			e.preventDefault();
			duplicateBlock(blockId);
			return;
		}

		// Cmd+Shift+S: split (secondary shortcut)
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 's' || e.key === 'S')) {
			e.preventDefault();
			splitBlock(blockId);
			return;
		}

		// Cmd+Shift+M: merge with next
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'm' || e.key === 'M')) {
			e.preventDefault();
			mergeWithNext(blockId);
			return;
		}
	}

	// ── Exported API (matches ThreadComposer contract) ────
	export function getBlocks(): ThreadBlock[] { return [...blocks]; }

	export function setBlocks(newBlocks: ThreadBlock[]) {
		onchange([...newBlocks]);
	}

	export async function handleInlineAssist(voiceCue?: string): Promise<void> {
		if (!focusedBlockId) return;
		const block = blocks.find((b) => b.id === focusedBlockId);
		if (!block) return;
		const textarea = document.querySelector(
			`[data-block-id="${focusedBlockId}"] textarea`
		) as HTMLTextAreaElement | null;
		if (!textarea) return;
		const start = textarea.selectionStart;
		const end = textarea.selectionEnd;
		const selectedText = start !== end ? block.text.slice(start, end) : block.text;
		if (!selectedText.trim()) return;
		assistingBlockId = focusedBlockId;
		try {
			const result = await api.assist.improve(selectedText, voiceCue || undefined);
			if (start !== end) {
				updateBlockText(block.id, block.text.slice(0, start) + result.content + block.text.slice(end));
			} else {
				updateBlockText(block.id, result.content);
			}
		} catch {
			/* Error surfaced via parent */
		} finally {
			assistingBlockId = null;
		}
	}

	export function handlePaletteAction(actionId: string) {
		if (!focusedBlockId) {
			const sorted = threadOps.sortBlocks(blocks);
			if (sorted.length > 0) focusedBlockId = sorted[0].id;
		}
		if (!focusedBlockId) return;
		switch (actionId) {
			case 'add-card': addBlock(); break;
			case 'duplicate': duplicateBlock(focusedBlockId); break;
			case 'split': splitBlock(focusedBlockId); break;
			case 'merge': mergeWithNext(focusedBlockId); break;
			case 'move-up': {
				const s = threadOps.sortBlocks(blocks);
				const idx = s.findIndex((b) => b.id === focusedBlockId);
				if (idx > 0) { moveBlock(focusedBlockId, idx - 1); focusBlock(focusedBlockId); }
				break;
			}
			case 'move-down': {
				const s = threadOps.sortBlocks(blocks);
				const idx = s.findIndex((b) => b.id === focusedBlockId);
				if (idx < s.length - 1) { moveBlock(focusedBlockId, idx + 1); focusBlock(focusedBlockId); }
				break;
			}
		}
	}
</script>

<div class="flow-lane" role="list" aria-label="Thread editor">
	<div class="lane-spine" aria-hidden="true"></div>
	<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{reorderAnnouncement}</div>

	{#each sortedBlocks as block, i (block.id)}
		<ThreadFlowCard
			{block}
			index={i}
			total={sortedBlocks.length}
			focused={focusedBlockId === block.id}
			assisting={assistingBlockId === block.id}
			dragging={draggingBlockId === block.id}
			dropTarget={dropTargetBlockId === block.id}
			ontext={(text) => updateBlockText(block.id, text)}
			onfocus={() => { focusedBlockId = block.id; }}
			onblur={() => { if (focusedBlockId === block.id) focusedBlockId = null; }}
			onkeydown={(e) => handleCardKeydown(e, block.id)}
			onmedia={(paths) => updateBlockMedia(block.id, paths)}
			onmerge={() => mergeWithNext(block.id)}
			onremove={() => removeBlock(block.id)}
			onaddafter={() => addBlockAfter(block.id)}
			onpaste={(text) => handlePaste(block.id, text)}
			ondragstart={(e) => handleDragStart(e, block.id)}
			ondragend={handleDragEnd}
			ondragover={(e) => handleCardDragOver(e, block.id)}
			ondragenter={(e) => handleCardDragEnter(e, block.id)}
			ondragleave={(e) => handleCardDragLeave(e, block.id)}
			ondrop={(e) => handleCardDrop(e, block.id)}
		/>
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
		padding-left: 32px;
	}

	.lane-spine {
		position: absolute;
		left: 15px;
		top: 24px;
		bottom: 24px;
		width: 1px;
		background: color-mix(in srgb, var(--color-border-subtle) 60%, transparent);
		pointer-events: none;
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
		.flow-lane {
			padding-left: 0;
		}

		.lane-spine {
			display: none;
		}
	}
</style>
