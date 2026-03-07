/**
 * Pure, stateless utility functions for thread block operations.
 * No DOM access, no Svelte imports — testable in isolation.
 */

import type { ThreadBlock } from '$lib/api';
import { tweetWeightedLen, MAX_TWEET_CHARS } from './tweetLength';

export function createDefaultBlocks(): ThreadBlock[] {
	return [
		{ id: crypto.randomUUID(), text: '', media_paths: [], order: 0 },
		{ id: crypto.randomUUID(), text: '', media_paths: [], order: 1 }
	];
}

export function normalizeOrder(blocks: ThreadBlock[]): ThreadBlock[] {
	return blocks.map((b, i) => ({ ...b, order: i }));
}

export function sortBlocks(blocks: ThreadBlock[]): ThreadBlock[] {
	return [...blocks].sort((a, b) => a.order - b.order);
}

export function addBlock(blocks: ThreadBlock[]): { blocks: ThreadBlock[]; newId: string } {
	const sorted = sortBlocks(blocks);
	const newBlock: ThreadBlock = { id: crypto.randomUUID(), text: '', media_paths: [], order: 0 };
	sorted.push(newBlock);
	return { blocks: normalizeOrder(sorted), newId: newBlock.id };
}

export function addBlockAfter(
	blocks: ThreadBlock[],
	afterId: string
): { blocks: ThreadBlock[]; newId: string } | null {
	const sorted = sortBlocks(blocks);
	const idx = sorted.findIndex((b) => b.id === afterId);
	if (idx === -1) return null;
	const newBlock: ThreadBlock = { id: crypto.randomUUID(), text: '', media_paths: [], order: 0 };
	sorted.splice(idx + 1, 0, newBlock);
	return { blocks: normalizeOrder(sorted), newId: newBlock.id };
}

export function removeBlock(blocks: ThreadBlock[], id: string): ThreadBlock[] | null {
	if (blocks.length <= 1) return null;
	const sorted = blocks.filter((b) => b.id !== id).sort((a, b) => a.order - b.order);
	return normalizeOrder(sorted);
}

export function updateBlockText(blocks: ThreadBlock[], id: string, text: string): ThreadBlock[] {
	return blocks.map((b) => (b.id === id ? { ...b, text } : b));
}

export function updateBlockMedia(
	blocks: ThreadBlock[],
	id: string,
	paths: string[]
): ThreadBlock[] {
	return blocks.map((b) => (b.id === id ? { ...b, media_paths: paths } : b));
}

export function moveBlock(
	blocks: ThreadBlock[],
	blockId: string,
	direction: 'up' | 'down'
): ThreadBlock[] | null {
	const sorted = sortBlocks(blocks);
	const idx = sorted.findIndex((b) => b.id === blockId);
	if (idx === -1) return null;
	const newIdx = direction === 'up' ? idx - 1 : idx + 1;
	if (newIdx < 0 || newIdx >= sorted.length) return null;
	const [moved] = sorted.splice(idx, 1);
	sorted.splice(newIdx, 0, moved);
	return normalizeOrder(sorted);
}

export function moveBlockToIndex(
	blocks: ThreadBlock[],
	blockId: string,
	newIndex: number
): ThreadBlock[] | null {
	const sorted = sortBlocks(blocks);
	const currentIndex = sorted.findIndex((b) => b.id === blockId);
	if (currentIndex === -1 || currentIndex === newIndex) return null;
	const [moved] = sorted.splice(currentIndex, 1);
	sorted.splice(newIndex, 0, moved);
	return normalizeOrder(sorted);
}

export function duplicateBlock(
	blocks: ThreadBlock[],
	id: string
): { blocks: ThreadBlock[]; newId: string } | null {
	const sorted = sortBlocks(blocks);
	const idx = sorted.findIndex((b) => b.id === id);
	if (idx === -1) return null;
	const source = sorted[idx];
	const newBlock: ThreadBlock = {
		id: crypto.randomUUID(),
		text: source.text,
		media_paths: [...source.media_paths],
		order: 0
	};
	sorted.splice(idx + 1, 0, newBlock);
	return { blocks: normalizeOrder(sorted), newId: newBlock.id };
}

/**
 * Split a block at the given cursor position.
 * Returns null if either half would be empty after trim.
 */
export function splitBlockAt(
	blocks: ThreadBlock[],
	id: string,
	cursorPos: number
): { blocks: ThreadBlock[]; newId: string } | null {
	const sorted = sortBlocks(blocks);
	const idx = sorted.findIndex((b) => b.id === id);
	if (idx === -1) return null;
	const source = sorted[idx];

	const splitPos = cursorPos;

	// If cursor is at start or end, just add an empty block
	if (splitPos <= 0 || splitPos >= source.text.length) {
		if (source.text.trim() === '') return null;
		const newBlock: ThreadBlock = {
			id: crypto.randomUUID(),
			text: '',
			media_paths: [],
			order: 0
		};
		if (splitPos <= 0) {
			sorted.splice(idx, 0, newBlock);
		} else {
			sorted.splice(idx + 1, 0, newBlock);
		}
		return { blocks: normalizeOrder(sorted), newId: newBlock.id };
	}

	const textBefore = source.text.slice(0, splitPos).trim();
	const textAfter = source.text.slice(splitPos).trim();
	if (!textBefore && !textAfter) return null;

	const newBlock: ThreadBlock = {
		id: crypto.randomUUID(),
		text: textAfter,
		media_paths: [],
		order: 0
	};
	sorted[idx] = { ...source, text: textBefore };
	sorted.splice(idx + 1, 0, newBlock);
	return { blocks: normalizeOrder(sorted), newId: newBlock.id };
}

/**
 * Merge a block with the next block in order.
 * Returns null if at end, at minimum 2 blocks, or combined media > 4.
 */
export function mergeWithNext(
	blocks: ThreadBlock[],
	id: string
): { blocks: ThreadBlock[]; cursorPos: number } | null {
	const sorted = sortBlocks(blocks);
	const idx = sorted.findIndex((b) => b.id === id);
	if (idx === -1 || idx >= sorted.length - 1 || sorted.length <= 1) return null;

	const current = sorted[idx];
	const next = sorted[idx + 1];
	const combinedMedia = [...current.media_paths, ...next.media_paths];
	if (combinedMedia.length > 4) return null;

	const joinPoint = current.text.length;
	const separator = current.text.endsWith('\n') ? '' : '\n';
	sorted[idx] = {
		...current,
		text: current.text + separator + next.text,
		media_paths: combinedMedia
	};
	sorted.splice(idx + 1, 1);
	return { blocks: normalizeOrder(sorted), cursorPos: joinPoint + separator.length };
}

/**
 * Merge a block with the previous block in order.
 * Returns null if at start, at minimum 2 blocks, or combined media > 4.
 */
export function mergeWithPrevious(
	blocks: ThreadBlock[],
	id: string
): { blocks: ThreadBlock[]; targetId: string; cursorPos: number } | null {
	const sorted = sortBlocks(blocks);
	const idx = sorted.findIndex((b) => b.id === id);
	if (idx <= 0 || sorted.length <= 1) return null;

	const current = sorted[idx];
	const prev = sorted[idx - 1];
	const combinedMedia = [...prev.media_paths, ...current.media_paths];
	if (combinedMedia.length > 4) return null;

	const joinPoint = prev.text.length;
	const separator = prev.text.endsWith('\n') ? '' : '\n';
	sorted[idx - 1] = {
		...prev,
		text: prev.text + separator + current.text,
		media_paths: combinedMedia
	};
	sorted.splice(idx, 1);
	return {
		blocks: normalizeOrder(sorted),
		targetId: prev.id,
		cursorPos: joinPoint + separator.length
	};
}

/**
 * Validate a thread for submission readiness.
 */
export function validateThread(
	blocks: ThreadBlock[]
): { valid: boolean; errors: string[] } {
	const errors: string[] = [];
	const sorted = sortBlocks(blocks);
	const nonEmpty = blocks.filter((b) => b.text.trim().length > 0);

	// "Needs 2 posts" is an incomplete state, not an error — the disabled
	// submit button already communicates this. Only surface actionable errors.

	for (const block of blocks) {
		if (tweetWeightedLen(block.text) > MAX_TWEET_CHARS) {
			const idx = sorted.findIndex((b) => b.id === block.id);
			errors.push(`Post ${idx + 1} exceeds ${MAX_TWEET_CHARS} characters.`);
		}
		if (block.media_paths.length > 4) {
			const idx = sorted.findIndex((b) => b.id === block.id);
			errors.push(`Post ${idx + 1} has too many media (max 4).`);
		}
	}

	return { valid: nonEmpty.length >= 2 && errors.length === 0, errors };
}
