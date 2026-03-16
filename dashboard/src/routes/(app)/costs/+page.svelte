<script lang="ts">
	import { onMount } from 'svelte';
	import { loading, error, loadCosts } from '$lib/stores/costs';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import CostsLlmTab from './CostsLlmTab.svelte';
	import CostsXApiTab from './CostsXApiTab.svelte';

	let days = $state(30);
	let activeTab = $state<'llm' | 'xapi'>('llm');

	function handleDaysChange(newDays: number) {
		days = newDays;
		loadCosts(newDays);
	}

	onMount(() => {
		loadCosts(days);
		const handler = () => loadCosts(days);
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});
</script>

<svelte:head>
	<title>Costs — Tuitbot</title>
</svelte:head>

<div class="page">
	<div class="page-header">
		<h1>Costs</h1>
		<div class="header-controls">
			<div class="tab-switcher">
				<button
					class="tab-btn"
					class:active={activeTab === 'llm'}
					onclick={() => (activeTab = 'llm')}
				>LLM</button>
				<button
					class="tab-btn"
					class:active={activeTab === 'xapi'}
					onclick={() => (activeTab = 'xapi')}
				>X API</button>
			</div>
			<div class="period-selector">
				<button
					class="period-btn"
					class:active={days === 7}
					onclick={() => handleDaysChange(7)}
				>7d</button>
				<button
					class="period-btn"
					class:active={days === 30}
					onclick={() => handleDaysChange(30)}
				>30d</button>
				<button
					class="period-btn"
					class:active={days === 90}
					onclick={() => handleDaysChange(90)}
				>90d</button>
			</div>
		</div>
	</div>

	{#if $error}
		<ErrorState message={$error} onretry={() => loadCosts(days)} />
	{:else if $loading}
		<div class="loading">Loading cost data...</div>
	{:else if activeTab === 'llm'}
		<CostsLlmTab />
	{:else}
		<CostsXApiTab />
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
		align-items: center;
		justify-content: space-between;
	}

	.header-controls {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	h1 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
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
			align-items: flex-start;
			gap: 12px;
		}
	}
</style>
