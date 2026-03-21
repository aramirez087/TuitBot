/**
 * parseThreadContent.ts — Thread block content parsing utilities
 *
 * WHY here and not in types.ts: These are runtime functions, not type definitions.
 * Moved from api/types.ts during type cleanup.
 */

import type { ThreadBlock, ThreadBlocksPayload } from '$lib/api';

/**
 * Parse stored thread content, detecting new blocks format vs legacy string array.
 * Returns `ThreadBlock[]` for blocks format, `string[]` for legacy format.
 */
export function parseThreadContent(content: string): ThreadBlock[] | string[] {
	try {
		const parsed = JSON.parse(content);
		if (parsed && typeof parsed === 'object' && !Array.isArray(parsed) && parsed.blocks) {
			return (parsed as ThreadBlocksPayload).blocks;
		}
		if (Array.isArray(parsed)) {
			return parsed as string[];
		}
	} catch {
		// Not JSON — return as single-item array
	}
	return [content];
}

/**
 * Check whether stored content uses the versioned blocks payload format.
 */
export function isBlocksPayload(content: string): boolean {
	try {
		const parsed = JSON.parse(content);
		return parsed && typeof parsed === 'object' && !Array.isArray(parsed) && 'blocks' in parsed;
	} catch {
		return false;
	}
}
