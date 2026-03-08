<script lang="ts">
	import { onMount } from 'svelte';
	import DraftStudioQuickStart from '$lib/components/home/DraftStudioQuickStart.svelte';
	import AnalyticsHome from '$lib/components/home/AnalyticsHome.svelte';
	import { homeSurface, homeSurfaceReady, loadHomeSurface } from '$lib/stores/homeSurface';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

	onMount(() => {
		loadHomeSurface();
		const handler = () => loadHomeSurface();
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});
</script>

<svelte:head>
	<title>{$homeSurface === 'drafts' ? 'Home' : 'Dashboard'} — Tuitbot</title>
</svelte:head>

{#if !$homeSurfaceReady}
	<div class="loading-skeleton"></div>
{:else if $homeSurface === 'drafts'}
	<DraftStudioQuickStart />
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

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}
</style>
