<script lang="ts">
	import { onMount } from 'svelte';
	import { Search, MessageCircle, Loader2, ExternalLink, Sparkles } from 'lucide-svelte';
	import { api } from '$lib/api';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';

	interface DiscoveredTweet {
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

	let tweets = $state<DiscoveredTweet[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let minScore = $state(50);
	let limit = $state(20);

	let composingId = $state<string | null>(null);
	let composedReply = $state('');
	let generating = $state(false);
	let queueing = $state(false);

	async function loadFeed() {
		loading = true;
		error = null;
		try {
			tweets = await api.discovery.feed(minScore, limit);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load discovery feed';
		} finally {
			loading = false;
		}
	}

	async function composeReply(tweetId: string) {
		composingId = tweetId;
		composedReply = '';
		generating = true;
		try {
			const result = await api.discovery.composeReply(tweetId);
			composedReply = result.content;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to generate reply — is LLM configured?';
			composingId = null;
		} finally {
			generating = false;
		}
	}

	async function queueReply(tweetId: string) {
		if (!composedReply.trim()) return;
		queueing = true;
		try {
			await api.discovery.queueReply(tweetId, composedReply.trim());
			composingId = null;
			composedReply = '';
			await loadFeed();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to queue reply';
		} finally {
			queueing = false;
		}
	}

	function scoreColor(score: number): string {
		if (score >= 80) return 'var(--color-success)';
		if (score >= 60) return 'var(--color-accent)';
		if (score >= 40) return 'var(--color-warning)';
		return 'var(--color-text-subtle)';
	}

	onMount(loadFeed);
</script>

<svelte:head>
	<title>Discovery Feed — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<h1>Discovery Feed</h1>
	<div class="controls">
		<label class="score-control">
			<span class="control-label">Min score</span>
			<input
				type="number"
				class="score-input"
				bind:value={minScore}
				min="0"
				max="100"
				step="5"
			/>
		</label>
		<label class="score-control">
			<span class="control-label">Limit</span>
			<input
				type="number"
				class="score-input"
				bind:value={limit}
				min="5"
				max="100"
				step="5"
			/>
		</label>
		<button class="refresh-btn" onclick={loadFeed} disabled={loading}>
			<Search size={14} />
			Refresh
		</button>
	</div>
</div>

{#if loading && tweets.length === 0}
	<div class="loading-state">
		<Loader2 size={20} class="spinner" />
		<span>Discovering tweets...</span>
	</div>
{:else if error && tweets.length === 0}
	<ErrorState message={error} onretry={loadFeed} />
{:else if tweets.length === 0}
	<EmptyState
		title="No tweets discovered"
		description="Tuitbot hasn't found relevant conversations yet. Make sure discovery is running and keywords are configured."
	/>
{:else}
	<div class="tweet-list">
		{#each tweets as tweet (tweet.id)}
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

				{#if composingId === tweet.id}
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
								<button class="cancel-btn" onclick={() => { composingId = null; composedReply = ''; }}>Cancel</button>
								<button class="queue-btn" onclick={() => queueReply(tweet.id)} disabled={!composedReply.trim() || queueing}>
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
							<button class="reply-btn" onclick={() => composeReply(tweet.id)}>
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
		{/each}
	</div>
{/if}

<style>
	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 20px;
		flex-wrap: wrap;
		gap: 12px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.score-control {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.control-label {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.score-input {
		width: 64px;
		padding: 6px 8px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 13px;
		text-align: center;
	}

	.refresh-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.refresh-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
	}

	.refresh-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.tweet-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

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

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 60px 20px;
		color: var(--color-text-muted);
		font-size: 13px;
	}

	.loading-state :global(.spinner) {
		animation: spin 1s linear infinite;
	}

	:global(.spinner) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
