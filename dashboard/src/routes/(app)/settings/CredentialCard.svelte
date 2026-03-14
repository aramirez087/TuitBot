<script lang="ts">
	import { ChevronDown, ChevronRight } from 'lucide-svelte';
	import type { AccountAuthStatus } from '$lib/api';
	import { getAccountId } from '$lib/api/http';
	import { type Account } from '$lib/stores/accounts';
	import XApiCredentialForm from './XApiCredentialForm.svelte';
	import ScraperSessionForm from './ScraperSessionForm.svelte';

	interface Props {
		account: Account;
		status: AccountAuthStatus | undefined;
		onStatusChange: () => void;
	}

	const { account, status, onStatusChange }: Props = $props();

	let expanded = $state(false);

	const isActiveAccount = $derived(account.id === getAccountId());

	const oauthColor = $derived.by(() => {
		if (!status?.oauth_linked) return 'none';
		if (status.oauth_expired) return 'expired';
		return 'linked';
	});

	const oauthLabel = $derived.by(() => {
		if (!status?.oauth_linked) return 'Not linked';
		if (status.oauth_expired) return 'Expired';
		if (status.oauth_expires_at) {
			return `Linked (expires ${new Date(status.oauth_expires_at).toLocaleDateString()})`;
		}
		return 'Linked';
	});

	const scraperColor = $derived(status?.scraper_linked ? 'linked' : 'none');
	const scraperLabel = $derived(status?.scraper_linked ? 'Linked' : 'Not linked');
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
		{#if expanded}<ChevronDown size={14} />{:else}<ChevronRight size={14} />{/if}
	</button>

	{#if expanded}
		<div class="cred-body">
			<XApiCredentialForm {account} {status} {isActiveAccount} {onStatusChange} />
			<ScraperSessionForm {account} {status} {isActiveAccount} {onStatusChange} />
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
	.cred-card:hover { border-color: var(--color-border); }

	.cred-header {
		display: flex; align-items: center; gap: 8px; width: 100%;
		padding: 8px 12px; background: transparent; border: none;
		cursor: pointer; color: var(--color-text-muted); font-size: 12px; text-align: left;
	}
	.cred-header:hover { background: var(--color-surface-hover); }

	.cred-account-name {
		font-weight: 600; font-size: 12px; color: var(--color-text);
		flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}

	.cred-summary { display: flex; gap: 4px; align-items: center; }

	.cred-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
	.cred-dot-linked { background: var(--color-success); }
	.cred-dot-expired { background: var(--color-warning); }
	.cred-dot-none { background: var(--color-text-subtle); opacity: 0.4; }

	.cred-body { padding: 0 12px 12px; display: flex; flex-direction: column; gap: 12px; }
</style>
