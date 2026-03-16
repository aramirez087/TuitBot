import { api, type ThreadBlock } from '$lib/api';
import * as threadOps from '$lib/utils/threadOps';

// ── Keyboard handler ──────────────────────────────────────────────────────────

export interface KeydownCtx {
	blocks: ThreadBlock[];
	sortedBlocks: ThreadBlock[];
	addBlockAfter: (id: string) => void;
	splitBlock: (id: string) => void;
	mergeWithPrevious: (id: string) => void;
	mergeWithNext: (id: string) => void;
	duplicateBlock: (id: string) => void;
	moveBlock: (id: string, newIdx: number) => void;
	focusBlock: (id: string) => void;
}

export function handleCardKeydown(
	e: KeyboardEvent,
	blockId: string,
	ctx: KeydownCtx,
): void {
	const {
		blocks,
		sortedBlocks,
		addBlockAfter,
		splitBlock,
		mergeWithPrevious,
		mergeWithNext,
		duplicateBlock,
		moveBlock,
		focusBlock,
	} = ctx;

	// Cmd+Enter: split at cursor (or add block if at end)
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
			const idx = sortedBlocks.findIndex((b) => b.id === blockId);
			if (idx > 0 && sortedBlocks.length > 1) {
				e.preventDefault();
				mergeWithPrevious(blockId);
				return;
			}
		}
	}

	// Tab / Shift+Tab: navigate between blocks
	if (e.key === 'Tab' && !e.altKey && !e.metaKey && !e.ctrlKey) {
		e.preventDefault();
		const idx = sortedBlocks.findIndex((b) => b.id === blockId);
		if (e.shiftKey) {
			if (idx > 0) focusBlock(sortedBlocks[idx - 1].id);
		} else {
			if (idx < sortedBlocks.length - 1) focusBlock(sortedBlocks[idx + 1].id);
		}
		return;
	}

	// Alt+Arrow: reorder
	if (e.altKey && e.key === 'ArrowUp') {
		e.preventDefault();
		const idx = sortedBlocks.findIndex((b) => b.id === blockId);
		if (idx > 0) { moveBlock(blockId, idx - 1); focusBlock(blockId); }
		return;
	}
	if (e.altKey && e.key === 'ArrowDown') {
		e.preventDefault();
		const idx = sortedBlocks.findIndex((b) => b.id === blockId);
		if (idx < sortedBlocks.length - 1) { moveBlock(blockId, idx + 1); focusBlock(blockId); }
		return;
	}

	if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'd') {
		e.preventDefault();
		duplicateBlock(blockId);
		return;
	}
	if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 's' || e.key === 'S')) {
		e.preventDefault();
		splitBlock(blockId);
		return;
	}
	if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'm' || e.key === 'M')) {
		e.preventDefault();
		mergeWithNext(blockId);
		return;
	}
}

// ── Palette action handler ────────────────────────────────────────────────────

export interface PaletteCtx {
	getFocusedBlockId: () => string | null;
	setFocusedBlockId: (id: string) => void;
	sortedBlocks: ThreadBlock[];
	addBlock: () => void;
	duplicateBlock: (id: string) => void;
	splitBlock: (id: string) => void;
	mergeWithNext: (id: string) => void;
	moveBlock: (id: string, newIdx: number) => void;
	focusBlock: (id: string) => void;
}

export function handlePaletteAction(actionId: string, ctx: PaletteCtx): void {
	let focusedBlockId = ctx.getFocusedBlockId();
	if (!focusedBlockId) {
		const first = ctx.sortedBlocks[0];
		if (first) { ctx.setFocusedBlockId(first.id); focusedBlockId = first.id; }
	}
	if (!focusedBlockId) return;

	const { sortedBlocks, addBlock, duplicateBlock, splitBlock, mergeWithNext, moveBlock, focusBlock } = ctx;

	switch (actionId) {
		case 'add-card': addBlock(); break;
		case 'duplicate': duplicateBlock(focusedBlockId); break;
		case 'split': splitBlock(focusedBlockId); break;
		case 'merge': mergeWithNext(focusedBlockId); break;
		case 'move-up': {
			const idx = sortedBlocks.findIndex((b) => b.id === focusedBlockId);
			if (idx > 0) { moveBlock(focusedBlockId, idx - 1); focusBlock(focusedBlockId); }
			break;
		}
		case 'move-down': {
			const idx = sortedBlocks.findIndex((b) => b.id === focusedBlockId);
			if (idx < sortedBlocks.length - 1) { moveBlock(focusedBlockId, idx + 1); focusBlock(focusedBlockId); }
			break;
		}
	}
}

// ── Inline assist handler ─────────────────────────────────────────────────────

export interface AssistCtx {
	getFocusedBlockId: () => string | null;
	blocks: ThreadBlock[];
	updateBlockText: (id: string, text: string) => void;
	setAssistingBlockId: (id: string | null) => void;
}

export async function handleInlineAssist(
	voiceCue: string | undefined,
	ctx: AssistCtx,
): Promise<void> {
	const focusedBlockId = ctx.getFocusedBlockId();
	if (!focusedBlockId) return;
	const block = ctx.blocks.find((b) => b.id === focusedBlockId);
	if (!block) return;
	const textarea = document.querySelector(
		`[data-block-id="${focusedBlockId}"] textarea`,
	) as HTMLTextAreaElement | null;
	if (!textarea) return;
	const start = textarea.selectionStart;
	const end = textarea.selectionEnd;
	const selectedText = start !== end ? block.text.slice(start, end) : block.text;
	if (!selectedText.trim()) return;
	ctx.setAssistingBlockId(focusedBlockId);
	try {
		const result = await api.assist.improve(selectedText, voiceCue || undefined);
		if (start !== end) {
			ctx.updateBlockText(block.id, block.text.slice(0, start) + result.content + block.text.slice(end));
		} else {
			ctx.updateBlockText(block.id, result.content);
		}
	} catch {
		/* Error surfaced via parent */
	} finally {
		ctx.setAssistingBlockId(null);
	}
}

// Re-export threadOps for convenience (avoids duplicate import in lane)
export { threadOps };
