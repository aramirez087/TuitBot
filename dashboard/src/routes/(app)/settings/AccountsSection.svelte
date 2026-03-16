<script lang="ts">
	import { onMount } from "svelte";
	import { Users } from "lucide-svelte";
	import SettingsSection from "$lib/components/settings/SettingsSection.svelte";
	import CredentialCard from "./CredentialCard.svelte";
	import { api } from "$lib/api";
	import type { AccountAuthStatus } from "$lib/api";
	import { accounts, type Account } from "$lib/stores/accounts";
	import AccountList from "./AccountList.svelte";
	import AddAccountModal from "./AddAccountModal.svelte";

	let authStatuses = $state<Map<string, AccountAuthStatus>>(new Map());
	let statusesLoading = $state(true);

	const activeAccounts = $derived(
		$accounts.filter((a: Account) => a.status === "active"),
	);

	onMount(() => {
		loadAuthStatuses();
	});

	async function loadAuthStatuses() {
		statusesLoading = true;
		const statuses = new Map<string, AccountAuthStatus>();
		const promises = $accounts.map(async (account: Account) => {
			try {
				const status = await api.accounts.authStatus(account.id);
				statuses.set(account.id, status);
			} catch {
				// Ignore — status will be absent
			}
		});
		await Promise.all(promises);
		authStatuses = statuses;
		statusesLoading = false;
	}
</script>

<SettingsSection
	id="accounts"
	title="Accounts"
	description="Manage your X accounts and credentials"
	icon={Users}
>
	<AccountList
		{authStatuses}
		{statusesLoading}
		onStatusChange={loadAuthStatuses}
	/>
	<AddAccountModal onCreated={loadAuthStatuses} />

	{#if !statusesLoading && activeAccounts.length > 0}
		<div class="cred-detail">
			<h3 class="cred-title">Credential Management</h3>
			<div class="cred-grid">
				{#each activeAccounts as account (account.id)}
					<CredentialCard
						{account}
						status={authStatuses.get(account.id)}
						onStatusChange={loadAuthStatuses}
					/>
				{/each}
			</div>
		</div>
	{/if}
</SettingsSection>

<style>
	.cred-detail {
		margin-top: 20px;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.cred-title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-subtle);
		margin: 0 0 10px;
	}

	.cred-grid {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
</style>
