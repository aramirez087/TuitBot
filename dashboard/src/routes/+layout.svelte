<script lang="ts">
	import "../app.css";
	import { setToken } from "$lib/api";
	import { connectWs } from "$lib/stores/websocket";
	import { initTheme } from "$lib/stores/theme";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { page } from "$app/stores";
	import { api } from "$lib/api";

	let { children } = $props();
	let ready = $state(false);

	onMount(async () => {
		initTheme();

		// Get API token from Tauri or dev fallback.
		let token = "";
		try {
			const { invoke } = await import("@tauri-apps/api/core");
			token = await invoke("get_api_token");
		} catch {
			token = __DEV_API_TOKEN__;
			if (!token) {
				console.warn(
					"No API token available. Start tuitbot-server to generate ~/.tuitbot/api_token, then restart the dev server.",
				);
			}
		}

		if (token) {
			setToken(token);
			connectWs(token);
		}

		// Check if config exists — redirect to onboarding if not.
		try {
			const status = await api.settings.configStatus();
			if (!status.configured && !$page.url.pathname.startsWith("/onboarding")) {
				goto("/onboarding");
			}
		} catch {
			// Server not ready yet — allow through.
		}

		ready = true;
	});
</script>

{#if ready}
	{@render children()}
{:else}
	<div class="loading-screen">
		<div class="loading-spinner"></div>
	</div>
{/if}

<style>
	.loading-screen {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		background-color: var(--color-base);
	}

	.loading-spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--color-border-subtle);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
