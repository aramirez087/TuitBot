<script lang="ts">
	import { onMount } from 'svelte';
	import { Users, User, RefreshCw, Pencil, Trash2, Plus, Check, X, Loader2, ShieldCheck, AlertCircle } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import { api } from '$lib/api';
	import type { AccountAuthStatus } from '$lib/api';
	import {
		accounts,
		currentAccountId,
		createAccount,
		renameAccount,
		archiveAccount,
		syncAccountProfile,
		type Account
	} from '$lib/stores/accounts';

	const DEFAULT_ACCOUNT_ID = '00000000-0000-0000-0000-000000000000';
	const ARCHIVE_PHRASE = 'ARCHIVE';

	// --- Create state ---
	let newLabel = $state('');
	let creating = $state(false);
	let createError = $state('');

	// --- Rename state ---
	let editingId = $state<string | null>(null);
	let editLabel = $state('');
	let renaming = $state(false);
	let renameError = $state('');

	// --- Archive state ---
	let archivingId = $state<string | null>(null);
	let confirmArchiveText = $state('');
	let archiving = $state(false);
	let archiveError = $state('');

	// --- Sync state ---
	let syncingId = $state<string | null>(null);

	// --- Auth status ---
	let authStatuses = $state<Map<string, AccountAuthStatus>>(new Map());
	let statusesLoading = $state(true);

	const activeAccounts = $derived(
		$accounts.filter((a: Account) => a.status === 'active')
	);

	const hasOnlyDefault = $derived(
		activeAccounts.length <= 1 && activeAccounts.every((a: Account) => a.id === DEFAULT_ACCOUNT_ID)
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

	// --- Create ---

	async function handleCreate() {
		const label = newLabel.trim();
		if (!label || creating) return;
		creating = true;
		createError = '';
		try {
			await createAccount(label);
			newLabel = '';
			await loadAuthStatuses();
		} catch (e) {
			createError = e instanceof Error ? e.message : 'Failed to create account';
		} finally {
			creating = false;
		}
	}

	// --- Rename ---

	function startRename(account: Account) {
		editingId = account.id;
		editLabel = account.label;
		renameError = '';
	}

	function cancelRename() {
		editingId = null;
		editLabel = '';
		renameError = '';
	}

	async function handleRename() {
		if (!editingId || renaming) return;
		const label = editLabel.trim();
		if (!label) return;
		renaming = true;
		renameError = '';
		try {
			await renameAccount(editingId, label);
			editingId = null;
			editLabel = '';
		} catch (e) {
			renameError = e instanceof Error ? e.message : 'Failed to rename account';
		} finally {
			renaming = false;
		}
	}

	function handleRenameKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') handleRename();
		if (e.key === 'Escape') cancelRename();
	}

	// --- Archive ---

	function startArchive(id: string) {
		archivingId = id;
		confirmArchiveText = '';
		archiveError = '';
	}

	function cancelArchive() {
		archivingId = null;
		confirmArchiveText = '';
		archiveError = '';
	}

	async function handleArchive() {
		if (!archivingId || archiving || confirmArchiveText !== ARCHIVE_PHRASE) return;
		archiving = true;
		archiveError = '';
		try {
			await archiveAccount(archivingId);
			archivingId = null;
			confirmArchiveText = '';
			await loadAuthStatuses();
		} catch (e) {
			archiveError = e instanceof Error ? e.message : 'Failed to archive account';
		} finally {
			archiving = false;
		}
	}

	// --- Sync profile ---

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

	// --- Helpers ---

	function isDefault(id: string): boolean {
		return id === DEFAULT_ACCOUNT_ID;
	}

	function isActive(id: string): boolean {
		return id === $currentAccountId;
	}

	function credentialLabel(status: AccountAuthStatus | undefined): { text: string; color: string } {
		if (!status || !status.has_credentials) return { text: 'No credentials', color: 'warning' };
		if (status.oauth_expired) return { text: 'Token expired', color: 'warning' };
		return { text: 'Linked', color: 'success' };
	}
</script>

<SettingsSection
	id="accounts"
	title="Accounts"
	description="Manage your X accounts and credentials"
	icon={Users}
>
	{#if hasOnlyDefault && !statusesLoading}
		<!-- Empty state -->
		<div class="empty-state">
			<div class="empty-icon">
				<Users size={28} />
			</div>
			<p class="empty-title">Add your first X account</p>
			<p class="empty-desc">
				Create an account entry, then link your X credentials to start automating.
			</p>
		</div>
	{/if}

	<!-- Account roster -->
	<div class="account-list">
		{#each activeAccounts as account (account.id)}
			{@const status = authStatuses.get(account.id)}
			{@const cred = credentialLabel(status)}
			<div class="account-row" class:editing={editingId === account.id} class:archiving-row={archivingId === account.id}>
				<!-- Avatar -->
				<div class="account-avatar">
					{#if account.x_avatar_url}
						<img src={account.x_avatar_url} alt={account.x_username ?? account.label} />
					{:else}
						<User size={18} />
					{/if}
				</div>

				<!-- Info -->
				<div class="account-info">
					{#if editingId === account.id}
						<div class="rename-row">
							<input
								class="rename-input"
								type="text"
								bind:value={editLabel}
								onkeydown={handleRenameKeydown}
								disabled={renaming}
								autofocus
							/>
							<button class="icon-btn confirm" onclick={handleRename} disabled={renaming || !editLabel.trim()} title="Save">
								{#if renaming}
									<Loader2 size={14} class="spinning" />
								{:else}
									<Check size={14} />
								{/if}
							</button>
							<button class="icon-btn cancel" onclick={cancelRename} disabled={renaming} title="Cancel">
								<X size={14} />
							</button>
						</div>
						{#if renameError}
							<p class="inline-error">{renameError}</p>
						{/if}
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
							{#if isDefault(account.id)}
								<span class="badge badge-default">Default</span>
							{/if}
							{#if isActive(account.id)}
								<span class="badge badge-active">Active</span>
							{/if}
							{#if !statusesLoading}
								<span class="badge badge-{cred.color}">
									{#if cred.color === 'success'}
										<ShieldCheck size={10} />
									{:else}
										<AlertCircle size={10} />
									{/if}
									{cred.text}
								</span>
							{/if}
						</div>
					{/if}
				</div>

				<!-- Actions -->
				{#if editingId !== account.id && archivingId !== account.id}
					<div class="account-actions">
						<button
							class="icon-btn"
							onclick={() => handleSync(account.id)}
							disabled={syncingId === account.id}
							title="Sync X profile"
						>
							<RefreshCw size={14} class={syncingId === account.id ? 'spinning' : ''} />
						</button>
						<button
							class="icon-btn"
							onclick={() => startRename(account)}
							title="Rename"
						>
							<Pencil size={14} />
						</button>
						{#if !isDefault(account.id) && !isActive(account.id)}
							<button
								class="icon-btn danger"
								onclick={() => startArchive(account.id)}
								title="Archive account"
							>
								<Trash2 size={14} />
							</button>
						{/if}
					</div>
				{/if}

				<!-- Archive confirmation -->
				{#if archivingId === account.id}
					<div class="archive-confirm">
						<p class="archive-prompt">
							Type <code>{ARCHIVE_PHRASE}</code> to confirm
						</p>
						<div class="archive-row">
							<input
								class="archive-input"
								type="text"
								placeholder={ARCHIVE_PHRASE}
								bind:value={confirmArchiveText}
								disabled={archiving}
								autocomplete="off"
								spellcheck="false"
							/>
							<button
								class="archive-btn"
								onclick={handleArchive}
								disabled={confirmArchiveText !== ARCHIVE_PHRASE || archiving}
							>
								{#if archiving}
									<Loader2 size={14} class="spinning" />
								{:else}
									<Trash2 size={14} />
								{/if}
								Archive
							</button>
							<button class="icon-btn cancel" onclick={cancelArchive} disabled={archiving}>
								<X size={14} />
							</button>
						</div>
						{#if archiveError}
							<p class="inline-error">{archiveError}</p>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
	</div>

	<!-- Create form -->
	<div class="create-form">
		<div class="create-row">
			<input
				class="create-input"
				type="text"
				placeholder="Account label (e.g. My Brand)"
				bind:value={newLabel}
				disabled={creating}
				onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handleCreate(); }}
			/>
			<button
				class="create-btn"
				onclick={handleCreate}
				disabled={creating || !newLabel.trim()}
			>
				{#if creating}
					<Loader2 size={14} class="spinning" />
					Creating...
				{:else}
					<Plus size={14} />
					Add Account
				{/if}
			</button>
		</div>
		{#if createError}
			<p class="inline-error">{createError}</p>
		{/if}
	</div>

	<!-- Credential status detail -->
	{#if !statusesLoading && activeAccounts.length > 0}
		<div class="cred-detail">
			<h3 class="cred-title">Credential Status</h3>
			<div class="cred-grid">
				{#each activeAccounts as account (account.id)}
					{@const status = authStatuses.get(account.id)}
					<div class="cred-row">
						<span class="cred-account">{account.x_username ? `@${account.x_username}` : account.label}</span>
						<div class="cred-badges">
							<span class="cred-badge" class:linked={status?.oauth_linked && !status?.oauth_expired} class:expired={status?.oauth_expired} class:none={!status?.oauth_linked}>
								OAuth: {status?.oauth_linked ? (status?.oauth_expired ? 'expired' : 'linked') : 'none'}
							</span>
							<span class="cred-badge" class:linked={status?.scraper_linked} class:none={!status?.scraper_linked}>
								Scraper: {status?.scraper_linked ? 'linked' : 'none'}
							</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</SettingsSection>

<style>
	/* Empty state */
	.empty-state {
		text-align: center;
		padding: 20px 0 24px;
	}

	.empty-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
		margin-bottom: 12px;
	}

	.empty-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.empty-desc {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
		max-width: 320px;
		margin-inline: auto;
		line-height: 1.5;
	}

	/* Account list */
	.account-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-bottom: 16px;
	}

	.account-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		border-radius: 6px;
		background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		flex-wrap: wrap;
		transition: border-color 0.15s;
	}

	.account-row:hover {
		border-color: var(--color-border);
	}

	.account-row.editing,
	.account-row.archiving-row {
		border-color: var(--color-border);
	}

	/* Avatar */
	.account-avatar {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		background: var(--color-surface-active);
		color: var(--color-text-muted);
		flex-shrink: 0;
		overflow: hidden;
	}

	.account-avatar img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	/* Info */
	.account-info {
		flex: 1;
		min-width: 0;
	}

	.account-identity {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.account-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.account-label-tag {
		font-size: 11px;
		color: var(--color-text-subtle);
		padding: 1px 6px;
		background: var(--color-surface-hover);
		border-radius: 4px;
		white-space: nowrap;
	}

	.account-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 3px;
	}

	/* Badges */
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.03em;
		padding: 2px 6px;
		border-radius: 4px;
	}

	.badge-default {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.badge-active {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.badge-success {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.badge-warning {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	/* Actions */
	.account-actions {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-shrink: 0;
	}

	.icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: background 0.15s, color 0.15s;
	}

	.icon-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.icon-btn.danger:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	.icon-btn.confirm:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.icon-btn.cancel:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	/* Rename */
	.rename-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.rename-input {
		flex: 1;
		padding: 4px 8px;
		font-size: 13px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 5px;
		color: var(--color-text);
		min-width: 0;
	}

	.rename-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	/* Archive confirm */
	.archive-confirm {
		width: 100%;
		padding-top: 8px;
	}

	.archive-prompt {
		font-size: 12px;
		color: var(--color-text-muted);
		margin: 0 0 6px;
	}

	.archive-prompt code {
		padding: 1px 5px;
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		border-radius: 3px;
		font-size: 11px;
		font-family: var(--font-mono, ui-monospace, monospace);
		color: var(--color-danger);
	}

	.archive-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.archive-input {
		padding: 5px 8px;
		font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 5px;
		color: var(--color-text);
		width: 120px;
	}

	.archive-input:focus {
		outline: none;
		border-color: var(--color-danger);
	}

	.archive-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 5px 10px;
		font-size: 12px;
		font-weight: 500;
		color: white;
		background: var(--color-danger);
		border: none;
		border-radius: 5px;
		cursor: pointer;
		transition: opacity 0.15s;
		white-space: nowrap;
	}

	.archive-btn:hover:not(:disabled) {
		opacity: 0.9;
	}

	.archive-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	/* Create form */
	.create-form {
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.create-row {
		display: flex;
		gap: 8px;
	}

	.create-input {
		flex: 1;
		padding: 8px 12px;
		font-size: 13px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
	}

	.create-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.create-input::placeholder {
		color: var(--color-text-subtle);
	}

	.create-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		font-size: 13px;
		font-weight: 500;
		color: white;
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		cursor: pointer;
		transition: opacity 0.15s;
		white-space: nowrap;
	}

	.create-btn:hover:not(:disabled) {
		opacity: 0.9;
	}

	.create-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	/* Inline error */
	.inline-error {
		font-size: 12px;
		color: var(--color-danger);
		margin: 6px 0 0;
	}

	/* Credential detail */
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

	.cred-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 10px;
		background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		border-radius: 5px;
		font-size: 12px;
	}

	.cred-account {
		color: var(--color-text);
		font-weight: 500;
	}

	.cred-badges {
		display: flex;
		gap: 8px;
	}

	.cred-badge {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.cred-badge.linked {
		color: var(--color-success);
	}

	.cred-badge.expired {
		color: var(--color-warning);
	}

	.cred-badge.none {
		color: var(--color-text-subtle);
	}

	/* Spinner animation */
	:global(.spinning) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
