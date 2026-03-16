<script lang="ts">
	import { Loader2, MessageCircle, ExternalLink, Sparkles } from 'lucide-svelte';

	export interface DiscoveredTweet {
		id: string;
		author_username: string;
		content: string;
		relevance_score: number;
		matched_keyword: string | null;
		like_count: number;
		retweet_count: number;
		reply_count: number;
		replied_to: boolean;
		discovered_at: string;
	}

	interface Props {
		tweet: DiscoveredTweet;
		composingId: string | null;
		composedReply: string;
		generating: boolean;
		queueing: boolean;
		onComposeReply: (tweetId: string) => void;
		onQueueReply: (tweetId: string) => void;
		onCancelCompose: () => void;
	}

	let {
		tweet,
		composingId,
		composedReply = $bindable(),
		generating,
		queueing,
		onComposeReply,
		onQueueReply,
		onCancelCompose,
	}: Props = $props();

	const isComposing = $derived(composingId === tweet.id);

	function scoreColor(score: number): string {
		if (score >= 80) return 'var(--color-success)';
		if (score >= 60) return 'var(--color-accent)';
		if (score >= 40) return 'var(--color-warning)';
		return 'var(--color-text-subtle)';
	}
</script>

<div class="tweet-card" class:replied={tweet.replied_to}>
	<div class="tweet-header">
		<span class="author">@{tweet.author_username}</span>
		<div class="header-meta">
			{#if tweet.matched_keyword}
				<span class="keyword-badge">{tweet.matched_keyword}</span>
			{/if}
			<span class="score-badge" style="color: {scoreColor(tweet.relevance_score ?? 0)}">
				{Math.round(tweet.relevance_score ?? 0)}
			</span>
		</div>
	</div>

	<p class="tweet-content">{tweet.content}</p>

	<div class="tweet-meta">
		<span class="stat">{tweet.like_count} likes</span>
		<span class="stat">{tweet.retweet_count} RTs</span>
		<span class="stat">{tweet.reply_count} replies</span>
		<span class="tweet-date">{new Date(tweet.discovered_at).toLocaleDateString()}</span>
	</div>

	{#if isComposing}
		<div class="reply-compose">
			{#if generating}
				<div class="generating">
					<Loader2 size={14} class="spinner" />
					<span>Generating reply...</span>
				</div>
			{:else}
				<textarea
					class="reply-textarea"
					bind:value={composedReply}
					rows="3"
					placeholder="Edit reply before queuing..."
				></textarea>
				<div class="reply-actions">
					<button class="cancel-btn" onclick={onCancelCompose}>Cancel</button>
					<button
						class="queue-btn"
						onclick={() => onQueueReply(tweet.id)}
						disabled={!composedReply.trim() || queueing}
					>
						<MessageCircle size={14} />
						{queueing ? 'Queuing...' : 'Queue Reply'}
					</button>
				</div>
			{/if}
		</div>
	{:else}
		<div class="tweet-actions">
			{#if tweet.replied_to}
				<span class="replied-badge">Replied</span>
			{:else}
				<button class="reply-btn" onclick={() => onComposeReply(tweet.id)}>
					<Sparkles size={14} />
					Compose Reply
				</button>
			{/if}
			<a
				class="link-btn"
				href="https://x.com/{tweet.author_username}/status/{tweet.id}"
				target="_blank"
				rel="noopener noreferrer"
			>
				<ExternalLink size={14} />
				View
			</a>
		</div>
	{/if}
</div>

<style>
	.tweet-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 16px;
	}

	.tweet-card.replied {
		opacity: 0.6;
	}

	.tweet-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.author {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-accent);
	}

	.header-meta {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.keyword-badge {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}

	.score-badge {
		font-size: 13px;
		font-weight: 700;
		font-family: var(--font-mono);
	}

	.tweet-content {
		font-size: 13px;
		color: var(--color-text);
		line-height: 1.5;
		margin: 0 0 8px;
		white-space: pre-wrap;
	}

	.tweet-meta {
		display: flex;
		gap: 12px;
		margin-bottom: 12px;
	}

	.stat {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.tweet-date {
		font-size: 11px;
		color: var(--color-text-subtle);
		margin-left: auto;
	}

	.tweet-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.reply-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: transparent;
		color: var(--color-accent);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.reply-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.link-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text-muted);
		font-size: 12px;
		text-decoration: none;
		transition: all 0.15s ease;
	}

	.link-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.replied-badge {
		font-size: 11px;
		color: var(--color-success);
		font-weight: 500;
	}

	.reply-compose {
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.generating {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px;
		color: var(--color-text-muted);
		font-size: 12px;
	}

	.generating :global(.spinner) {
		animation: spin 1s linear infinite;
	}

	.reply-textarea {
		width: 100%;
		padding: 10px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
		resize: vertical;
		line-height: 1.5;
		box-sizing: border-box;
	}

	.reply-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.reply-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}

	.cancel-btn {
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.cancel-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.queue-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.queue-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.queue-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
