<script lang="ts">
	import { Globe, RefreshCw, Trash2, Loader2, Check, X, ClipboardPaste } from 'lucide-svelte';
	import { api } from '$lib/api';
	import type { AccountAuthStatus } from '$lib/api';
	import { type Account } from '$lib/stores/accounts';
	import { reloadCapabilities } from '$lib/stores/runtime';

	interface Props {
		account: Account;
		status: AccountAuthStatus | undefined;
		isActiveAccount: boolean;
		onStatusChange: () => void;
	}

	const { account, status, isActiveAccount, onStatusChange }: Props = $props();

	let showScraperForm = $state(false);
	let scraperAuthToken = $state('');
	let scraperCt0 = $state('');
	let scraperUsername = $state('');
	let importingScraper = $state(false);
	let scraperError = $state('');
	let scraperSuccess = $state('');
	let deletingScraper = $state(false);

	const scraperLabel = $derived(status?.scraper_linked ? 'Linked' : 'Not linked');
	const scraperColor = $derived(status?.scraper_linked ? 'linked' : 'none');

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

<div class="cred-section">
	<div class="cred-section-header">
		<Globe size={13} />
		<span class="cred-section-label">Browser Session</span>
		<span class="cred-status cred-status-{scraperColor}">{scraperLabel}</span>
	</div>

	{#if !showScraperForm}
		<div class="cred-actions">
			{#if !status?.scraper_linked}
				<button class="action-btn primary" onclick={() => (showScraperForm = true)}>
					<ClipboardPaste size={12} />
					Import Session
				</button>
			{:else}
				<button class="action-btn" onclick={() => (showScraperForm = true)}>
					<RefreshCw size={12} />
					Replace
				</button>
				<button class="action-btn danger" onclick={handleScraperDelete} disabled={deletingScraper}>
					{#if deletingScraper}<Loader2 size={12} class="spinning" />{:else}<Trash2 size={12} />{/if}
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
					{#if importingScraper}<Loader2 size={12} class="spinning" />{:else}<Check size={12} />{/if}
					Import
				</button>
				<button class="action-btn" onclick={cancelScraperForm} disabled={importingScraper}>
					<X size={12} />
					Cancel
				</button>
			</div>
		</div>
	{/if}

	{#if scraperError}<p class="cred-msg error">{scraperError}</p>{/if}
	{#if scraperSuccess}<p class="cred-msg success"><Check size={12} />{scraperSuccess}</p>{/if}
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

	.scraper-form { display: flex; flex-direction: column; gap: 6px; }
	.scraper-input {
		padding: 5px 8px; font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-surface); border: 1px solid var(--color-border);
		border-radius: 5px; color: var(--color-text);
	}
	.scraper-input:focus { outline: none; border-color: var(--color-accent); }
	.scraper-input::placeholder { color: var(--color-text-subtle); font-family: inherit; }
	.scraper-form-actions { display: flex; gap: 6px; margin-top: 2px; }

	.cred-msg { display: flex; align-items: center; gap: 4px; font-size: 11px; margin: 0; line-height: 1.4; }
	.cred-msg.error { color: var(--color-danger); }
	.cred-msg.success { color: var(--color-success); }

	:global(.spinning) { animation: spin 1s linear infinite; }
	@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
