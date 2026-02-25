<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import {
		loading,
		error,
		loadMcpData,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/mcp';

	import OverviewSection from './OverviewSection.svelte';
	import PolicySection from './PolicySection.svelte';
	import ToolsSection from './ToolsSection.svelte';
	import ErrorsSection from './ErrorsSection.svelte';
	import ExecutionsSection from './ExecutionsSection.svelte';

	let hours = $state(24);
	let activeTab = $state<'overview' | 'policy' | 'tools' | 'errors' | 'executions'>('overview');

	onMount(() => {
		loadMcpData(hours);
		startAutoRefresh(30_000, hours);
	});

	onDestroy(() => {
		stopAutoRefresh();
	});

	function handleHoursChange(newHours: number) {
		hours = newHours;
		loadMcpData(hours);
		stopAutoRefresh();
		startAutoRefresh(30_000, hours);
	}

	function formatRate(rate: number): string {
		return (rate * 100).toFixed(1) + '%';
	}

	function formatLatency(ms: number): string {
		if (ms < 1) return '<1ms';
		if (ms >= 1000) return (ms / 1000).toFixed(1) + 's';
		return Math.round(ms) + 'ms';
	}

	function formatTime(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
		} catch {
			return iso;
		}
	}

	function formatDate(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
		} catch {
			return iso;
		}
	}
</script>

<div class="page">
	<div class="page-header">
		<div class="page-title">
			<h1>MCP Governance</h1>
			<span class="subtitle">Policy, telemetry, and tool execution insights</span>
		</div>
		<div class="header-controls">
			<div class="tab-switcher">
				<button
					class="tab-btn"
					class:active={activeTab === 'overview'}
					onclick={() => (activeTab = 'overview')}>Overview</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'policy'}
					onclick={() => (activeTab = 'policy')}>Policy</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'tools'}
					onclick={() => (activeTab = 'tools')}>Tools</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'errors'}
					onclick={() => (activeTab = 'errors')}>Errors</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'executions'}
					onclick={() => (activeTab = 'executions')}>Executions</button
				>
			</div>
			<div class="period-selector">
				<button
					class="period-btn"
					class:active={hours === 1}
					onclick={() => handleHoursChange(1)}>1h</button
				>
				<button
					class="period-btn"
					class:active={hours === 24}
					onclick={() => handleHoursChange(24)}>24h</button
				>
				<button
					class="period-btn"
					class:active={hours === 168}
					onclick={() => handleHoursChange(168)}>7d</button
				>
			</div>
		</div>
	</div>

	{#if $error}
		<ErrorState message={$error} onretry={() => loadMcpData(hours)} />
	{:else if $loading}
		<div class="loading">Loading MCP data...</div>
	{:else if activeTab === 'overview'}
		<OverviewSection {formatRate} {formatLatency} />
	{:else if activeTab === 'policy'}
		<PolicySection />
	{:else if activeTab === 'tools'}
		<ToolsSection {formatRate} {formatLatency} />
	{:else if activeTab === 'errors'}
		<ErrorsSection {formatTime} {formatDate} />
	{:else if activeTab === 'executions'}
		<ExecutionsSection {formatTime} {formatDate} {formatLatency} />
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 20px;
		max-width: 1000px;
	}

	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 16px;
	}

	.page-title {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	h1 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.header-controls {
		display: flex;
		gap: 8px;
		align-items: center;
		flex-shrink: 0;
	}

	.tab-switcher {
		display: flex;
		gap: 4px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 3px;
	}

	.tab-btn {
		padding: 4px 14px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.tab-btn.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.tab-btn:hover:not(.active) {
		color: var(--color-text);
	}

	.period-selector {
		display: flex;
		gap: 4px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 3px;
	}

	.period-btn {
		padding: 4px 12px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.period-btn.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.period-btn:hover:not(.active) {
		color: var(--color-text);
	}

	.loading {
		text-align: center;
		padding: 48px 0;
		color: var(--color-text-muted);
		font-size: 14px;
	}

	@media (max-width: 800px) {
		.page-header {
			flex-direction: column;
			gap: 12px;
		}

		.header-controls {
			flex-wrap: wrap;
		}
	}
</style>
