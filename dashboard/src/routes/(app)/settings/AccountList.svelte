<script lang="ts">
	import {
		User,
		RefreshCw,
		Pencil,
		Trash2,
		X,
		Loader2,
		ShieldCheck,
		AlertCircle,
	} from "lucide-svelte";
	import {
		accounts,
		currentAccountId,
		archiveAccount,
		syncAccountProfile,
		type Account,
	} from "$lib/stores/accounts";
	import type { AccountAuthStatus } from "$lib/api";
	import ProfileEditForm from "./ProfileEditForm.svelte";

	interface Props {
		authStatuses: Map<string, AccountAuthStatus>;
		statusesLoading: boolean;
		onStatusChange: () => void;
	}

	const { authStatuses, statusesLoading, onStatusChange }: Props = $props();

	const DEFAULT_ACCOUNT_ID = "00000000-0000-0000-0000-000000000000";
	const ARCHIVE_PHRASE = "ARCHIVE";

	let editingId = $state<string | null>(null);
	let archivingId = $state<string | null>(null);
	let confirmArchiveText = $state("");
	let archiving = $state(false);
	let archiveError = $state("");
	let syncingId = $state<string | null>(null);

	const activeAccounts = $derived(
		$accounts.filter((a: Account) => a.status === "active"),
	);

	function isDefault(id: string): boolean {
		return id === DEFAULT_ACCOUNT_ID;
	}
	function isActive(id: string): boolean {
		return id === $currentAccountId;
	}
	function credentialLabel(status: AccountAuthStatus | undefined): { text: string; color: string } {
		if (!status || !status.has_credentials) return { text: "No credentials", color: "warning" };
		if (status.oauth_expired) return { text: "Token expired", color: "warning" };
		return { text: "Linked", color: "success" };
	}

	function startArchive(id: string) {
		archivingId = id;
		confirmArchiveText = "";
		archiveError = "";
	}
	function cancelArchive() {
		archivingId = null;
		confirmArchiveText = "";
		archiveError = "";
	}
	async function handleArchive() {
		if (!archivingId || archiving || confirmArchiveText !== ARCHIVE_PHRASE) return;
		archiving = true;
		archiveError = "";
		try {
			await archiveAccount(archivingId);
			archivingId = null;
			confirmArchiveText = "";
			onStatusChange();
		} catch (e) {
			archiveError = e instanceof Error ? e.message : "Failed to archive account";
		} finally {
			archiving = false;
		}
	}
	async function handleSync(id: string) {
		if (syncingId) return;
		syncingId = id;
		try {
			await syncAccountProfile(id);
		} catch {
			// Non-critical
		} finally {
			syncingId = null;
		}
	}
</script>

<div class="account-list">
	{#each activeAccounts as account (account.id)}
		{@const status = authStatuses.get(account.id)}
		{@const cred = credentialLabel(status)}
		<div
			class="account-row"
			class:editing={editingId === account.id}
			class:archiving-row={archivingId === account.id}
		>
			<div class="account-avatar">
				{#if account.x_avatar_url}
					<img src={account.x_avatar_url} alt={account.x_username ?? account.label} />
				{:else}
					<User size={18} />
				{/if}
			</div>

			<div class="account-info">
				{#if editingId === account.id}
					<ProfileEditForm
						accountId={account.id}
						initialLabel={account.label}
						onConfirm={() => { editingId = null; }}
						onCancel={() => { editingId = null; }}
					/>
				{:else}
					<div class="account-identity">
						<span class="account-name">
							{account.x_username ? `@${account.x_username}` : account.label}
						</span>
						{#if account.x_username && account.label !== account.x_username}
							<span class="account-label-tag">{account.label}</span>
						{/if}
					</div>
					<div class="account-meta">
						{#if isDefault(account.id)}<span class="badge badge-default">Default</span>{/if}
						{#if isActive(account.id)}<span class="badge badge-active">Active</span>{/if}
						{#if !statusesLoading}
							<span class="badge badge-{cred.color}">
								{#if cred.color === "success"}<ShieldCheck size={10} />{:else}<AlertCircle size={10} />{/if}
								{cred.text}
							</span>
						{/if}
					</div>
				{/if}
			</div>

			{#if editingId !== account.id && archivingId !== account.id}
				<div class="account-actions">
					<button class="icon-btn" onclick={() => handleSync(account.id)} disabled={syncingId === account.id} title="Sync X profile">
						<RefreshCw size={14} class={syncingId === account.id ? "spinning" : ""} />
					</button>
					<button class="icon-btn" onclick={() => { editingId = account.id; }} title="Rename">
						<Pencil size={14} />
					</button>
					{#if !isDefault(account.id) && !isActive(account.id)}
						<button class="icon-btn danger" onclick={() => startArchive(account.id)} title="Archive account">
							<Trash2 size={14} />
						</button>
					{/if}
				</div>
			{/if}

			{#if archivingId === account.id}
				<div class="archive-confirm">
					<p class="archive-prompt">Type <code>{ARCHIVE_PHRASE}</code> to confirm</p>
					<div class="archive-row">
						<input class="archive-input" type="text" placeholder={ARCHIVE_PHRASE}
							bind:value={confirmArchiveText} disabled={archiving}
							autocomplete="off" spellcheck="false" />
						<button class="archive-btn" onclick={handleArchive}
							disabled={confirmArchiveText !== ARCHIVE_PHRASE || archiving}>
							{#if archiving}<Loader2 size={14} class="spinning" />{:else}<Trash2 size={14} />{/if}
							Archive
						</button>
						<button class="icon-btn cancel" onclick={cancelArchive} disabled={archiving}>
							<X size={14} />
						</button>
					</div>
					{#if archiveError}<p class="inline-error">{archiveError}</p>{/if}
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.account-list { display: flex; flex-direction: column; gap: 2px; margin-bottom: 16px; }

	.account-row {
		display: flex; align-items: center; gap: 12px; padding: 10px 12px;
		border-radius: 6px; background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		flex-wrap: wrap; transition: border-color 0.15s;
	}
	.account-row:hover { border-color: var(--color-border); }
	.account-row.editing, .account-row.archiving-row { border-color: var(--color-border); }

	.account-avatar {
		display: flex; align-items: center; justify-content: center;
		width: 32px; height: 32px; border-radius: 50%;
		background: var(--color-surface-active); color: var(--color-text-muted);
		flex-shrink: 0; overflow: hidden;
	}
	.account-avatar img { width: 100%; height: 100%; object-fit: cover; }

	.account-info { flex: 1; min-width: 0; }
	.account-identity { display: flex; align-items: center; gap: 8px; }
	.account-name {
		font-size: 13px; font-weight: 600; color: var(--color-text);
		overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
	}
	.account-label-tag {
		font-size: 11px; color: var(--color-text-subtle);
		padding: 1px 6px; background: var(--color-surface-hover);
		border-radius: 4px; white-space: nowrap;
	}
	.account-meta { display: flex; align-items: center; gap: 6px; margin-top: 3px; }

	.badge {
		display: inline-flex; align-items: center; gap: 3px;
		font-size: 10px; font-weight: 600; text-transform: uppercase;
		letter-spacing: 0.03em; padding: 2px 6px; border-radius: 4px;
	}
	.badge-default { background: color-mix(in srgb, var(--color-accent) 12%, transparent); color: var(--color-accent); }
	.badge-active, .badge-success { background: color-mix(in srgb, var(--color-success) 12%, transparent); color: var(--color-success); }
	.badge-warning { background: color-mix(in srgb, var(--color-warning) 12%, transparent); color: var(--color-warning); }

	.account-actions { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }

	.icon-btn {
		display: inline-flex; align-items: center; justify-content: center;
		width: 28px; height: 28px; border: none; border-radius: 5px;
		background: transparent; color: var(--color-text-muted);
		cursor: pointer; transition: background 0.15s, color 0.15s;
	}
	.icon-btn:hover:not(:disabled) { background: var(--color-surface-hover); color: var(--color-text); }
	.icon-btn.danger:hover:not(:disabled) { background: color-mix(in srgb, var(--color-danger) 12%, transparent); color: var(--color-danger); }
	.icon-btn.cancel:hover:not(:disabled) { background: var(--color-surface-hover); color: var(--color-text); }
	.icon-btn:disabled { opacity: 0.4; cursor: not-allowed; }

	.archive-confirm { width: 100%; padding-top: 8px; }
	.archive-prompt { font-size: 12px; color: var(--color-text-muted); margin: 0 0 6px; }
	.archive-prompt code {
		padding: 1px 5px;
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		border-radius: 3px; font-size: 11px;
		font-family: var(--font-mono, ui-monospace, monospace);
		color: var(--color-danger);
	}
	.archive-row { display: flex; align-items: center; gap: 6px; }
	.archive-input {
		padding: 5px 8px; font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-surface); border: 1px solid var(--color-border);
		border-radius: 5px; color: var(--color-text); width: 120px;
	}
	.archive-input:focus { outline: none; border-color: var(--color-danger); }
	.archive-btn {
		display: inline-flex; align-items: center; gap: 4px;
		padding: 5px 10px; font-size: 12px; font-weight: 500;
		color: white; background: var(--color-danger); border: none;
		border-radius: 5px; cursor: pointer; transition: opacity 0.15s; white-space: nowrap;
	}
	.archive-btn:hover:not(:disabled) { opacity: 0.9; }
	.archive-btn:disabled { opacity: 0.4; cursor: not-allowed; }

	.inline-error { font-size: 12px; color: var(--color-danger); margin: 6px 0 0; }

	:global(.spinning) { animation: spin 1s linear infinite; }
	@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
