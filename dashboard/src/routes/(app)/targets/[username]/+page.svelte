<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { ArrowLeft, UserCheck, TrendingUp, Clock, Star } from 'lucide-svelte';
	import InteractionTimeline from '$lib/components/InteractionTimeline.svelte';
	import { api, type TargetTimelineItem, type TargetStats } from '$lib/api';

	const username = $derived($page.params.username ?? '');

	let timeline = $state<TargetTimelineItem[]>([]);
	let stats = $state<TargetStats | null>(null);
	let loadingTimeline = $state(true);
	let loadingStats = $state(true);
	let errorMsg = $state<string | null>(null);

	async function loadData(user: string) {
		loadingTimeline = true;
		loadingStats = true;
		errorMsg = null;

		try {
			const [timelineData, statsData] = await Promise.all([
				api.targets.timeline(user),
				api.targets.stats(user)
			]);
			timeline = timelineData;
			stats = statsData;
		} catch (e) {
			errorMsg = e instanceof Error ? e.message : 'Failed to load target data';
		} finally {
			loadingTimeline = false;
			loadingStats = false;
		}
	}

	function formatDate(iso: string | null): string {
		if (!iso) return '—';
		return new Date(iso).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatFrequency(days: number | null): string {
		if (days === null) return '—';
		if (days < 1) return 'multiple per day';
		if (days < 2) return 'daily';
		return `every ${days.toFixed(1)} days`;
	}

	onMount(() => {
		loadData(username);
	});
</script>

<svelte:head>
	<title>@{username} — Targets — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<a href="/targets" class="back-link">
		<ArrowLeft size={16} />
		Back to Targets
	</a>

	<div class="target-header">
		<h1>@{username}</h1>
	</div>

	{#if stats}
		<div class="stats-row">
			<div class="stat-chip">
				<UserCheck size={14} />
				<span>{stats.total_replies} interactions</span>
			</div>
			<div class="stat-chip">
				<TrendingUp size={14} />
				<span>avg score: {stats.avg_score.toFixed(1)}</span>
			</div>
			{#if stats.first_interaction}
				<div class="stat-chip">
					<Clock size={14} />
					<span>since {formatDate(stats.first_interaction)}</span>
				</div>
			{/if}
			<div class="stat-chip">
				<Star size={14} />
				<span>frequency: {formatFrequency(stats.interaction_frequency_days)}</span>
			</div>
		</div>
	{:else if loadingStats}
		<div class="stats-row">
			{#each { length: 4 } as _}
				<div class="stat-chip skeleton"></div>
			{/each}
		</div>
	{/if}

	{#if stats?.best_reply_content}
		<div class="best-reply">
			<div class="best-reply-label">
				<Star size={12} />
				Best reply ({stats.best_reply_score?.toFixed(0)} pts)
			</div>
			<p class="best-reply-text">{stats.best_reply_content}</p>
		</div>
	{/if}
</div>

{#if errorMsg}
	<div class="error-banner">
		<span>{errorMsg}</span>
		<button onclick={() => loadData(username)}>Retry</button>
	</div>
{/if}

<div class="timeline-section">
	<div class="section-header">
		<h2>Interaction Timeline</h2>
	</div>
	<div class="timeline-container">
		<InteractionTimeline items={timeline} loading={loadingTimeline} />
	</div>
</div>

<style>
	.page-header {
		margin-bottom: 24px;
	}

	.back-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: var(--color-text-muted);
		text-decoration: none;
		margin-bottom: 16px;
		transition: color 0.15s ease;
	}

	.back-link:hover {
		color: var(--color-accent);
	}

	.target-header {
		margin-bottom: 12px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.stats-row {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-bottom: 16px;
	}

	.stat-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border-radius: 6px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.stat-chip.skeleton {
		width: 120px;
		height: 28px;
		background-color: var(--color-surface-active);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.best-reply {
		padding: 12px 16px;
		border-radius: 8px;
		background-color: color-mix(in srgb, var(--color-accent) 5%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-accent) 15%, var(--color-border-subtle));
		margin-bottom: 0;
	}

	.best-reply-label {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
		margin-bottom: 6px;
	}

	.best-reply-text {
		margin: 0;
		font-size: 13px;
		color: var(--color-text);
		line-height: 1.5;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid var(--color-danger);
		border-radius: 8px;
		margin-bottom: 20px;
		color: var(--color-danger);
		font-size: 13px;
	}

	.error-banner button {
		padding: 4px 12px;
		border: 1px solid var(--color-danger);
		border-radius: 4px;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
	}

	.error-banner button:hover {
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.timeline-section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		overflow: hidden;
	}

	.section-header {
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.section-header h2 {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.timeline-container {
		background-color: var(--color-base);
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

	@media (max-width: 640px) {
		.stats-row {
			flex-direction: column;
		}
	}
</style>
