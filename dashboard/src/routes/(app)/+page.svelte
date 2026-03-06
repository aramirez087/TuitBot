<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type ComposeRequest, type ScheduleConfig } from '$lib/api';
	import ComposeWorkspace from '$lib/components/composer/ComposeWorkspace.svelte';
	import AnalyticsHome from '$lib/components/home/AnalyticsHome.svelte';
	import { homeSurface, homeSurfaceReady, loadHomeSurface } from '$lib/stores/homeSurface';

	let schedule = $state<ScheduleConfig | null>(null);
	let canPublish = $state(true);

	onMount(async () => {
		await loadHomeSurface();
		try {
			const [cfg, rt] = await Promise.all([
				api.content.schedule(),
				api.runtime.status(),
			]);
			schedule = cfg;
			canPublish = rt.can_post;
		} catch {
			// Non-critical; workspace works without these
		}
	});

	async function handleSubmit(data: ComposeRequest) {
		await api.content.compose(data);
	}
</script>

<svelte:head>
	<title>{$homeSurface === 'composer' ? 'Compose' : 'Dashboard'} — Tuitbot</title>
</svelte:head>

{#if !$homeSurfaceReady}
	<div class="loading-skeleton"></div>
{:else if $homeSurface === 'composer'}
	<div class="home-composer">
		<ComposeWorkspace
			{schedule}
			{canPublish}
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
