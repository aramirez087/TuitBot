<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type ComposeRequest, type ScheduleConfig } from '$lib/api';
	import ComposeWorkspace from '$lib/components/composer/ComposeWorkspace.svelte';
	import AnalyticsHome from '$lib/components/home/AnalyticsHome.svelte';
	import { persistGet } from '$lib/stores/persistence';

	type HomeSurface = 'composer' | 'analytics';
	let homeSurface = $state<HomeSurface>('composer');
	let loaded = $state(false);
	let schedule = $state<ScheduleConfig | null>(null);

	onMount(async () => {
		homeSurface = await persistGet<HomeSurface>('home_surface', 'composer');
		loaded = true;
		try {
			const cfg = await api.content.schedule();
			schedule = cfg;
		} catch {
			// Schedule loading is non-critical; workspace works without it
		}
	});

	async function handleSubmit(data: ComposeRequest) {
		await api.content.compose(data);
	}
</script>

<svelte:head>
	<title>{homeSurface === 'composer' ? 'Compose' : 'Dashboard'} — Tuitbot</title>
</svelte:head>

{#if !loaded}
	<div class="loading-skeleton"></div>
{:else if homeSurface === 'composer'}
	<div class="home-composer">
		<ComposeWorkspace
			{schedule}
			onsubmit={handleSubmit}
			embedded={true}
		/>
	</div>
{:else}
	<AnalyticsHome />
{/if}

<style>
	.loading-skeleton {
		height: 200px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.home-composer {
		max-width: 860px;
		margin: 0 auto;
		min-height: calc(100vh - 120px);
		display: flex;
		flex-direction: column;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}
</style>
