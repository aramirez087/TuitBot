<script lang="ts">
	import { api, type ThreadBlock } from '$lib/api';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';
	import { Plus, GripVertical } from 'lucide-svelte';
	import MediaSlot from './MediaSlot.svelte';
	import ThreadCardActions from './composer/ThreadCardActions.svelte';

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
		sorted.push({ id: crypto.randomUUID(), text: '', media_paths: [], order: 0 });
		blocks = normalizeOrder(sorted);
		emitChange();
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

	function getCharCount(text: string): number { return tweetWeightedLen(text); }
	function isOverLimit(text: string): boolean { return getCharCount(text) > MAX_TWEET_CHARS; }
	function isWarning(text: string): boolean { return getCharCount(text) > 260 && !isOverLimit(text); }

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

<div class="thread-composer" role="region" aria-label="Thread editor">
	<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{reorderAnnouncement}</div>

	{#each sortedBlocks as block, i (block.id)}
		<div
			class="tweet-card"
			class:focused={focusedBlockId === block.id}
			class:assisting={assistingBlockId === block.id}
			class:over-limit={isOverLimit(block.text)}
			class:dragging={draggingBlockId === block.id}
			class:drop-target={dropTargetBlockId === block.id}
			data-block-id={block.id}
			role="listitem"
			ondragover={(e) => handleCardDragOver(e, block.id)}
			ondragenter={(e) => handleCardDragEnter(e, block.id)}
			ondragleave={(e) => handleCardDragLeave(e, block.id)}
			ondrop={(e) => handleCardDrop(e, block.id)}
		>
			<div class="card-gutter">
				<div class="card-number" aria-hidden="true">{i + 1}</div>
				<div class="drag-handle" title="Drag to reorder"
					aria-label="Reorder tweet {i + 1}. Use Alt+Up or Alt+Down to move."
					draggable="true" role="button" tabindex="-1"
					ondragstart={(e) => handleDragStart(e, block.id)} ondragend={handleDragEnd}>
					<GripVertical size={14} />
				</div>
			</div>
			<div class="card-body">
				<textarea class="card-textarea" class:over-limit={isOverLimit(block.text)}
					placeholder={i === 0 ? 'Start your thread...' : `Tweet ${i + 1}...`}
					value={block.text}
					oninput={(e) => updateBlockText(block.id, e.currentTarget.value)}
					onfocus={() => (focusedBlockId = block.id)}
					onblur={() => { if (focusedBlockId === block.id) focusedBlockId = null; }}
					onkeydown={(e) => handleCardKeydown(e, block.id)}
					rows={3} aria-label={`Tweet ${i + 1} of ${sortedBlocks.length}`}
				></textarea>
				<MediaSlot mediaPaths={block.media_paths}
					onmediachange={(paths) => updateBlockMedia(block.id, paths)} />
				<div class="card-footer">
					<div class="char-counter" class:over-limit={isOverLimit(block.text)}
						class:warning={isWarning(block.text)} aria-live="polite" aria-label="Character count">
						{getCharCount(block.text)}/{MAX_TWEET_CHARS}
					</div>
					<ThreadCardActions index={i} total={sortedBlocks.length}
						onduplicate={() => duplicateBlock(block.id)} onsplit={() => splitBlock(block.id)}
						onmerge={() => mergeWithNext(block.id)} onremove={() => removeBlock(block.id)} />
				</div>
			</div>
			{#if i < sortedBlocks.length - 1}
				<div class="thread-line" aria-hidden="true"></div>
			{/if}
		</div>
	{/each}

	<button class="add-card-btn" onclick={addBlock} aria-label="Add another tweet to thread">
		<Plus size={14} /> Add tweet
	</button>

	{#if mergeError}<div class="merge-error" role="alert">{mergeError}</div>{/if}

	{#if validationErrors.length > 0}
		<div class="validation-summary" role="status" aria-live="polite">
			{#each validationErrors as err}<p class="validation-error">{err}</p>{/each}
		</div>
	{/if}
</div>

<style>
	.thread-composer { display: flex; flex-direction: column; gap: 0; }
	.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0; }
	.tweet-card { position: relative; display: flex; gap: 8px; padding: 12px; border: 1px solid var(--color-border-subtle); border-radius: 8px; background: var(--color-surface); margin-bottom: 8px; transition: border-color 0.15s ease, box-shadow 0.15s ease, opacity 0.15s ease; }
	.tweet-card.focused { border-color: var(--color-accent); box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-accent) 20%, transparent); }
	.tweet-card.over-limit { border-color: var(--color-danger); }
	.tweet-card.assisting { border-color: var(--color-accent); opacity: 0.7; pointer-events: none; }
	.tweet-card.dragging { opacity: 0.5; }
	.tweet-card.drop-target { border-color: var(--color-accent); border-style: dashed; }
	.card-gutter { display: flex; flex-direction: column; align-items: center; gap: 4px; flex-shrink: 0; width: 24px; padding-top: 2px; }
	.card-number { font-size: 11px; font-weight: 600; color: var(--color-text-muted); font-family: var(--font-mono); line-height: 1; }
	.drag-handle { color: var(--color-text-subtle); cursor: grab; opacity: 0.6; display: flex; align-items: center; transition: opacity 0.15s ease; border: none; background: none; padding: 0; }
	.drag-handle:hover { opacity: 1; }
	.drag-handle:active { cursor: grabbing; }
	.card-body { flex: 1; min-width: 0; }
	.card-textarea { width: 100%; padding: 8px 10px; border: 1px solid var(--color-border); border-radius: 6px; background: var(--color-base); color: var(--color-text); font-size: 13px; font-family: var(--font-sans); line-height: 1.5; resize: vertical; box-sizing: border-box; transition: border-color 0.15s ease; }
	.card-textarea:focus { outline: none; border-color: var(--color-accent); }
	.card-textarea.over-limit { border-color: var(--color-danger); }
	.card-footer { display: flex; align-items: center; justify-content: space-between; margin-top: 4px; }
	.char-counter { font-size: 11px; color: var(--color-text-subtle); font-family: var(--font-mono); }
	.char-counter.warning { color: var(--color-warning); }
	.char-counter.over-limit { color: var(--color-danger); font-weight: 600; }
	.thread-line { position: absolute; left: 23px; bottom: -9px; width: 2px; height: 8px; background: var(--color-border-subtle); }
	.add-card-btn { display: flex; align-items: center; gap: 6px; padding: 8px 12px; border: 1px dashed var(--color-border); border-radius: 6px; background: transparent; color: var(--color-text-muted); font-size: 12px; cursor: pointer; transition: all 0.15s ease; margin-top: 4px; }
	.add-card-btn:hover { border-color: var(--color-accent); color: var(--color-accent); background: color-mix(in srgb, var(--color-accent) 5%, transparent); }
	.merge-error { margin-top: 8px; padding: 8px 12px; border-radius: 6px; background: color-mix(in srgb, var(--color-danger) 8%, transparent); font-size: 12px; color: var(--color-danger); }
	.validation-summary { margin-top: 8px; padding: 8px 12px; border-radius: 6px; background: color-mix(in srgb, var(--color-danger) 8%, transparent); }
	.validation-error { font-size: 12px; color: var(--color-danger); margin: 0; padding: 2px 0; }
	@media (pointer: coarse) {
		.drag-handle { min-width: 44px; min-height: 44px; display: flex; align-items: center; justify-content: center; }
		.add-card-btn { min-height: 44px; }
	}
	@media (max-width: 640px) {
		.tweet-card { padding: 10px; }
		.card-footer { flex-wrap: wrap; gap: 8px; }
		.card-gutter { width: 20px; }
		.card-textarea { font-size: 16px; }
		.thread-line { left: 19px; }
	}
</style>
