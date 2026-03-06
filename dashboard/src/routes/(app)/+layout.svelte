<script lang="ts">
	import Sidebar from "$lib/components/Sidebar.svelte";
	import ConnectionBanner from "$lib/components/ConnectionBanner.svelte";
	import { loadStats as loadApprovalStats } from "$lib/stores/approval";
	import { connected } from "$lib/stores/websocket";
	import { checkForUpdate } from "$lib/stores/update";
	import { initAccounts, syncCurrentProfile, bootstrapState } from "$lib/stores/accounts";
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
		'7': '/costs',
		'8': '/observability',
		'9': '/settings',
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
				if ($page.url.pathname === '/') {
					// Already on home — focus the compose textarea
					window.dispatchEvent(new CustomEvent('tuitbot:compose'));
				} else {
					// Navigate to home (composer is default surface)
					goto('/');
				}
			}
		}
	}

	onMount(async () => {
		await initAccounts();
		syncCurrentProfile();
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
		{#if $bootstrapState === 'loading'}
			<div class="bootstrap-loading" in:fade={{ duration: 100 }}>
				<div class="bootstrap-spinner"></div>
			</div>
		{:else}
			{#if !$connected}
				<ConnectionBanner />
			{/if}
			{#key $page.url.pathname}
				<div in:fade={{ duration: 150 }}>
					{@render children()}
				</div>
			{/key}
		{/if}
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

	.bootstrap-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 200px;
	}

	.bootstrap-spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border-subtle);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
