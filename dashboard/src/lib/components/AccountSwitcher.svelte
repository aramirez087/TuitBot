<script lang="ts">
	import { accounts, currentAccountId, switchAccount, type Account } from "$lib/stores/accounts";
	import { goto } from "$app/navigation";
	import { ChevronDown, User, Plus } from "lucide-svelte";

	let { collapsed = false }: { collapsed?: boolean } = $props();
	let open = $state(false);

	const current = $derived(
		$accounts.find((a: Account) => a.id === $currentAccountId)
	);

	const displayLabel = $derived(
		current?.x_username ? `@${current.x_username}` : current?.label ?? 'Default'
	);

	function toggle() {
		open = !open;
	}

	function select(id: string) {
		switchAccount(id);
		open = false;
	}

	function addAccount() {
		open = false;
		goto('/settings#accounts');
		setTimeout(() => document.getElementById('accounts')?.scrollIntoView({ behavior: 'smooth' }), 100);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			open = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="account-switcher" class:collapsed>
	<button
		class="account-trigger"
		onclick={toggle}
		title={collapsed ? displayLabel : undefined}
	>
		{#if current?.x_avatar_url}
			<img
				class="account-avatar"
				src={current.x_avatar_url}
				alt={displayLabel}
			/>
		{:else}
			<User size={collapsed ? 16 : 14} />
		{/if}
		{#if !collapsed}
			<span class="account-label">{displayLabel}</span>
			<ChevronDown size={12} />
		{/if}
	</button>

	{#if open}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="account-backdrop" onclick={() => (open = false)}></div>
		<div class="account-dropdown">
			{#each $accounts as account (account.id)}
				<button
					class="account-option"
					class:active={account.id === $currentAccountId}
					onclick={() => select(account.id)}
				>
					{#if account.x_avatar_url}
						<img
							class="account-avatar"
							src={account.x_avatar_url}
							alt={account.x_username ?? account.label}
						/>
					{:else}
						<User size={14} />
					{/if}
					<span>
						{account.x_username ? `@${account.x_username}` : account.label}
					</span>
				</button>
			{/each}
			<div class="dropdown-divider"></div>
			<button class="account-option add-account" onclick={addAccount}>
				<Plus size={14} />
				<span>Add Account</span>
			</button>
		</div>
	{/if}
</div>

<style>
	.account-switcher {
		position: relative;
		padding: 0 8px;
		margin-bottom: 4px;
	}

	.account-switcher.collapsed {
		padding: 0 4px;
	}

	.account-trigger {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		transition: background-color 0.15s ease, border-color 0.15s ease;
	}

	.collapsed .account-trigger {
		justify-content: center;
		padding: 6px;
	}

	.account-trigger:hover {
		background-color: var(--color-surface-hover);
		border-color: var(--color-border);
	}

	.account-avatar {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.account-label {
		flex: 1;
		text-align: left;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.account-backdrop {
		position: fixed;
		inset: 0;
		z-index: 99;
	}

	.account-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 4px;
		padding: 4px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		z-index: 100;
		min-width: 180px;
	}

	.account-option {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 10px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		transition: background-color 0.15s ease;
	}

	.account-option:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}

	.account-option.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.account-option.add-account {
		color: var(--color-accent);
	}

	.dropdown-divider {
		height: 1px;
		margin: 4px 6px;
		background: var(--color-border-subtle);
	}
</style>
