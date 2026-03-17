<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { onboardingSession } from '$lib/stores/onboarding-session';
	import type { OnboardingXUser } from '$lib/stores/onboarding-session';
	import { deploymentMode } from '$lib/stores/runtime';
	import { api } from '$lib/api';
	import { trackFunnel } from '$lib/analytics/funnel';
	import XApiGuide from './XApiGuide.svelte';
	import XApiKeyForm from './XApiKeyForm.svelte';
	import XApiConnectPanel from './XApiConnectPanel.svelte';
	import XApiFeatureMatrix from './XApiFeatureMatrix.svelte';

	const CALLBACK_URL = 'http://127.0.0.1:8080/callback';

	interface Props {
		hasServerClientId?: boolean;
	}

	let { hasServerClientId = false }: Props = $props();

	let clientId = $state($onboardingData.client_id);
	let clientSecret = $state($onboardingData.client_secret);
	let copied = $state(false);
	let analysisPhase = $state<'idle' | 'running' | 'done' | 'failed'>('idle');

	let selectedMode = $derived(
		$onboardingData.provider_backend === 'scraper' ? 'scraper' : 'x_api'
	);
	let isCloud = $derived($deploymentMode === 'cloud');
	let isHeroMode = $derived(hasServerClientId && !isCloud);
	let showConnect = $derived(
		!isHeroMode && selectedMode === 'x_api' && clientId.trim().length > 0 && !isCloud
	);

	let pollTimer: ReturnType<typeof setInterval> | null = null;
	let timeoutCounter = $state(0);

	$effect(() => {
		onboardingData.updateField('client_id', clientId);
	});

	$effect(() => {
		onboardingData.updateField('client_secret', clientSecret);
	});

	$effect(() => {
		return () => {
			if (pollTimer) {
				clearInterval(pollTimer);
				pollTimer = null;
			}
		};
	});

	function setMode(mode: string) {
		onboardingData.updateField('provider_backend', mode === 'x_api' ? '' : mode);
		if (mode === 'scraper') {
			trackFunnel('onboarding:scraper-selected');
		}
	}

	function copyCallbackUrl() {
		navigator.clipboard.writeText(CALLBACK_URL);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}

	async function startXAuth() {
		onboardingSession.setLoading(true);
		trackFunnel('onboarding:x-auth-started', {
			mode: selectedMode === 'scraper' ? 'scraper' : 'api',
			hero: isHeroMode,
		});
		try {
			const result = isHeroMode
				? await api.onboarding.startAuth()
				: await api.onboarding.startAuth(clientId.trim());
			onboardingSession.setAuthUrl(result.authorization_url, result.state);
			await openAuthWindow(result.authorization_url, result.state);
		} catch (e) {
			const msg = e instanceof Error ? e.message : 'Failed to start auth';
			trackFunnel('onboarding:x-auth-error', { error: msg });
			onboardingSession.setError(msg);
		}
	}

	async function openAuthWindow(url: string, oauthState: string) {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const { listen } = await import('@tauri-apps/api/event');
			const unlisten = await listen<{ code: string; state: string }>(
				'oauth-callback',
				async (event) => {
					unlisten();
					const { code, state } = event.payload;
					if (code && state === oauthState) {
						await handleOAuthCallback(code, state);
					}
				}
			);
			await invoke('open_oauth_window', { url });
		} catch {
			window.open(url, '_blank', 'noopener');
			startPolling();
		}
	}

	async function handleOAuthCallback(code: string, state: string) {
		try {
			const result = await api.onboarding.completeAuth(code, state);
			if (result.status === 'connected' && result.user) {
				const user = result.user as OnboardingXUser;
				onboardingSession.setConnected(user);
				trackFunnel('onboarding:x-auth-success', { username: user.username });
				onboardingData.updateField('x_user_id', user.id);
				onboardingData.updateField('x_username', user.username);
				onboardingData.updateField('x_display_name', user.name);
				onboardingData.updateField('x_avatar_url', user.profile_image_url ?? '');
				runInlineAnalysis();
			}
		} catch (e) {
			const msg = e instanceof Error ? e.message : 'OAuth callback failed';
			onboardingSession.setError(msg);
		}
	}

	function startPolling() {
		if (pollTimer) clearInterval(pollTimer);
		const TIMEOUT_MS = 300_000; // 5 minutes
		const COUNTDOWN_INTERVAL_MS = 1000; // update every 1 second
		let elapsed = 0;
		
		// Countdown timer: update every second
		const countdownTimer = setInterval(() => {
			elapsed += COUNTDOWN_INTERVAL_MS;
			timeoutCounter = Math.max(0, Math.floor((TIMEOUT_MS - elapsed) / 1000));
			if (elapsed >= TIMEOUT_MS) {
				clearInterval(countdownTimer);
			}
		}, COUNTDOWN_INTERVAL_MS);
		
		// Poll for auth status every 2 seconds
		pollTimer = setInterval(async () => {
			try {
				const status = await api.onboarding.authStatus();
				if (status.connected && status.user) {
					const user = status.user as OnboardingXUser;
					onboardingSession.setConnected(user);
					trackFunnel('onboarding:x-auth-success', { username: user.username });
					onboardingData.updateField('x_user_id', user.id);
					onboardingData.updateField('x_username', user.username);
					onboardingData.updateField('x_display_name', user.name);
					onboardingData.updateField('x_avatar_url', user.profile_image_url ?? '');
					if (pollTimer) {
						clearInterval(pollTimer);
						clearInterval(countdownTimer);
						pollTimer = null;
						timeoutCounter = 0;
					}
					runInlineAnalysis();
				}
			} catch {
				// Silently retry on network errors during polling.
			}
		}, 2000);

		// Timeout handler: clear timers and show error after 5 minutes
		setTimeout(() => {
			if (pollTimer) {
				clearInterval(pollTimer);
				clearInterval(countdownTimer);
				pollTimer = null;
				timeoutCounter = 0;
				if (!$onboardingSession.x_connected) {
					trackFunnel('onboarding:x-auth-error', { error: 'timeout' });
					onboardingSession.setError(
						'Connection timed out. Click "Login with X" to try again.'
					);
				}
			}
		}, TIMEOUT_MS);
	}

	async function runInlineAnalysis() {
		analysisPhase = 'running';
		onboardingSession.setAnalyzing(true);
		try {
			const result = await api.onboarding.analyzeProfile();
			if (result.profile) {
				onboardingData.prefillFromInference(result.profile);
				onboardingSession.setInferredProfile(result.profile, result.warnings ?? []);
				analysisPhase = 'done';
				trackFunnel('onboarding:inline-analysis-done', { status: result.status });
			} else {
				analysisPhase = 'failed';
			}
		} catch {
			analysisPhase = 'failed';
		}
		onboardingSession.setAnalyzing(false);
	}
</script>

<div class="step">
	{#if isHeroMode}
		<h2 class="step-title">Connect Your X Account</h2>
		<p class="step-description">
			Sign in with X to get started. We'll analyze your profile to set up
			Tuitbot automatically.
		</p>
		<XApiConnectPanel
			heroMode={true}
			{analysisPhase}
			{timeoutCounter}
			onStartAuth={startXAuth}
			onSetScraperMode={() => setMode('scraper')}
		/>
	{:else}
		<h2 class="step-title">X Access</h2>
		<p class="step-description">Choose how Tuitbot connects to X.</p>

		{#if !isCloud}
			<div class="mode-selector">
				<button
					type="button"
					class="mode-card"
					class:selected={selectedMode === 'x_api'}
					onclick={() => setMode('x_api')}
				>
					<div class="mode-radio" class:checked={selectedMode === 'x_api'}></div>
					<div class="mode-info">
						<span class="mode-label"
							>Official X API <span class="mode-badge">Recommended</span></span
						>
						<span class="mode-desc"
							>Full features. Post, discover, and engage. Requires Client ID from the
							Developer Portal.</span
						>
					</div>
				</button>
				<button
					type="button"
					class="mode-card"
					class:selected={selectedMode === 'scraper'}
					onclick={() => setMode('scraper')}
				>
					<div class="mode-radio" class:checked={selectedMode === 'scraper'}></div>
					<div class="mode-info">
						<span class="mode-label">Local No-Key Mode</span>
						<span class="mode-desc"
							>Get started instantly. No API credentials needed. Discovery and drafting.
							Read-only by default.</span
						>
					</div>
				</button>
			</div>
		{/if}

		{#if selectedMode === 'x_api' || isCloud}
			<XApiGuide
				callbackUrl={CALLBACK_URL}
				{copied}
				onCopy={copyCallbackUrl}
			/>
			<XApiKeyForm bind:clientId bind:clientSecret />
			{#if showConnect}
				<XApiConnectPanel
					heroMode={false}
					{analysisPhase}
					{timeoutCounter}
					onStartAuth={startXAuth}
					onSetScraperMode={() => setMode('scraper')}
				/>
			{/if}
		{:else}
			<XApiFeatureMatrix />
		{/if}
	{/if}
</div>

<style>
	.step {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.step-title {
		font-size: 20px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.step-description {
		font-size: 14px;
		color: var(--color-text-muted);
		line-height: 1.5;
		margin: 0;
	}

	.mode-selector {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.mode-card {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 14px 16px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		cursor: pointer;
		text-align: left;
		transition:
			border-color 0.15s,
			background 0.15s;
	}

	.mode-card:hover {
		border-color: var(--color-border-hover, var(--color-text-muted));
	}

	.mode-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, var(--color-base));
	}

	.mode-radio {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 2px solid var(--color-border);
		flex-shrink: 0;
		margin-top: 2px;
		transition: border-color 0.15s;
	}

	.mode-radio.checked {
		border-color: var(--color-accent);
		background: var(--color-accent);
		box-shadow: inset 0 0 0 3px var(--color-base);
	}

	.mode-info {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.mode-label {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
	}

	.mode-badge {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		padding: 1px 6px;
		border-radius: 3px;
		margin-left: 4px;
		vertical-align: middle;
	}

	.mode-desc {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.4;
	}
</style>
