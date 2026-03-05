/**
 * Pure helper functions for composer autosave and crash recovery.
 * Extracted from ComposeWorkspace to keep the orchestrator focused on reactive state.
 */

import { api, type ThreadBlock } from '$lib/api';
import type { AttachedMedia } from '$lib/components/composer/TweetEditor.svelte';

export const AUTOSAVE_KEY = 'tuitbot:compose:draft';
export const AUTOSAVE_ACTIVE_KEY = 'tuitbot:compose:active';
export const AUTOSAVE_DEBOUNCE_MS = 500;
export const AUTOSAVE_TTL_MS = 7 * 24 * 60 * 60 * 1000;

export interface AutosavePayload {
	mode: string;
	tweetText: string;
	blocks: ThreadBlock[];
	tweetMedia?: Array<{ path: string; mediaType: string; altText?: string }>;
	timestamp: number;
}

/** Serialize current editor state into localStorage. */
export function saveAutoSave(
	mode: 'tweet' | 'thread',
	tweetText: string,
	blocks: ThreadBlock[],
	attachedMedia: AttachedMedia[]
): void {
	const tweetMedia = attachedMedia.map((m) => ({
		path: m.path,
		mediaType: m.mediaType,
		altText: m.altText
	}));
	const payload: AutosavePayload = { mode, tweetText, blocks, tweetMedia, timestamp: Date.now() };
	try {
		localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(payload));
	} catch {
		/* quota */
	}
}

/** Remove autosave data from localStorage. */
export function clearAutoSave(): void {
	localStorage.removeItem(AUTOSAVE_KEY);
}

/** Read and validate the autosave payload, returning null if expired or empty. */
export function readAutoSave(): AutosavePayload | null {
	try {
		const raw = localStorage.getItem(AUTOSAVE_KEY);
		if (!raw) return null;
		const data: AutosavePayload = JSON.parse(raw);
		if (Date.now() - data.timestamp > AUTOSAVE_TTL_MS) {
			localStorage.removeItem(AUTOSAVE_KEY);
			return null;
		}
		const hasContent =
			data.tweetText?.trim() ||
			data.blocks?.some((b: ThreadBlock) => b.text.trim()) ||
			(data.tweetMedia?.length ?? 0) > 0;
		if (!hasContent) return null;
		return data;
	} catch {
		localStorage.removeItem(AUTOSAVE_KEY);
		return null;
	}
}

/** Reconstruct AttachedMedia from autosave payload. */
export function restoreMedia(
	tweetMedia?: Array<{ path: string; mediaType: string; altText?: string }>
): AttachedMedia[] {
	if (!tweetMedia?.length) return [];
	return tweetMedia.map((m) => ({
		path: m.path,
		previewUrl: api.media.fileUrl(m.path),
		mediaType: m.mediaType,
		altText: m.altText
	}));
}

/** Check whether the previous session was a normal navigation (vs crash). */
export function wasNavigationExit(): boolean {
	try {
		const flag = sessionStorage.getItem(AUTOSAVE_ACTIVE_KEY) === '1';
		sessionStorage.removeItem(AUTOSAVE_ACTIVE_KEY);
		return flag;
	} catch {
		return false;
	}
}

/** Mark this session as active (called on beforeunload and normal teardown). */
export function markSessionActive(): void {
	try {
		sessionStorage.setItem(AUTOSAVE_ACTIVE_KEY, '1');
	} catch {
		/* unavailable */
	}
}

/** Clear the session active flag after successful submission. */
export function clearSessionFlag(): void {
	try {
		sessionStorage.removeItem(AUTOSAVE_ACTIVE_KEY);
	} catch {
		/* unavailable */
	}
}
