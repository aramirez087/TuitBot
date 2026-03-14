<script lang="ts">
	import {
		Link,
		Unlink,
		RefreshCw,
		Loader2,
		Check,
		X,
		ExternalLink,
		ClipboardPaste,
		ShieldCheck,
	} from 'lucide-svelte';
	import { api } from '$lib/api';
	import type { AccountAuthStatus } from '$lib/api';
	import { syncAccountProfile, type Account } from '$lib/stores/accounts';
	import { reloadCapabilities } from '$lib/stores/runtime';

	interface Props {
		account: Account;
		status: AccountAuthStatus | undefined;
		isActiveAccount: boolean;
		onStatusChange: () => void;
	}

	const { account, status, isActiveAccount, onStatusChange }: Props = $props();

	let oauthStep = $state<'idle' | 'waiting' | 'submitting'>('idle');
	let oauthStateParam = $state('');
	let callbackCode = $state('');
	let oauthError = $state('');
	let oauthSuccess = $state('');
	let unlinking = $state(false);
	let unlinkError = $state('');

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

	async function startOAuthLink() {
		oauthError = '';
		oauthSuccess = '';
		callbackCode = '';
		oauthStep = 'submitting';
		try {
			const result = await api.accounts.startAuth(account.id);
			oauthStateParam = result.state;
			await openAuthWindow(result.authorization_url);
			oauthStep = 'waiting';
		} catch (e) {
			oauthError = e instanceof Error ? e.message : 'Failed to start OAuth flow';
			oauthStep = 'idle';
		}
	}

	/** Open auth URL in an isolated Tauri webview (no shared browser cookies)
	 *  so X doesn't pre-fill a previously logged-in account. The Rust side
	 *  intercepts the callback redirect, extracts the code, and emits an
	 *  `oauth-callback` event. Falls back to window.open outside Tauri. */
	async function openAuthWindow(url: string) {
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const { listen } = await import('@tauri-apps/api/event');

			const unlisten = await listen<{ code: string; state: string }>(
				'oauth-callback',
				async (event) => {
					unlisten();
					const { code, state } = event.payload;
					if (code && state === oauthStateParam) {
						callbackCode = code;
						await completeOAuthLink();
					}
				}
			);

			await invoke('open_oauth_window', { url });
		} catch {
			// Not running in Tauri — fall back to default browser.
			window.open(url, '_blank', 'noopener');
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
</script>

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
				<button class="action-btn danger" onclick={handleUnlink} disabled={unlinking}>
					{#if unlinking}<Loader2 size={12} class="spinning" />{:else}<Unlink size={12} />{/if}
					Unlink
				</button>
			{/if}
		</div>
	{:else if oauthStep === 'waiting'}
		<div class="oauth-callback">
			<p class="oauth-hint">
				<ExternalLink size={12} />
				Authorize in the popup window, then paste the code below.
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

	{#if oauthError}<p class="cred-msg error">{oauthError}</p>{/if}
	{#if oauthSuccess}<p class="cred-msg success"><Check size={12} />{oauthSuccess}</p>{/if}
	{#if unlinkError}<p class="cred-msg error">{unlinkError}</p>{/if}
</div>

<style>
	.cred-section { display: flex; flex-direction: column; gap: 8px; }
	.cred-section-header {
		display: flex; align-items: center; gap: 6px;
		color: var(--color-text-muted); font-size: 11px; font-weight: 600;
		text-transform: uppercase; letter-spacing: 0.03em;
	}
	.cred-section-label { flex: 1; }
	.cred-status { font-size: 11px; font-weight: 500; text-transform: none; letter-spacing: 0; }
	.cred-status-linked { color: var(--color-success); }
	.cred-status-expired { color: var(--color-warning); }
	.cred-status-none { color: var(--color-text-subtle); }

	.cred-actions { display: flex; gap: 6px; flex-wrap: wrap; }

	.action-btn {
		display: inline-flex; align-items: center; gap: 4px;
		padding: 4px 10px; font-size: 11px; font-weight: 500;
		border: 1px solid var(--color-border); border-radius: 5px;
		background: var(--color-surface); color: var(--color-text);
		cursor: pointer; transition: background 0.15s, border-color 0.15s, color 0.15s;
		white-space: nowrap;
	}
	.action-btn:hover:not(:disabled) { background: var(--color-surface-hover); }
	.action-btn:disabled { opacity: 0.4; cursor: not-allowed; }
	.action-btn.primary { background: var(--color-accent); border-color: var(--color-accent); color: white; }
	.action-btn.primary:hover:not(:disabled) { opacity: 0.9; background: var(--color-accent); }
	.action-btn.danger { color: var(--color-danger); border-color: color-mix(in srgb, var(--color-danger) 30%, transparent); }
	.action-btn.danger:hover:not(:disabled) { background: color-mix(in srgb, var(--color-danger) 10%, transparent); }

	.oauth-callback { display: flex; flex-direction: column; gap: 8px; }
	.oauth-hint {
		display: flex; align-items: center; gap: 6px;
		font-size: 11px; color: var(--color-text-muted); margin: 0; line-height: 1.4;
	}
	.oauth-code-row { display: flex; gap: 6px; align-items: center; }
	.oauth-code-input {
		flex: 1; min-width: 0; padding: 5px 8px; font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-surface); border: 1px solid var(--color-border);
		border-radius: 5px; color: var(--color-text);
	}
	.oauth-code-input:focus { outline: none; border-color: var(--color-accent); }

	.cred-loading {
		display: flex; align-items: center; gap: 8px;
		font-size: 12px; color: var(--color-text-muted); padding: 4px 0;
	}

	.cred-msg { display: flex; align-items: center; gap: 4px; font-size: 11px; margin: 0; line-height: 1.4; }
	.cred-msg.error { color: var(--color-danger); }
	.cred-msg.success { color: var(--color-success); }

	:global(.spinning) { animation: spin 1s linear infinite; }
	@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
