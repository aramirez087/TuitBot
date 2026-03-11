/**
 * Pure helper functions for compose operations.
 * Extracted from ComposeWorkspace to keep the orchestrator focused on reactive state.
 */

import type { ComposeRequest, ThreadBlock } from '$lib/api';
import type { AttachedMedia } from '$lib/components/composer/TweetEditor.svelte';
import { buildScheduledFor } from './timezone';

export interface BuildComposeRequestOpts {
	mode: 'tweet' | 'thread';
	tweetText: string;
	threadBlocks: ThreadBlock[];
	selectedTime: string | null;
	targetDate: Date;
	attachedMedia: AttachedMedia[];
	/** IANA timezone from ScheduleConfig (e.g. "America/New_York"). Falls back to "UTC". */
	timezone?: string;
}

/** Build a ComposeRequest from current editor state. */
export function buildComposeRequest(opts: BuildComposeRequestOpts): ComposeRequest {
	const { mode, tweetText, threadBlocks, selectedTime, targetDate, attachedMedia, timezone } =
		opts;
	const data: ComposeRequest = { content_type: mode, content: '' };

	if (mode === 'tweet') {
		data.content = tweetText.trim();
	} else {
		const validBlocks = threadBlocks
			.filter((b) => b.text.trim().length > 0)
			.sort((a, b) => a.order - b.order)
			.map((b, i) => ({ ...b, text: b.text.trim(), order: i }));
		data.content = JSON.stringify(validBlocks.map((b) => b.text));
		data.blocks = validBlocks;
		const allMedia = validBlocks.flatMap((b) => b.media_paths);
		if (allMedia.length > 0) data.media_paths = allMedia;
	}

	if (selectedTime) {
		const tz = timezone || 'UTC';
		// Extract YYYY-MM-DD from the target date in a timezone-safe way
		const year = targetDate.getFullYear();
		const month = String(targetDate.getMonth() + 1).padStart(2, '0');
		const day = String(targetDate.getDate()).padStart(2, '0');
		const dateStr = `${year}-${month}-${day}`;
		data.scheduled_for = buildScheduledFor(dateStr, selectedTime, tz);
	}

	if (attachedMedia.length > 0) data.media_paths = attachedMedia.map((m) => m.path);

	return data;
}

/** Append a voice cue prefix to a topic string for AI calls. */
export function topicWithCue(voiceCue: string, topic: string): string {
	return voiceCue ? `[Tone: ${voiceCue}] ${topic}` : topic;
}
