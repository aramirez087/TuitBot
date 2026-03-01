<script lang="ts">
	import "../app.css";
	import { setToken, setAuthMode, setCsrfToken } from "$lib/api";
	import { connectWs } from "$lib/stores/websocket";
	import { initTheme } from "$lib/stores/theme";
	import { checkAuth, authMode as authModeStore } from "$lib/stores/auth";
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";
	import { page } from "$app/stores";
	import { api } from "$lib/api";

	let { children } = $props();
	let ready = $state(false);

	onMount(async () => {
		initTheme();

		// Step 1: Try Tauri token or dev fallback (bearer mode).
		let token = "";
		try {
			const { invoke } = await import("@tauri-apps/api/core");
			token = await invoke("get_api_token");
		} catch {
			token = __DEV_API_TOKEN__;
		}

		if (token) {
			// Bearer mode: Tauri desktop or dev mode — unchanged.
			setToken(token);
			setAuthMode("bearer");
			authModeStore.set("tauri");
			connectWs(token);

			// Check config for onboarding redirect.
			try {
				const status = await api.settings.configStatus();
				if (!status.configured && !$page.url.pathname.startsWith("/onboarding")) {
					goto("/onboarding");
				}
			} catch {
				// Server not ready — allow through.
			}
		} else {
			// Web/LAN mode: check config status FIRST, then auth.
			const path = $page.url.pathname;

			try {
				const status = await api.settings.configStatus();

				if (!status.configured) {
					// Fresh or unconfigured instance — onboarding (skip login).
					if (!path.startsWith("/onboarding")) {
						const target = status.claimed
							? "/onboarding?claimed=1"
							: "/onboarding";
						goto(target);
					}
					ready = true;
					return;
				}

				// Configured instance — check for existing session.
				const hasSession = await checkAuth();
				if (hasSession) {
					connectWs();
					if (path.startsWith("/login")) {
						goto("/");
					}
				} else {
					if (!path.startsWith("/login") && !path.startsWith("/onboarding")) {
						goto("/login");
						ready = true;
						return;
					}
				}
			} catch {
				// Server not reachable — allow through.
			}
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
