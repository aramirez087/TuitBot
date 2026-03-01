/**
 * Pure helper functions for compose operations.
 * Extracted from ComposeWorkspace to keep the orchestrator focused on reactive state.
 */

import type { ComposeRequest, ThreadBlock } from '$lib/api';
import type { AttachedMedia } from '$lib/components/composer/TweetEditor.svelte';

export interface BuildComposeRequestOpts {
	mode: 'tweet' | 'thread';
	tweetText: string;
	threadBlocks: ThreadBlock[];
	selectedTime: string | null;
	targetDate: Date;
	attachedMedia: AttachedMedia[];
}

/** Build a ComposeRequest from current editor state. */
export function buildComposeRequest(opts: BuildComposeRequestOpts): ComposeRequest {
	const { mode, tweetText, threadBlocks, selectedTime, targetDate, attachedMedia } = opts;
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
		const scheduled = new Date(targetDate);
		const [h, m] = selectedTime.split(':').map(Number);
		scheduled.setHours(h, m, 0, 0);
		data.scheduled_for = scheduled.toISOString().replace('Z', '');
	}

	if (attachedMedia.length > 0) data.media_paths = attachedMedia.map((m) => m.path);

	return data;
}

/** Append a voice cue prefix to a topic string for AI calls. */
export function topicWithCue(voiceCue: string, topic: string): string {
	return voiceCue ? `[Tone: ${voiceCue}] ${topic}` : topic;
}
