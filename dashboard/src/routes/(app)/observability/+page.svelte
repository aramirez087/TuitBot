<script lang="ts">
	import { onMount } from 'svelte';
	import { Activity, AlertTriangle, DollarSign, Shield, Loader2 } from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import RateLimitBar from '$lib/components/RateLimitBar.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import {
		runtimeStatus,
		approvalStats,
		rateLimits,
		llmCosts,
		xApiCosts,
		mcpSummary,
		recentErrors,
		errorsToday,
		loading,
		error,
		loadAll
	} from '$lib/stores/observability';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

	function formatCost(val: number): string {
		return '$' + val.toFixed(2);
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

	onMount(() => {
		loadAll();
		const handler = () => loadAll();
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});
</script>

<svelte:head>
	<title>Observability — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<h1>System Health</h1>
	<p class="subtitle">Aggregated observability dashboard</p>
</div>

{#if $loading}
	<div class="loading-state">
		<Loader2 size={20} class="spinner" />
		<span>Loading system health data...</span>
	</div>
{:else if $error}
	<ErrorState message={$error} onretry={loadAll} />
{:else}
	<!-- Section 1: System Status -->
	<section class="section">
		<h2>System Status</h2>
		<div class="stat-grid">
			<StatCard
				label="Runtime"
				value={$runtimeStatus ? $runtimeStatus.mode : 'Unknown'}
			>
				{#snippet icon()}<Activity size={18} />{/snippet}
			</StatCard>
			<StatCard
				label="Pending Approvals"
				value={$approvalStats?.pending ?? 0}
			/>
			<StatCard
				label="Errors Today"
				value={$errorsToday}
			>
				{#snippet icon()}<AlertTriangle size={18} />{/snippet}
			</StatCard>
		</div>
	</section>

	<!-- Section 2: Cost Summary -->
	<section class="section">
		<h2>Cost Summary</h2>
		<div class="stat-grid">
			<StatCard
				label="LLM Cost Today"
				value={formatCost($llmCosts?.cost_today ?? 0)}
			>
				{#snippet icon()}<DollarSign size={18} />{/snippet}
			</StatCard>
			<StatCard
				label="LLM Cost 7d"
				value={formatCost($llmCosts?.cost_7d ?? 0)}
			/>
			<StatCard
				label="X API Cost Today"
				value={formatCost($xApiCosts?.cost_today ?? 0)}
			/>
			<StatCard
				label="X API Cost 7d"
				value={formatCost($xApiCosts?.cost_7d ?? 0)}
			/>
		</div>
	</section>

	<!-- Section 3: Rate Limits -->
	{#if $rateLimits}
		<section class="section">
			<h2>Rate Limits</h2>
			<div class="rate-limits-grid">
				<RateLimitBar label="Replies" usage={$rateLimits.replies} />
				<RateLimitBar label="Tweets" usage={$rateLimits.tweets} />
				<RateLimitBar label="Threads" usage={$rateLimits.threads} />
			</div>
		</section>
	{/if}

	<!-- Section 4: MCP Governance -->
	{#if $mcpSummary}
		<section class="section">
			<h2>MCP Governance</h2>
			<div class="stat-grid">
				<StatCard
					label="Total Calls (24h)"
					value={$mcpSummary.total_calls}
				>
					{#snippet icon()}<Shield size={18} />{/snippet}
				</StatCard>
				<StatCard
					label="Success Rate"
					value={($mcpSummary.overall_success_rate * 100).toFixed(1) + '%'}
				/>
				<StatCard
					label="Unique Tools"
					value={$mcpSummary.unique_tools}
				/>
			</div>
			{#if Object.keys($mcpSummary.policy_decisions).length > 0}
				<div class="policy-decisions">
					<h3>Policy Decisions</h3>
					<div class="decision-chips">
						{#each Object.entries($mcpSummary.policy_decisions) as [decision, count]}
							<span class="decision-chip">
								{decision}: <strong>{count}</strong>
							</span>
						{/each}
					</div>
				</div>
			{/if}
		</section>
	{/if}

	<!-- Section 5: Recent Errors -->
	<section class="section">
		<h2>Recent Errors</h2>
		{#if $recentErrors.length === 0}
			<p class="muted">No recent errors.</p>
		{:else}
			<div class="error-list">
				{#each $recentErrors as err}
					<div class="error-item">
						<span class="error-type">{err.action_type}</span>
						<span class="error-message">{err.message ?? 'No details'}</span>
						<span class="error-time">{relativeTime(err.created_at)}</span>
					</div>
				{/each}
			</div>
		{/if}
	</section>
{/if}

<style>
	.page-header {
		margin-bottom: 24px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	h2 {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 12px;
	}

	h3 {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-muted);
		margin: 12px 0 8px;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.muted {
		font-size: 13px;
		color: var(--color-text-subtle);
		margin: 0;
	}

	.section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 20px;
		margin-bottom: 16px;
	}

	.stat-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 12px;
	}

	.rate-limits-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 20px;
	}

	.policy-decisions {
		margin-top: 8px;
	}

	.decision-chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.decision-chip {
		font-size: 12px;
		padding: 4px 10px;
		border-radius: 4px;
		background-color: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}

	.error-list {
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		overflow: hidden;
	}

	.error-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 14px;
		border-bottom: 1px solid var(--color-border-subtle);
		font-size: 13px;
	}

	.error-item:last-child {
		border-bottom: none;
	}

	.error-type {
		font-weight: 600;
		color: var(--color-danger);
		white-space: nowrap;
		min-width: 80px;
	}

	.error-message {
		flex: 1;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.error-time {
		color: var(--color-text-subtle);
		font-size: 11px;
		white-space: nowrap;
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

	:global(.spinner) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	@media (max-width: 640px) {
		.rate-limits-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
