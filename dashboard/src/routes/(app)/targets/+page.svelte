<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { Target, Plus } from 'lucide-svelte';
	import TargetCard from '$lib/components/TargetCard.svelte';
	import AddTargetModal from '$lib/components/AddTargetModal.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import {
		targets,
		loading,
		error,
		targetCount,
		repliesToday,
		loadTargets,
		addTarget,
		removeTarget,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/targets';
	import { config, loadSettings } from '$lib/stores/settings';

	let showAddModal = $state(false);
	let addSubmitting = $state(false);
	let addError = $state<string | null>(null);

	const maxDailyReplies = $derived($config?.targets?.max_target_replies_per_day ?? 3);

	async function handleAddTarget(username: string) {
		addSubmitting = true;
		addError = null;
		const err = await addTarget(username);
		addSubmitting = false;
		if (err) {
			addError = err;
		} else {
			showAddModal = false;
		}
	}

	function handleOpenAddModal() {
		addError = null;
		showAddModal = true;
	}

	async function handleRemoveTarget(username: string) {
		await removeTarget(username);
	}

	function handleViewTarget(username: string) {
		goto(`/targets/${encodeURIComponent(username)}`);
	}

	onMount(() => {
		loadTargets();
		loadSettings();
		startAutoRefresh();
	});

	onDestroy(() => {
		stopAutoRefresh();
	});
</script>

<svelte:head>
	<title>Targets — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<div class="page-header-row">
		<div>
			<h1>Target Accounts</h1>
			<p class="subtitle">
				{#if $targetCount > 0}
					{$targetCount} account{$targetCount !== 1 ? 's' : ''} monitored
					{#if $repliesToday > 0}
						&middot; {$repliesToday} {$repliesToday === 1 ? 'reply' : 'replies'} today
					{/if}
				{:else}
					Monitor accounts to build relationships
				{/if}
			</p>
		</div>
		<button class="add-btn" onclick={handleOpenAddModal}>
			<Plus size={16} />
			Add Target
		</button>
	</div>
</div>

{#if $error && $targets.length === 0}
	<ErrorState message={$error} onretry={() => loadTargets()} />
{:else if $error}
	<div class="error-banner">
		<span>{$error}</span>
		<button onclick={() => loadTargets()}>Retry</button>
	</div>
{/if}

<div class="targets-section">
	{#if $loading && $targets.length === 0}
		<div class="feed-container">
			{#each { length: 3 } as _}
				<div class="skeleton-item"></div>
			{/each}
		</div>
	{:else if $targets.length === 0}
		<div class="feed-container">
			<EmptyState
				title="No target accounts yet"
				description="Add accounts to build relationships through automated engagement."
				actionLabel="Add your first target"
				onaction={handleOpenAddModal}
			>
				{#snippet icon()}<Target size={40} strokeWidth={1.2} />{/snippet}
			</EmptyState>
		</div>
	{:else}
		<div class="feed-container">
			{#each $targets as target (target.account_id)}
				<TargetCard
					{target}
					{maxDailyReplies}
					onview={handleViewTarget}
					onremove={handleRemoveTarget}
				/>
			{/each}
		</div>
	{/if}
</div>

<AddTargetModal
	open={showAddModal}
	submitting={addSubmitting}
	error={addError}
	onclose={() => (showAddModal = false)}
	onsubmit={handleAddTarget}
/>

<style>
	.page-header {
		margin-bottom: 24px;
	}

	.page-header-row {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 16px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.add-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
		transition: background-color 0.15s ease;
	}

	.add-btn:hover {
		background: var(--color-accent-hover);
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

	.targets-section {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 0;
		overflow: hidden;
	}

	.feed-container {
		background-color: var(--color-base);
		overflow: hidden;
	}


	.skeleton-item {
		height: 130px;
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

	@media (max-width: 640px) {
		.page-header-row {
			flex-direction: column;
		}
	}
</style>
