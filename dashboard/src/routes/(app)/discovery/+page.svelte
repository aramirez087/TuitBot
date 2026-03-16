<script lang="ts">
	import { onMount } from 'svelte';
	import { Loader2 } from 'lucide-svelte';
	import { api } from '$lib/api';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';
	import DiscoveryControls from './DiscoveryControls.svelte';
	import DiscoveryTweetCard, { type DiscoveredTweet } from './DiscoveryTweetCard.svelte';

	let tweets = $state<DiscoveredTweet[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let minScore = $state(50);
	let maxScore = $state<number | undefined>(undefined);
	let keyword = $state('');
	let keywords = $state<string[]>([]);
	let limit = $state(20);

	let composingId = $state<string | null>(null);
	let composedReply = $state('');
	let generating = $state(false);
	let queueing = $state(false);

	async function loadFeed() {
		loading = true;
		error = null;
		try {
			tweets = await api.discovery.feed({
				minScore,
				maxScore,
				keyword: keyword || undefined,
				limit,
			});
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load discovery feed';
		} finally {
			loading = false;
		}
	}

	async function loadKeywords() {
		try {
			keywords = await api.discovery.keywords();
		} catch {
			// Non-critical — silently ignore.
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

	onMount(() => {
		loadFeed();
		loadKeywords();
		const handler = () => {
			loadFeed();
			loadKeywords();
		};
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});
</script>

<svelte:head>
	<title>Discovery Feed — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<h1>Discovery Feed</h1>
	<DiscoveryControls
		bind:minScore
		bind:maxScore
		bind:keyword
		bind:limit
		{keywords}
		{loading}
		onRefresh={loadFeed}
	/>
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
			<DiscoveryTweetCard
				{tweet}
				{composingId}
				bind:composedReply
				{generating}
				{queueing}
				onComposeReply={composeReply}
				onQueueReply={queueReply}
				onCancelCompose={() => {
					composingId = null;
					composedReply = '';
				}}
			/>
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

	.tweet-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
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

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
