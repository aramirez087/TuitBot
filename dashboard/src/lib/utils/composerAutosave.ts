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

// ---------------------------------------------------------------------------
// Draft-scoped autosave (Draft Studio)
// ---------------------------------------------------------------------------

const DRAFT_KEY_PREFIX = 'tuitbot:compose:draft:';
const LOCAL_DEBOUNCE_MS = 500;
const SERVER_DEBOUNCE_MS = 1500;

export type SyncStatus = 'saved' | 'saving' | 'unsaved' | 'offline' | 'conflict';

/** Read draft-scoped crash recovery data from localStorage. */
export function readDraftAutoSave(draftId: number): AutosavePayload | null {
	try {
		const raw = localStorage.getItem(DRAFT_KEY_PREFIX + draftId);
		if (!raw) return null;
		const data: AutosavePayload = JSON.parse(raw);
		if (Date.now() - data.timestamp > AUTOSAVE_TTL_MS) {
			localStorage.removeItem(DRAFT_KEY_PREFIX + draftId);
			return null;
		}
		return data;
	} catch {
		localStorage.removeItem(DRAFT_KEY_PREFIX + draftId);
		return null;
	}
}

/** Clear draft-scoped crash recovery data. */
export function clearDraftAutoSave(draftId: number): void {
	try {
		localStorage.removeItem(DRAFT_KEY_PREFIX + draftId);
	} catch {
		/* quota */
	}
}

function serializeContent(
	mode: 'tweet' | 'thread',
	tweetText: string,
	blocks: ThreadBlock[]
): { content: string; content_type: string } {
	if (mode === 'thread') {
		const validBlocks = blocks
			.sort((a, b) => a.order - b.order)
			.filter((b) => b.text.trim().length > 0);
		return {
			content: JSON.stringify(validBlocks.map((b) => b.text)),
			content_type: 'thread'
		};
	}
	return { content: tweetText.trim(), content_type: 'tweet' };
}

/**
 * Manages debounced server autosave and local crash recovery for a single draft.
 * Create a new instance when selecting a draft; destroy it when switching away.
 */
export class DraftSaveManager {
	private draftId: number;
	private lastServerUpdatedAt: string;
	private localTimer: ReturnType<typeof setTimeout> | null = null;
	private serverTimer: ReturnType<typeof setTimeout> | null = null;
	private onSyncStatus: (status: SyncStatus) => void;
	private destroyed = false;
	private pendingContent: { content: string; content_type: string } | null = null;
	private saving = false;

	constructor(
		draftId: number,
		serverUpdatedAt: string,
		onSyncStatus: (s: SyncStatus) => void
	) {
		this.draftId = draftId;
		this.lastServerUpdatedAt = serverUpdatedAt;
		this.onSyncStatus = onSyncStatus;
	}

	/** Called on every content change — debounces local and server saves. */
	save(
		mode: 'tweet' | 'thread',
		tweetText: string,
		blocks: ThreadBlock[],
		attachedMedia: AttachedMedia[]
	): void {
		if (this.destroyed) return;

		// Debounce localStorage (crash recovery)
		if (this.localTimer) clearTimeout(this.localTimer);
		this.localTimer = setTimeout(() => {
			if (this.destroyed) return;
			const tweetMedia = attachedMedia.map((m) => ({
				path: m.path,
				mediaType: m.mediaType,
				altText: m.altText
			}));
			const payload: AutosavePayload = {
				mode, tweetText, blocks, tweetMedia, timestamp: Date.now()
			};
			try {
				localStorage.setItem(
					DRAFT_KEY_PREFIX + this.draftId,
					JSON.stringify(payload)
				);
			} catch {
				/* quota */
			}
		}, LOCAL_DEBOUNCE_MS);

		// Debounce server PATCH
		this.pendingContent = serializeContent(mode, tweetText, blocks);
		this.onSyncStatus('unsaved');
		if (this.serverTimer) clearTimeout(this.serverTimer);
		this.serverTimer = setTimeout(() => {
			this.flushServer();
		}, SERVER_DEBOUNCE_MS);
	}

	/** Flush any pending server save immediately. */
	async flush(): Promise<void> {
		if (this.localTimer) { clearTimeout(this.localTimer); this.localTimer = null; }
		if (this.serverTimer) { clearTimeout(this.serverTimer); this.serverTimer = null; }
		if (this.pendingContent && !this.saving) {
			await this.flushServer();
		}
	}

	/** Tear down timers, flush pending save. */
	async destroy(): Promise<void> {
		this.destroyed = true;
		if (this.localTimer) { clearTimeout(this.localTimer); this.localTimer = null; }
		if (this.serverTimer) { clearTimeout(this.serverTimer); this.serverTimer = null; }
		if (this.pendingContent && !this.saving) {
			await this.flushServer();
		}
		clearDraftAutoSave(this.draftId);
	}

	/** Force-save with conflict override (after user chooses "use mine"). */
	async forceServerSave(
		content: string,
		contentType: string,
		newUpdatedAt: string
	): Promise<boolean> {
		this.lastServerUpdatedAt = newUpdatedAt;
		this.pendingContent = { content, content_type: contentType };
		return this.flushServer();
	}

	getLastServerUpdatedAt(): string {
		return this.lastServerUpdatedAt;
	}

	private async flushServer(): Promise<boolean> {
		if (!this.pendingContent) return true;
		const payload = this.pendingContent;
		this.pendingContent = null;
		this.saving = true;

		if (!this.destroyed) this.onSyncStatus('saving');

		try {
			const result = await api.draftStudio.autosave(this.draftId, {
				content: payload.content,
				content_type: payload.content_type,
				updated_at: this.lastServerUpdatedAt
			});
			this.lastServerUpdatedAt = result.updated_at;
			if (!this.destroyed) this.onSyncStatus('saved');
			this.saving = false;
			return true;
		} catch (e) {
			this.saving = false;
			if (this.destroyed) return false;
			if (e instanceof Error && e.message === 'stale_write') {
				this.onSyncStatus('conflict');
			} else {
				this.onSyncStatus('offline');
				// Re-queue for retry on next edit
				if (!this.pendingContent) this.pendingContent = payload;
			}
			return false;
		}
	}
}
