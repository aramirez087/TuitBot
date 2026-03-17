<script lang="ts">
	import { CheckCircle2, Loader2, RefreshCw, Sparkles } from 'lucide-svelte';
	import { onboardingSession } from '$lib/stores/onboarding-session';

	interface Props {
		heroMode: boolean;
		analysisPhase: 'idle' | 'running' | 'done' | 'failed';
		timeoutCounter?: number;
		onStartAuth: () => void;
		onSetScraperMode: () => void;
	}

	const { heroMode, analysisPhase, timeoutCounter = 0, onStartAuth, onSetScraperMode }: Props = $props();
</script>

<div class="connect-section">
	{#if $onboardingSession.x_connected && $onboardingSession.x_user}
		<!-- Connected state (shared between hero and dev modes) -->
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
				<span class="display-name">{$onboardingSession.x_user.name}</span>
				<span class="username">@{$onboardingSession.x_user.username}</span>
			</div>
			<div class="connected-badge">
				<CheckCircle2 size={18} />
				Connected
			</div>
		</div>

		{#if analysisPhase === 'running'}
			<div class="analysis-status" role="status" aria-live="polite" aria-label="Profile analysis in progress">
				<span class="spinner"><Loader2 size={14} /></span>
				<span>Analyzing your profile...</span>
			</div>
		{:else if analysisPhase === 'done'}
			<div class="analysis-status analysis-done" role="status" aria-live="polite" aria-label="Profile analysis complete">
				<Sparkles size={14} />
				<span>Profile analyzed — fields pre-filled below.</span>
			</div>
		{:else if analysisPhase === 'failed'}
			<div class="analysis-status analysis-failed" role="alert" aria-live="assertive">
				<span class="error-icon">⚠</span>
				<span>Profile analysis failed. You can fill in the fields manually below.</span>
			</div>
		{/if}
	{:else if heroMode}
		<!-- Hero mode: full-width centred login -->
		<div class="hero-connect">
			{#if $onboardingSession.auth_error}
				<div class="auth-error" role="alert">
					{$onboardingSession.auth_error}
				</div>
			{/if}

			<button
				type="button"
				class="btn-connect btn-connect-hero"
				onclick={$onboardingSession.auth_loading ? undefined : onStartAuth}
				disabled={$onboardingSession.auth_loading}
			>
				{#if $onboardingSession.auth_loading}
					<span class="spinner"><Loader2 size={18} /></span>
					Waiting for X...
				{:else if $onboardingSession.auth_error}
					<RefreshCw size={18} />
					Retry
				{:else}
					<svg
						viewBox="0 0 24 24"
						width="18"
						height="18"
						fill="currentColor"
						aria-hidden="true"
					>
						<path
							d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
						/>
					</svg>
					Login with X
				{/if}
			</button>

			{#if $onboardingSession.auth_loading}
				<p class="connect-hint">
					Complete the sign-in in the window that opened, then return here.
					{#if timeoutCounter > 0}
						<span class="timeout-hint">(Expires in {timeoutCounter}s)</span>
					{/if}
				</p>
			{/if}
		</div>

		<button type="button" class="scraper-fallback" onclick={onSetScraperMode}>
			Or continue without an X account
		</button>
	{:else}
		<!-- Dev mode: boxed prompt -->
		<div class="connect-prompt">
			<p class="connect-heading">Connect your X account</p>
			<p class="connect-desc">
				Sign in with X to speed up setup. We'll pre-fill your profile info from
				your account.
			</p>

			{#if $onboardingSession.auth_error}
				<div class="auth-error" role="alert">
					{$onboardingSession.auth_error}
				</div>
			{/if}

			<button
				type="button"
				class="btn-connect"
				onclick={$onboardingSession.auth_loading ? undefined : onStartAuth}
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
					Complete the sign-in in the window that opened, then return here.
					{#if timeoutCounter > 0}
						<span class="timeout-hint">(Expires in {timeoutCounter}s)</span>
					{/if}
				</p>
			{/if}

			<p class="connect-skip">
				You can skip this and connect later in Settings.
			</p>
		</div>
	{/if}
</div>

<style>
	.connect-section {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 4px;
	}

	/* --- Hero connect --- */
	.hero-connect {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 14px;
		padding: 32px 24px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 10px;
	}

	.btn-connect-hero {
		padding: 14px 32px;
		font-size: 16px;
		border-radius: 10px;
		align-self: center;
	}

	.scraper-fallback {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 13px;
		cursor: pointer;
		padding: 4px 0;
		transition: color 0.15s;
	}

	.scraper-fallback:hover {
		color: var(--color-text-muted);
		text-decoration: underline;
	}

	/* --- Dev mode connect prompt --- */
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

	/* --- Shared connect button --- */
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
		text-align: center;
	}

	.timeout-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-weight: 500;
		font-style: normal;
		display: block;
		margin-top: 2px;
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

	/* --- Connected card --- */
	.connected-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 16px;
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border: 1px solid
			color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
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

	/* --- Analysis status --- */
	.analysis-status {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, var(--color-border));
		border-radius: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.analysis-done {
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border-color: color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
		color: var(--color-success, #22c55e);
	}

	.analysis-failed {
		background: color-mix(in srgb, var(--color-danger) 6%, var(--color-base));
		border-color: color-mix(in srgb, var(--color-danger) 25%, var(--color-border));
		color: var(--color-danger);
	}

	.error-icon {
		font-size: 14px;
		font-weight: 600;
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
</style>
