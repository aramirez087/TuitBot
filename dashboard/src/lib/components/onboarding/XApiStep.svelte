<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { onboardingSession } from '$lib/stores/onboarding-session';
	import type { OnboardingXUser } from '$lib/stores/onboarding-session';
	import { deploymentMode } from '$lib/stores/runtime';
	import { api } from '$lib/api';
	import {
		ExternalLink,
		Copy,
		Check,
		CheckCircle2,
		XCircle,
		Loader2,
		RefreshCw
	} from 'lucide-svelte';

	const CALLBACK_URL = 'http://127.0.0.1:8080/callback';

	let clientId = $state($onboardingData.client_id);
	let clientSecret = $state($onboardingData.client_secret);
	let copied = $state(false);

	let selectedMode = $derived(
		$onboardingData.provider_backend === 'scraper' ? 'scraper' : 'x_api'
	);
	let isCloud = $derived($deploymentMode === 'cloud');
	let showConnect = $derived(
		selectedMode === 'x_api' && clientId.trim().length > 0 && !isCloud
	);

	// Poll timer reference for cleanup.
	let pollTimer: ReturnType<typeof setInterval> | null = null;

	$effect(() => {
		onboardingData.updateField('client_id', clientId);
	});

	$effect(() => {
		onboardingData.updateField('client_secret', clientSecret);
	});

	// Clean up poll timer on destroy.
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
	}

	function copyCallbackUrl() {
		navigator.clipboard.writeText(CALLBACK_URL);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}

	async function startXAuth() {
		onboardingSession.setLoading(true);
		try {
			const result = await api.onboarding.startAuth();
			onboardingSession.setAuthUrl(result.authorization_url, result.state);

			// Open the auth URL.
			window.open(result.authorization_url, '_blank', 'noopener');

			// Start polling for completion.
			startPolling();
		} catch (e) {
			const msg = e instanceof Error ? e.message : 'Failed to start auth';
			onboardingSession.setError(msg);
		}
	}

	function startPolling() {
		if (pollTimer) clearInterval(pollTimer);
		pollTimer = setInterval(async () => {
			try {
				const status = await api.onboarding.authStatus();
				if (status.connected && status.user) {
					onboardingSession.setConnected(status.user as OnboardingXUser);
					if (pollTimer) {
						clearInterval(pollTimer);
						pollTimer = null;
					}
				}
			} catch {
				// Silently retry on network errors during polling.
			}
		}, 2000);

		// Stop polling after 5 minutes to avoid infinite loops.
		setTimeout(() => {
			if (pollTimer) {
				clearInterval(pollTimer);
				pollTimer = null;
				if (!$onboardingSession.x_connected) {
					onboardingSession.setError(
						'Connection timed out. Click "Connect with X" to try again.'
					);
				}
			}
		}, 300_000);
	}

	function retryAuth() {
		onboardingSession.setError('');
		startXAuth();
	}
</script>

<div class="step">
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
		<div class="setup-guide">
			<p class="guide-heading">Quick setup (~2 minutes)</p>
			<ol class="guide-steps">
				<li>
					Go to the <a
						href="https://developer.x.com/en/portal/dashboard"
						target="_blank"
						rel="noopener noreferrer"
						class="link"
						>X Developer Portal <ExternalLink size={10} /></a
					> and create a Project &amp; App (or select an existing one)
				</li>
				<li>
					Under <strong>User authentication settings</strong>, enable OAuth 2.0
				</li>
				<li>
					Set App type to <strong>Native App</strong> and paste this as your Callback
					URL:
					<span class="callback-url">
						<code>{CALLBACK_URL}</code>
						<button
							class="copy-btn"
							onclick={copyCallbackUrl}
							title="Copy to clipboard"
						>
							{#if copied}
								<Check size={13} />
							{:else}
								<Copy size={13} />
							{/if}
						</button>
					</span>
				</li>
				<li>Copy the <strong>Client ID</strong> from the "Keys and tokens" tab</li>
			</ol>
		</div>

		<div class="fields">
			<div class="field">
				<label class="field-label" for="client-id"
					>Client ID <span class="required">*</span></label
				>
				<input
					id="client-id"
					type="text"
					class="field-input"
					placeholder="Your OAuth 2.0 Client ID"
					bind:value={clientId}
				/>
			</div>

			<div class="field">
				<label class="field-label" for="client-secret"
					>Client Secret <span class="optional">(optional)</span></label
				>
				<input
					id="client-secret"
					type="password"
					class="field-input"
					placeholder="For confidential clients only"
					bind:value={clientSecret}
				/>
				<span class="field-hint"
					>Only needed for confidential OAuth clients. Most users can skip this.</span
				>
			</div>
		</div>

		{#if showConnect}
			<div class="connect-section">
				{#if $onboardingSession.x_connected && $onboardingSession.x_user}
					<div class="connected-card">
						{#if $onboardingSession.x_user.profile_image_url}
							<img
								src={$onboardingSession.x_user.profile_image_url}
								alt=""
								class="avatar"
							/>
						{:else}
							<div class="avatar avatar-placeholder"></div>
						{/if}
						<div class="connected-info">
							<span class="display-name"
								>{$onboardingSession.x_user.name}</span
							>
							<span class="username"
								>@{$onboardingSession.x_user.username}</span
							>
						</div>
						<div class="connected-badge">
							<CheckCircle2 size={18} />
							Connected
						</div>
					</div>
				{:else}
					<div class="connect-prompt">
						<p class="connect-heading">Connect your X account</p>
						<p class="connect-desc">
							Sign in with X to speed up setup. We'll pre-fill your profile
							info from your account.
						</p>

						{#if $onboardingSession.auth_error}
							<div class="auth-error" role="alert">
								{$onboardingSession.auth_error}
							</div>
						{/if}

						<button
							type="button"
							class="btn-connect"
							onclick={$onboardingSession.auth_loading ? undefined : startXAuth}
							disabled={$onboardingSession.auth_loading}
						>
							{#if $onboardingSession.auth_loading}
								<span class="spinner"><Loader2 size={16} /></span>
								Waiting for X...
							{:else if $onboardingSession.auth_error}
								<RefreshCw size={16} />
								Retry
							{:else}
								<svg
									viewBox="0 0 24 24"
									width="16"
									height="16"
									fill="currentColor"
									aria-hidden="true"
								>
									<path
										d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
									/>
								</svg>
								Continue with X
							{/if}
						</button>

						{#if $onboardingSession.auth_loading}
							<p class="connect-hint">
								Complete the sign-in in the window that opened, then return
								here.
							</p>
						{/if}

						<p class="connect-skip">
							You can skip this and connect later in Settings.
						</p>
					</div>
				{/if}
			</div>
		{/if}
	{:else}
		<div class="feature-matrix">
			<p class="matrix-heading">What you can do</p>
			<ul class="matrix-list">
				<li class="available"><CheckCircle2 size={14} /> Search and discover tweets</li>
				<li class="available">
					<CheckCircle2 size={14} /> Score conversations for relevance
				</li>
				<li class="available">
					<CheckCircle2 size={14} /> Draft replies and original content
				</li>
				<li class="available">
					<CheckCircle2 size={14} /> Plan and preview threads
				</li>
				<li class="unavailable">
					<XCircle size={14} /> Post tweets and replies
				</li>
				<li class="unavailable">
					<XCircle size={14} /> Mentions and home timeline
				</li>
			</ul>
			<p class="matrix-footer">
				You can switch to the Official X API anytime in Settings.
			</p>
		</div>
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

	.link {
		color: var(--color-accent);
		text-decoration: none;
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}

	.link:hover {
		text-decoration: underline;
	}

	.fields {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.required {
		color: var(--color-danger);
	}

	.optional {
		font-weight: 400;
		color: var(--color-text-subtle);
	}

	.field-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		transition: border-color 0.15s;
	}

	.field-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.field-input::placeholder {
		color: var(--color-text-subtle);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.setup-guide {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 14px 16px;
	}

	.guide-heading {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin: 0 0 10px;
	}

	.guide-steps {
		margin: 0;
		padding-left: 20px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.guide-steps strong {
		color: var(--color-text);
	}

	.callback-url {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 4px;
	}

	.callback-url code {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		padding: 3px 8px;
		font-size: 12px;
		color: var(--color-accent);
		font-family: monospace;
		user-select: all;
	}

	.copy-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 3px;
		background: transparent;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s;
	}

	.copy-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	/* --- Connect with X section --- */

	.connect-section {
		margin-top: 4px;
	}

	.connect-prompt {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.connect-heading {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.connect-desc {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.4;
		margin: 0;
	}

	.btn-connect {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 10px 20px;
		background: var(--color-text);
		color: var(--color-base);
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition:
			opacity 0.15s,
			transform 0.1s;
		align-self: flex-start;
	}

	.btn-connect:hover:not(:disabled) {
		opacity: 0.85;
	}

	.btn-connect:active:not(:disabled) {
		transform: scale(0.98);
	}

	.btn-connect:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.connect-hint {
		font-size: 12px;
		color: var(--color-text-muted);
		margin: 0;
		font-style: italic;
	}

	.connect-skip {
		font-size: 12px;
		color: var(--color-text-subtle);
		margin: 0;
	}

	.auth-error {
		padding: 8px 12px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		border-radius: 6px;
		color: var(--color-danger);
		font-size: 13px;
	}

	/* --- Connected state --- */

	.connected-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 16px;
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
		border-radius: 8px;
	}

	.avatar {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.avatar-placeholder {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
	}

	.connected-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		flex: 1;
		min-width: 0;
	}

	.display-name {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.username {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.connected-badge {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: 12px;
		font-weight: 600;
		color: var(--color-success, #22c55e);
		flex-shrink: 0;
	}

	.spinner {
		display: inline-flex;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.feature-matrix {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 14px 16px;
	}

	.matrix-heading {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin: 0 0 10px;
	}

	.matrix-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.matrix-list li {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		line-height: 1.4;
	}

	.matrix-list li.available {
		color: var(--color-success, #22c55e);
	}

	.matrix-list li.unavailable {
		color: var(--color-text-subtle);
	}

	.matrix-footer {
		margin: 12px 0 0;
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
	}
</style>
