<script lang="ts">
	import { api, type ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import ThreadFlowLane from './composer/ThreadFlowLane.svelte';

	let {
		initialBlocks = undefined,
		onchange,
		onvalidchange
	}: {
		initialBlocks?: ThreadBlock[];
		onchange: (blocks: ThreadBlock[]) => void;
		onvalidchange: (valid: boolean) => void;
	} = $props();

	function createDefaultBlocks(): ThreadBlock[] {
		return [
			{ id: crypto.randomUUID(), text: '', media_paths: [], order: 0 },
			{ id: crypto.randomUUID(), text: '', media_paths: [], order: 1 }
		];
	}

	let blocks = $state<ThreadBlock[]>(createDefaultBlocks());
	let focusedBlockId = $state<string | null>(null);
	let draggingBlockId = $state<string | null>(null);
	let dropTargetBlockId = $state<string | null>(null);
	let reorderAnnouncement = $state('');
	let mergeError = $state<string | null>(null);
	let assistingBlockId = $state<string | null>(null);

	$effect(() => {
		if (initialBlocks && initialBlocks.length > 0) {
			const currentIds = blocks.map((b) => b.id).join(',');
			const incomingIds = initialBlocks.map((b) => b.id).join(',');
			if (currentIds !== incomingIds) {
				blocks = [...initialBlocks];
			}
		}
	});

	const sortedBlocks = $derived([...blocks].sort((a, b) => a.order - b.order));

	const validationErrors = $derived.by(() => {
		const errors: string[] = [];
		const nonEmpty = blocks.filter((b) => b.text.trim().length > 0);
		if (nonEmpty.length < 2) errors.push('Thread needs at least 2 tweets with content.');
		for (const block of blocks) {
			if (tweetWeightedLen(block.text) > MAX_TWEET_CHARS) {
				const idx = sortedBlocks.findIndex((b) => b.id === block.id);
				errors.push(`Tweet ${idx + 1} exceeds ${MAX_TWEET_CHARS} characters.`);
			}
			if (block.media_paths.length > 4) {
				const idx = sortedBlocks.findIndex((b) => b.id === block.id);
				errors.push(`Tweet ${idx + 1} has too many media (max 4).`);
			}
		}
		return errors;
	});

	const canSubmit = $derived(
		blocks.filter((b) => b.text.trim().length > 0).length >= 2 &&
			blocks.every((b) => tweetWeightedLen(b.text) <= MAX_TWEET_CHARS) &&
			blocks.every((b) => b.media_paths.length <= 4)
	);

	$effect(() => { onvalidchange(canSubmit); });

	function emitChange() { onchange([...blocks]); }

	function normalizeOrder(arr: ThreadBlock[]): ThreadBlock[] {
		return arr.map((b, i) => ({ ...b, order: i }));
	}

	function addBlock() {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const newBlock = { id: crypto.randomUUID(), text: '', media_paths: [], order: 0 };
		sorted.push(newBlock);
		blocks = normalizeOrder(sorted);
		emitChange();
		focusBlock(newBlock.id);
	}

	function addBlockAfter(afterId: string) {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const idx = sorted.findIndex((b) => b.id === afterId);
		if (idx === -1) return;
		const newBlock = { id: crypto.randomUUID(), text: '', media_paths: [], order: 0 };
		sorted.splice(idx + 1, 0, newBlock);
		blocks = normalizeOrder(sorted);
		emitChange();
		focusBlock(newBlock.id);
	}

	function removeBlock(id: string) {
		if (blocks.length <= 2) return;
		const sorted = blocks.filter((b) => b.id !== id).sort((a, b) => a.order - b.order);
		blocks = normalizeOrder(sorted);
		emitChange();
	}

	function updateBlockText(id: string, text: string) {
		blocks = blocks.map((b) => (b.id === id ? { ...b, text } : b));
		emitChange();
	}

	function updateBlockMedia(id: string, paths: string[]) {
		blocks = blocks.map((b) => (b.id === id ? { ...b, media_paths: paths } : b));
		emitChange();
	}

	function moveBlock(blockId: string, newIndex: number) {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const currentIndex = sorted.findIndex((b) => b.id === blockId);
		if (currentIndex === -1 || currentIndex === newIndex) return;
		const [moved] = sorted.splice(currentIndex, 1);
		sorted.splice(newIndex, 0, moved);
		blocks = normalizeOrder(sorted);
		reorderAnnouncement = `Tweet moved to position ${newIndex + 1}`;
		setTimeout(() => { reorderAnnouncement = ''; }, 1000);
		emitChange();
	}

	function focusBlock(blockId: string) {
		requestAnimationFrame(() => {
			const textarea = document.querySelector(
				`[data-block-id="${blockId}"] textarea`
			) as HTMLTextAreaElement | null;
			textarea?.focus();
		});
	}

	function handleDragStart(e: DragEvent, blockId: string) {
		draggingBlockId = blockId;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setData('text/plain', blockId);
		}
	}

	function handleDragEnd() { draggingBlockId = null; dropTargetBlockId = null; }

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
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const targetIndex = sorted.findIndex((b) => b.id === targetBlockId);
		if (targetIndex !== -1) moveBlock(draggingBlockId, targetIndex);
		draggingBlockId = null;
		dropTargetBlockId = null;
	}

	function handleCardKeydown(e: KeyboardEvent, blockId: string) {
		// Cmd+Shift+Enter: insert separator or split at cursor
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'Enter') {
			e.preventDefault();
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
		// Backspace at position 0: merge with previous card
		if (e.key === 'Backspace' && !e.metaKey && !e.ctrlKey && !e.shiftKey && !e.altKey) {
			const textarea = e.target as HTMLTextAreaElement;
			if (textarea.selectionStart === 0 && textarea.selectionEnd === 0) {
				const sorted = [...blocks].sort((a, b) => a.order - b.order);
				const idx = sorted.findIndex((b) => b.id === blockId);
				if (idx > 0 && sorted.length > 2) {
					e.preventDefault();
					mergeWithPrevious(blockId);
					return;
				}
			}
		}
		if (e.key === 'Tab' && !e.altKey && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			const sorted = [...blocks].sort((a, b) => a.order - b.order);
			const idx = sorted.findIndex((b) => b.id === blockId);
			if (e.shiftKey) { if (idx > 0) focusBlock(sorted[idx - 1].id); }
			else { if (idx < sorted.length - 1) focusBlock(sorted[idx + 1].id); }
			return;
		}
		if (e.altKey && e.key === 'ArrowUp') {
			e.preventDefault();
			const sorted = [...blocks].sort((a, b) => a.order - b.order);
			const idx = sorted.findIndex((b) => b.id === blockId);
			if (idx > 0) { moveBlock(blockId, idx - 1); focusBlock(blockId); }
			return;
		}
		if (e.altKey && e.key === 'ArrowDown') {
			e.preventDefault();
			const sorted = [...blocks].sort((a, b) => a.order - b.order);
			const idx = sorted.findIndex((b) => b.id === blockId);
			if (idx < sorted.length - 1) { moveBlock(blockId, idx + 1); focusBlock(blockId); }
			return;
		}
		if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'd') {
			e.preventDefault(); duplicateBlock(blockId); return;
		}
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 's' || e.key === 'S')) {
			e.preventDefault(); splitBlock(blockId); return;
		}
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'm' || e.key === 'M')) {
			e.preventDefault(); mergeWithNext(blockId); return;
		}
	}

	function duplicateBlock(id: string) {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const idx = sorted.findIndex((b) => b.id === id);
		if (idx === -1) return;
		const source = sorted[idx];
		const newBlock: ThreadBlock = {
			id: crypto.randomUUID(), text: source.text,
			media_paths: [...source.media_paths], order: 0
		};
		sorted.splice(idx + 1, 0, newBlock);
		blocks = normalizeOrder(sorted);
		emitChange(); focusBlock(newBlock.id);
	}

	function splitBlock(id: string) {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const idx = sorted.findIndex((b) => b.id === id);
		if (idx === -1) return;
		const source = sorted[idx];
		const textarea = document.querySelector(
			`[data-block-id="${id}"] textarea`
		) as HTMLTextAreaElement | null;
		let splitPos = textarea?.selectionStart ?? Math.floor(source.text.length / 2);
		if (splitPos > 0 && splitPos < source.text.length) {
			const before = source.text.slice(0, splitPos);
			const lastSpace = before.lastIndexOf(' ');
			const nextSpace = source.text.indexOf(' ', splitPos);
			if (lastSpace > splitPos - 10) splitPos = lastSpace + 1;
			else if (nextSpace !== -1 && nextSpace < splitPos + 10) splitPos = nextSpace + 1;
		}
		const textBefore = source.text.slice(0, splitPos).trim();
		const textAfter = source.text.slice(splitPos).trim();
		if (!textBefore || !textAfter) return;
		const newBlock: ThreadBlock = { id: crypto.randomUUID(), text: textAfter, media_paths: [], order: 0 };
		sorted[idx] = { ...source, text: textBefore };
		sorted.splice(idx + 1, 0, newBlock);
		blocks = normalizeOrder(sorted);
		emitChange(); focusBlock(newBlock.id);
	}

	function mergeWithNext(id: string) {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const idx = sorted.findIndex((b) => b.id === id);
		if (idx === -1 || idx >= sorted.length - 1 || sorted.length <= 2) return;
		const current = sorted[idx];
		const next = sorted[idx + 1];
		const combinedMedia = [...current.media_paths, ...next.media_paths];
		if (combinedMedia.length > 4) {
			mergeError = `Cannot merge: combined media would exceed 4 (has ${combinedMedia.length}).`;
			setTimeout(() => { mergeError = null; }, 3000);
			return;
		}
		const joinPoint = current.text.length;
		const separator = current.text.endsWith('\n') ? '' : '\n';
		sorted[idx] = { ...current, text: current.text + separator + next.text, media_paths: combinedMedia };
		sorted.splice(idx + 1, 1);
		blocks = normalizeOrder(sorted);
		emitChange();
		requestAnimationFrame(() => {
			const textarea = document.querySelector(
				`[data-block-id="${current.id}"] textarea`
			) as HTMLTextAreaElement | null;
			if (textarea) {
				textarea.focus();
				textarea.setSelectionRange(joinPoint + separator.length, joinPoint + separator.length);
			}
		});
	}

	function mergeWithPrevious(id: string) {
		const sorted = [...blocks].sort((a, b) => a.order - b.order);
		const idx = sorted.findIndex((b) => b.id === id);
		if (idx <= 0 || sorted.length <= 2) return;
		const current = sorted[idx];
		const prev = sorted[idx - 1];
		const combinedMedia = [...prev.media_paths, ...current.media_paths];
		if (combinedMedia.length > 4) {
			mergeError = `Cannot merge: combined media would exceed 4 (has ${combinedMedia.length}).`;
			setTimeout(() => { mergeError = null; }, 3000);
			return;
		}
		const joinPoint = prev.text.length;
		const separator = prev.text.endsWith('\n') ? '' : '\n';
		sorted[idx - 1] = { ...prev, text: prev.text + separator + current.text, media_paths: combinedMedia };
		sorted.splice(idx, 1);
		blocks = normalizeOrder(sorted);
		emitChange();
		requestAnimationFrame(() => {
			const textarea = document.querySelector(
				`[data-block-id="${prev.id}"] textarea`
			) as HTMLTextAreaElement | null;
			if (textarea) {
				textarea.focus();
				textarea.setSelectionRange(joinPoint + separator.length, joinPoint + separator.length);
			}
		});
	}

	export function getBlocks(): ThreadBlock[] { return [...blocks]; }
	export function setBlocks(newBlocks: ThreadBlock[]) { blocks = newBlocks; emitChange(); }

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
		} catch { /* Error surfaced via parent */ } finally { assistingBlockId = null; }
	}

	export function handlePaletteAction(actionId: string) {
		if (!focusedBlockId) {
			const sorted = [...blocks].sort((a, b) => a.order - b.order);
			if (sorted.length > 0) focusedBlockId = sorted[0].id;
		}
		if (!focusedBlockId) return;
		switch (actionId) {
			case 'add-card': addBlock(); break;
			case 'duplicate': duplicateBlock(focusedBlockId); break;
			case 'split': splitBlock(focusedBlockId); break;
			case 'merge': mergeWithNext(focusedBlockId); break;
			case 'move-up': {
				const s = [...blocks].sort((a, b) => a.order - b.order);
				const idx = s.findIndex((b) => b.id === focusedBlockId);
				if (idx > 0) { moveBlock(focusedBlockId, idx - 1); focusBlock(focusedBlockId); }
				break;
			}
			case 'move-down': {
				const s = [...blocks].sort((a, b) => a.order - b.order);
				const idx = s.findIndex((b) => b.id === focusedBlockId);
				if (idx < s.length - 1) { moveBlock(focusedBlockId, idx + 1); focusBlock(focusedBlockId); }
				break;
			}
		}
	}
</script>

<ThreadFlowLane
	blocks={sortedBlocks}
	{focusedBlockId}
	{assistingBlockId}
	{draggingBlockId}
	{dropTargetBlockId}
	{reorderAnnouncement}
	ontext={updateBlockText}
	onfocus={(id) => { focusedBlockId = id; }}
	onblur={(id) => { if (focusedBlockId === id) focusedBlockId = null; }}
	onkeydown={handleCardKeydown}
	onmedia={updateBlockMedia}
	onmerge={mergeWithNext}
	onremove={removeBlock}
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	ondragover={handleCardDragOver}
	ondragenter={handleCardDragEnter}
	ondragleave={handleCardDragLeave}
	ondrop={handleCardDrop}
	onaddcard={addBlock}
/>

{#if mergeError}<div class="merge-error" role="alert">{mergeError}</div>{/if}

{#if validationErrors.length > 0}
	<div class="validation-summary" role="status" aria-live="polite">
		{#each validationErrors as err}<p class="validation-error">{err}</p>{/each}
	</div>
{/if}

<style>
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
</style>
