<script lang="ts">
	import "../app.css";
	import Sidebar from "$lib/components/Sidebar.svelte";
	import { setToken } from "$lib/api";
	import { connectWs } from "$lib/stores/websocket";
	import { loadStats as loadApprovalStats } from "$lib/stores/approval";
	import { initTheme } from "$lib/stores/theme";
	import { onMount } from "svelte";

	let { children } = $props();

	onMount(async () => {
		initTheme();
		// In Tauri context, get token from the sidecar via invoke.
		// In browser dev mode, fall back to the token injected by Vite.
		let token = "";
		try {
			const { invoke } = await import("@tauri-apps/api/core");
			token = await invoke("get_api_token");
		} catch {
			// Not running in Tauri — use the dev token injected by vite.config.ts.
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
			loadApprovalStats();
		}
	});
</script>

<div class="app-shell">
	<Sidebar />
	<main class="main-content">
		{@render children()}
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
