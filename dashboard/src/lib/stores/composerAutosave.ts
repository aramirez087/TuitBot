import type { ThreadBlock } from '$lib/api';

/**
 * RecoveryData represents the composer state snapshot for autosave/recovery.
 * Used to restore composer session after interruptions (e.g., tab reload, app crash).
 * Captures: editing mode, tweet text, thread blocks, timestamps, and attached media.
 */
export interface RecoveryData {
	mode: string;
	tweetText: string;
	blocks: ThreadBlock[];
	timestamp: number;
	tweetMedia?: Array<{ path: string; mediaType: string; altText?: string }>;
}
