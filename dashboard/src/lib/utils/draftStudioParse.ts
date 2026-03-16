import type { ScheduledContentItem, ThreadBlock } from '$lib/api';
import type { AttachedMedia } from '$lib/components/composer/TweetEditor.svelte';

export interface HydrationPayload {
	mode: 'tweet' | 'thread';
	tweetText: string;
	threadBlocks: ThreadBlock[];
	attachedMedia: AttachedMedia[];
	updatedAt: string;
}

export function parseServerDraft(draft: ScheduledContentItem): HydrationPayload {
	if (draft.content_type === 'thread') {
		let texts: string[] = [];
		try {
			const parsed = JSON.parse(draft.content || '[]');
			texts = Array.isArray(parsed)
				? parsed.filter((t): t is string => typeof t === 'string')
				: [];
		} catch {
			texts = draft.content ? [draft.content] : [];
		}
		return {
			mode: 'thread',
			tweetText: '',
			threadBlocks:
				texts.length > 0
					? texts.map((text, i) => ({
							id: crypto.randomUUID(),
							text,
							media_paths: [],
							order: i,
						}))
					: [
							{ id: crypto.randomUUID(), text: '', media_paths: [], order: 0 },
							{ id: crypto.randomUUID(), text: '', media_paths: [], order: 1 },
						],
			attachedMedia: [],
			updatedAt: draft.updated_at,
		};
	}
	return {
		mode: 'tweet',
		tweetText: draft.content || '',
		threadBlocks: [],
		attachedMedia: [],
		updatedAt: draft.updated_at,
	};
}
