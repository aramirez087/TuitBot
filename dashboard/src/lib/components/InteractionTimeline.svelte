<script lang="ts">
	import { MessageSquare, SkipForward } from 'lucide-svelte';
	import type { TargetTimelineItem } from '$lib/api';

	interface Props {
		items: TargetTimelineItem[];
		loading: boolean;
	}

	let { items, loading }: Props = $props();

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	}

	function relativeTime(iso: string): string {
		const diff = Date.now() - new Date(iso).getTime();
		const mins = Math.floor(diff / 60_000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ago`;
	}
</script>

<div class="timeline">
	{#if loading}
		{#each { length: 4 } as _}
			<div class="skeleton-item"></div>
		{/each}
	{:else if items.length === 0}
		<div class="empty-state">
			<div class="empty-icon">
				<MessageSquare size={28} />
			</div>
			<p class="empty-title">No interactions yet</p>
			<p class="empty-hint">Tweets from this target and your replies will appear here.</p>
		</div>
	{:else}
		{#each items as item (item.tweet_id)}
			<div class="timeline-item" class:replied={item.replied_to} class:skipped={!item.replied_to}>
				<div class="timeline-marker">
					{#if item.replied_to}
						<MessageSquare size={14} />
					{:else}
						<SkipForward size={14} />
					{/if}
				</div>

				<div class="timeline-content">
					<div class="timeline-header">
						<span class="timeline-date">{formatDate(item.posted_at)}</span>
						<span class="timeline-ago">{relativeTime(item.posted_at)}</span>
						<span class="timeline-score">{Math.round(item.relevance_score)} pts</span>
					</div>

					<div class="tweet-text">{item.text}</div>

					<div class="tweet-stats">
						<span class="tweet-stat">&#x2665; {item.tweet_like_count}</span>
						<span class="tweet-stat">&#x1f4ac; {item.tweet_reply_count}</span>
					</div>

					{#if item.replied_to && item.reply_content}
						<div class="reply-block">
							<div class="reply-label">Your reply</div>
							<div class="reply-text">{item.reply_content}</div>
							{#if item.reply_created_at}
								<div class="reply-time">{relativeTime(item.reply_created_at)}</div>
							{/if}
						</div>
					{:else if !item.replied_to}
						<div class="skipped-label">Not replied</div>
					{/if}
				</div>
			</div>
		{/each}
	{/if}
</div>

<style>
	.timeline {
		display: flex;
		flex-direction: column;
	}

	.timeline-item {
		display: flex;
		gap: 12px;
		padding: 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		border-left: 3px solid transparent;
		transition: background-color 0.1s ease;
	}

	.timeline-item:last-child {
		border-bottom: none;
	}

	.timeline-item:hover {
		background-color: var(--color-surface-hover);
	}

	.timeline-item.replied {
		border-left-color: var(--color-success);
	}

	.timeline-item.skipped {
		border-left-color: var(--color-text-subtle);
		opacity: 0.75;
	}

	.timeline-marker {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 2px;
	}

	.replied .timeline-marker {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.skipped .timeline-marker {
		background-color: color-mix(in srgb, var(--color-text-subtle) 15%, transparent);
		color: var(--color-text-subtle);
	}

	.timeline-content {
		flex: 1;
		min-width: 0;
	}

	.timeline-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
	}

	.timeline-date {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
	}

	.timeline-ago {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.timeline-score {
		margin-left: auto;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
		font-variant-numeric: tabular-nums;
	}

	.tweet-text {
		font-size: 13px;
		color: var(--color-text);
		line-height: 1.5;
		margin-bottom: 6px;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.tweet-stats {
		display: flex;
		gap: 12px;
		margin-bottom: 8px;
	}

	.tweet-stat {
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.reply-block {
		padding: 10px 12px;
		border-radius: 6px;
		background-color: color-mix(in srgb, var(--color-success) 5%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-success) 15%, var(--color-border-subtle));
	}

	.reply-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-success);
		margin-bottom: 4px;
	}

	.reply-text {
		font-size: 13px;
		color: var(--color-text);
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.reply-time {
		margin-top: 4px;
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.skipped-label {
		font-size: 11px;
		font-weight: 500;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	.empty-state {
		padding: 48px 20px;
		text-align: center;
	}

	.empty-icon {
		color: var(--color-text-subtle);
		margin-bottom: 12px;
		opacity: 0.5;
	}

	.empty-title {
		margin: 0 0 6px;
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.empty-hint {
		margin: 0;
		font-size: 13px;
		color: var(--color-text-subtle);
	}

	.skeleton-item {
		height: 100px;
		border-bottom: 1px solid var(--color-border-subtle);
		background-color: var(--color-surface-active);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-item:last-child {
		border-bottom: none;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.4;
		}
	}
</style>
