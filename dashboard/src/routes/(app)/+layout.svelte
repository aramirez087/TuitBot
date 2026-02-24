<script lang="ts">
	import Sidebar from "$lib/components/Sidebar.svelte";
	import ConnectionBanner from "$lib/components/ConnectionBanner.svelte";
	import { loadStats as loadApprovalStats } from "$lib/stores/approval";
	import { connected } from "$lib/stores/websocket";
	import { checkForUpdate } from "$lib/stores/update";
	import { onMount, onDestroy } from "svelte";
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";
	import { fade } from "svelte/transition";

	let { children } = $props();

	const shortcuts: Record<string, string> = {
		'1': '/',
		'2': '/activity',
		'3': '/approval',
		'4': '/content',
		'5': '/targets',
		'6': '/strategy',
		'7': '/settings',
	};

	function handleKeydown(e: KeyboardEvent) {
		if (e.metaKey || e.ctrlKey) {
			const route = shortcuts[e.key];
			if (route) {
				e.preventDefault();
				goto(route);
				return;
			}
			if (e.key === ',') {
				e.preventDefault();
				goto('/settings');
				return;
			}
			if (e.key === 'n' || e.key === 'N') {
				e.preventDefault();
				window.dispatchEvent(new CustomEvent('tuitbot:compose'));
			}
		}
	}

	onMount(() => {
		loadApprovalStats();
		checkForUpdate();
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		if (typeof window !== 'undefined') {
			window.removeEventListener('keydown', handleKeydown);
		}
	});
</script>

<div class="app-shell">
	<Sidebar />
	<main class="main-content">
		{#if !$connected}
			<ConnectionBanner />
		{/if}
		{#key $page.url.pathname}
			<div in:fade={{ duration: 150 }}>
				{@render children()}
			</div>
		{/key}
	</main>
</div>

<style>
	.app-shell {
		display: flex;
		min-height: 100vh;
		background-color: var(--color-base);
	}

	.main-content {
		flex: 1;
		padding: 24px 32px;
		overflow-y: auto;
	}
</style>
