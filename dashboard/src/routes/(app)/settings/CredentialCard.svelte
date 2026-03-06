<script lang="ts">
	import {
		Link,
		Unlink,
		RefreshCw,
		Globe,
		Trash2,
		Loader2,
		Check,
		X,
		ExternalLink,
		ClipboardPaste,
		ShieldCheck,
		AlertCircle,
		ChevronDown,
		ChevronRight
	} from 'lucide-svelte';
	import { api } from '$lib/api';
	import type { AccountAuthStatus } from '$lib/api';
	import { syncAccountProfile, type Account } from '$lib/stores/accounts';
	import { reloadCapabilities } from '$lib/stores/runtime';
	import { getAccountId } from '$lib/api/http';

	let {
		account,
		status,
		onStatusChange
	}: {
		account: Account;
		status: AccountAuthStatus | undefined;
		onStatusChange: () => void;
	} = $props();

	// --- Expand state ---
	let expanded = $state(false);

	// --- OAuth link flow ---
	let oauthStep = $state<'idle' | 'waiting' | 'submitting'>('idle');
	let oauthStateParam = $state('');
	let callbackCode = $state('');
	let oauthError = $state('');
	let oauthSuccess = $state('');

	// --- OAuth unlink ---
	let unlinking = $state(false);
	let unlinkError = $state('');

	// --- Scraper import ---
	let showScraperForm = $state(false);
	let scraperAuthToken = $state('');
	let scraperCt0 = $state('');
	let scraperUsername = $state('');
	let importingScraper = $state(false);
	let scraperError = $state('');
	let scraperSuccess = $state('');

	// --- Scraper delete ---
	let deletingScraper = $state(false);

	const isActiveAccount = $derived(account.id === getAccountId());

	const oauthLabel = $derived.by(() => {
		if (!status?.oauth_linked) return 'Not linked';
		if (status.oauth_expired) return 'Expired';
		if (status.oauth_expires_at) {
			const d = new Date(status.oauth_expires_at);
			return `Linked (expires ${d.toLocaleDateString()})`;
		}
		return 'Linked';
	});

	const oauthColor = $derived.by(() => {
		if (!status?.oauth_linked) return 'none';
		if (status.oauth_expired) return 'expired';
		return 'linked';
	});

	const scraperLabel = $derived(status?.scraper_linked ? 'Linked' : 'Not linked');
	const scraperColor = $derived(status?.scraper_linked ? 'linked' : 'none');

	// --- OAuth actions ---

	async function startOAuthLink() {
		oauthError = '';
		oauthSuccess = '';
		callbackCode = '';
		oauthStep = 'submitting';
		try {
			const result = await api.accounts.startAuth(account.id);
			oauthStateParam = result.state;
			window.open(result.authorization_url, '_blank', 'noopener');
			oauthStep = 'waiting';
		} catch (e) {
			oauthError = e instanceof Error ? e.message : 'Failed to start OAuth flow';
			oauthStep = 'idle';
		}
	}

	async function completeOAuthLink() {
		const code = callbackCode.trim();
		if (!code || !oauthStateParam) return;
		oauthError = '';
		oauthStep = 'submitting';
		try {
			await api.accounts.completeAuth(account.id, code, oauthStateParam);
			oauthSuccess = 'X account linked successfully';
			oauthStep = 'idle';
			callbackCode = '';
			oauthStateParam = '';
			// Sync profile to pull avatar/username
			try {
				await syncAccountProfile(account.id);
			} catch {
				// Non-critical
			}
			if (isActiveAccount) await reloadCapabilities();
			onStatusChange();
		} catch (e) {
			oauthError = e instanceof Error ? e.message : 'Failed to complete OAuth';
			oauthStep = 'waiting';
		}
	}

	function cancelOAuth() {
		oauthStep = 'idle';
		callbackCode = '';
		oauthStateParam = '';
		oauthError = '';
	}

	async function handleUnlink() {
		unlinking = true;
		unlinkError = '';
		oauthSuccess = '';
		try {
			await api.accounts.unlinkOAuth(account.id);
			if (isActiveAccount) await reloadCapabilities();
			onStatusChange();
		} catch (e) {
			unlinkError = e instanceof Error ? e.message : 'Failed to unlink';
		} finally {
			unlinking = false;
		}
	}

	function handleCodeKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') completeOAuthLink();
		if (e.key === 'Escape') cancelOAuth();
	}

	// --- Scraper actions ---

	async function handleScraperImport() {
		if (!scraperAuthToken.trim() || !scraperCt0.trim()) {
			scraperError = 'Both auth_token and ct0 are required';
			return;
		}
		scraperError = '';
		scraperSuccess = '';
		importingScraper = true;
		try {
			await api.accounts.scraperSession.import(account.id, {
				auth_token: scraperAuthToken.trim(),
				ct0: scraperCt0.trim(),
				username: scraperUsername.trim() || undefined
			});
			scraperSuccess = 'Browser session imported';
			showScraperForm = false;
			scraperAuthToken = '';
			scraperCt0 = '';
			scraperUsername = '';
			if (isActiveAccount) await reloadCapabilities();
			onStatusChange();
		} catch (e) {
			scraperError = e instanceof Error ? e.message : 'Failed to import session';
		} finally {
			importingScraper = false;
		}
	}

	async function handleScraperDelete() {
		deletingScraper = true;
		scraperSuccess = '';
		scraperError = '';
		try {
			await api.accounts.scraperSession.delete(account.id);
			if (isActiveAccount) await reloadCapabilities();
			onStatusChange();
		} catch (e) {
			scraperError = e instanceof Error ? e.message : 'Failed to remove session';
		} finally {
			deletingScraper = false;
		}
	}

	function cancelScraperForm() {
		showScraperForm = false;
		scraperAuthToken = '';
		scraperCt0 = '';
		scraperUsername = '';
		scraperError = '';
	}
</script>

<div class="cred-card">
	<button class="cred-header" onclick={() => (expanded = !expanded)}>
		<span class="cred-account-name">
			{account.x_username ? `@${account.x_username}` : account.label}
		</span>
		<div class="cred-summary">
			<span class="cred-dot cred-dot-{oauthColor}" title="OAuth: {oauthLabel}"></span>
			<span class="cred-dot cred-dot-{scraperColor}" title="Scraper: {scraperLabel}"></span>
		</div>
		{#if expanded}
			<ChevronDown size={14} />
		{:else}
			<ChevronRight size={14} />
		{/if}
	</button>

	{#if expanded}
		<div class="cred-body">
			<!-- OAuth section -->
			<div class="cred-section">
				<div class="cred-section-header">
					<ShieldCheck size={13} />
					<span class="cred-section-label">OAuth Token</span>
					<span class="cred-status cred-status-{oauthColor}">{oauthLabel}</span>
				</div>

				{#if oauthStep === 'idle'}
					<div class="cred-actions">
						{#if !status?.oauth_linked}
							<button class="action-btn primary" onclick={startOAuthLink}>
								<Link size={12} />
								Link X Account
							</button>
						{:else}
							<button class="action-btn" onclick={startOAuthLink}>
								<RefreshCw size={12} />
								Relink
							</button>
							<button
								class="action-btn danger"
								onclick={handleUnlink}
								disabled={unlinking}
							>
								{#if unlinking}
									<Loader2 size={12} class="spinning" />
								{:else}
									<Unlink size={12} />
								{/if}
								Unlink
							</button>
						{/if}
					</div>
				{:else if oauthStep === 'waiting'}
					<div class="oauth-callback">
						<p class="oauth-hint">
							<ExternalLink size={12} />
							Authorize in the browser tab, then paste the code below.
						</p>
						<div class="oauth-code-row">
							<input
								class="oauth-code-input"
								type="text"
								placeholder="Paste authorization code"
								bind:value={callbackCode}
								onkeydown={handleCodeKeydown}
							/>
							<button
								class="action-btn primary"
								onclick={completeOAuthLink}
								disabled={!callbackCode.trim()}
							>
								<ClipboardPaste size={12} />
								Submit
							</button>
							<button class="action-btn" onclick={cancelOAuth}>
								<X size={12} />
							</button>
						</div>
					</div>
				{:else}
					<div class="cred-loading">
						<Loader2 size={14} class="spinning" />
						<span>Processing...</span>
					</div>
				{/if}

				{#if oauthError}
					<p class="cred-msg error">{oauthError}</p>
				{/if}
				{#if oauthSuccess}
					<p class="cred-msg success">
						<Check size={12} />
						{oauthSuccess}
					</p>
				{/if}
				{#if unlinkError}
					<p class="cred-msg error">{unlinkError}</p>
				{/if}
			</div>

			<!-- Scraper section -->
			<div class="cred-section">
				<div class="cred-section-header">
					<Globe size={13} />
					<span class="cred-section-label">Browser Session</span>
					<span class="cred-status cred-status-{scraperColor}">{scraperLabel}</span>
				</div>

				{#if !showScraperForm}
					<div class="cred-actions">
						{#if !status?.scraper_linked}
							<button
								class="action-btn primary"
								onclick={() => (showScraperForm = true)}
							>
								<ClipboardPaste size={12} />
								Import Session
							</button>
						{:else}
							<button
								class="action-btn"
								onclick={() => (showScraperForm = true)}
							>
								<RefreshCw size={12} />
								Replace
							</button>
							<button
								class="action-btn danger"
								onclick={handleScraperDelete}
								disabled={deletingScraper}
							>
								{#if deletingScraper}
									<Loader2 size={12} class="spinning" />
								{:else}
									<Trash2 size={12} />
								{/if}
								Remove
							</button>
						{/if}
					</div>
				{:else}
					<div class="scraper-form">
						<input
							class="scraper-input"
							type="text"
							placeholder="auth_token cookie"
							bind:value={scraperAuthToken}
							disabled={importingScraper}
						/>
						<input
							class="scraper-input"
							type="text"
							placeholder="ct0 cookie"
							bind:value={scraperCt0}
							disabled={importingScraper}
						/>
						<input
							class="scraper-input"
							type="text"
							placeholder="Username (optional)"
							bind:value={scraperUsername}
							disabled={importingScraper}
						/>
						<div class="scraper-form-actions">
							<button
								class="action-btn primary"
								onclick={handleScraperImport}
								disabled={importingScraper || !scraperAuthToken.trim() || !scraperCt0.trim()}
							>
								{#if importingScraper}
									<Loader2 size={12} class="spinning" />
								{:else}
									<Check size={12} />
								{/if}
								Import
							</button>
							<button
								class="action-btn"
								onclick={cancelScraperForm}
								disabled={importingScraper}
							>
								<X size={12} />
								Cancel
							</button>
						</div>
					</div>
				{/if}

				{#if scraperError}
					<p class="cred-msg error">{scraperError}</p>
				{/if}
				{#if scraperSuccess}
					<p class="cred-msg success">
						<Check size={12} />
						{scraperSuccess}
					</p>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.cred-card {
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-base);
		overflow: hidden;
		transition: border-color 0.15s;
	}

	.cred-card:hover {
		border-color: var(--color-border);
	}

	.cred-header {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 12px;
		background: transparent;
		border: none;
		cursor: pointer;
		color: var(--color-text-muted);
		font-size: 12px;
		text-align: left;
	}

	.cred-header:hover {
		background: var(--color-surface-hover);
	}

	.cred-account-name {
		font-weight: 600;
		font-size: 12px;
		color: var(--color-text);
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.cred-summary {
		display: flex;
		gap: 4px;
		align-items: center;
	}

	.cred-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.cred-dot-linked {
		background: var(--color-success);
	}

	.cred-dot-expired {
		background: var(--color-warning);
	}

	.cred-dot-none {
		background: var(--color-text-subtle);
		opacity: 0.4;
	}

	.cred-body {
		padding: 0 12px 12px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.cred-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.cred-section-header {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.cred-section-label {
		flex: 1;
	}

	.cred-status {
		font-size: 11px;
		font-weight: 500;
		text-transform: none;
		letter-spacing: 0;
	}

	.cred-status-linked {
		color: var(--color-success);
	}

	.cred-status-expired {
		color: var(--color-warning);
	}

	.cred-status-none {
		color: var(--color-text-subtle);
	}

	.cred-actions {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.action-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		font-size: 11px;
		font-weight: 500;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: var(--color-surface);
		color: var(--color-text);
		cursor: pointer;
		transition: background 0.15s, border-color 0.15s, color 0.15s;
		white-space: nowrap;
	}

	.action-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		border-color: var(--color-border);
	}

	.action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.action-btn.primary {
		background: var(--color-accent);
		border-color: var(--color-accent);
		color: white;
	}

	.action-btn.primary:hover:not(:disabled) {
		opacity: 0.9;
		background: var(--color-accent);
	}

	.action-btn.danger {
		color: var(--color-danger);
		border-color: color-mix(in srgb, var(--color-danger) 30%, transparent);
	}

	.action-btn.danger:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	/* OAuth callback flow */
	.oauth-callback {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.oauth-hint {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--color-text-muted);
		margin: 0;
		line-height: 1.4;
	}

	.oauth-code-row {
		display: flex;
		gap: 6px;
		align-items: center;
	}

	.oauth-code-input {
		flex: 1;
		min-width: 0;
		padding: 5px 8px;
		font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 5px;
		color: var(--color-text);
	}

	.oauth-code-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.cred-loading {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12px;
		color: var(--color-text-muted);
		padding: 4px 0;
	}

	/* Scraper form */
	.scraper-form {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.scraper-input {
		padding: 5px 8px;
		font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 5px;
		color: var(--color-text);
	}

	.scraper-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.scraper-input::placeholder {
		color: var(--color-text-subtle);
		font-family: inherit;
	}

	.scraper-form-actions {
		display: flex;
		gap: 6px;
		margin-top: 2px;
	}

	/* Messages */
	.cred-msg {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		margin: 0;
		line-height: 1.4;
	}

	.cred-msg.error {
		color: var(--color-danger);
	}

	.cred-msg.success {
		color: var(--color-success);
	}

	/* Spinner animation */
	:global(.spinning) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
