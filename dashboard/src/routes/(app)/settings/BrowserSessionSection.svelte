<script lang="ts">
	import { Globe, Trash2, CheckCircle, AlertCircle, Loader2 } from 'lucide-svelte';
	import { api } from '$lib/api';
	import { syncAccountProfile } from '$lib/stores/accounts';
	import { getAccountId } from '$lib/api/http';
	import { reloadCapabilities } from '$lib/stores/runtime';
	import { updateDraft } from '$lib/stores/settings';
	import { onMount } from 'svelte';

	let sessionExists = $state(false);
	let sessionUsername = $state<string | null>(null);
	let sessionCreatedAt = $state<string | null>(null);

	let authToken = $state('');
	let ct0 = $state('');
	let username = $state('');

	let importing = $state(false);
	let deleting = $state(false);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);
	let showForm = $state(false);

	onMount(async () => {
		try {
			const status = await api.settings.scraperSession.get();
			sessionExists = status.exists;
			sessionUsername = status.username ?? null;
			sessionCreatedAt = status.created_at ?? null;
		} catch {
			// Non-critical
		}
	});

	async function handleImport() {
		if (!authToken.trim() || !ct0.trim()) {
			error = 'Both auth_token and ct0 are required';
			return;
		}
		error = null;
		success = null;
		importing = true;
		try {
			const result = await api.settings.scraperSession.import({
				auth_token: authToken.trim(),
				ct0: ct0.trim(),
				username: username.trim() || undefined
			});
			sessionExists = true;
			sessionUsername = result.username ?? null;
			sessionCreatedAt = result.created_at ?? null;
			success = 'Browser session imported. You can now publish tweets directly.';
			showForm = false;
			authToken = '';
			ct0 = '';
			username = '';
			// Backend auto-sets provider_backend to "scraper" — sync the
			// settings draft so the UI reflects the change without a reload.
			if (result.backend_updated) {
				updateDraft('x_api.provider_backend', 'scraper');
			}
			// Refresh runtime status so canPost/capabilityTier update.
			reloadCapabilities();
			// Sync profile to pull avatar/username/display name from X.
			try {
				await syncAccountProfile(getAccountId());
			} catch (syncErr) {
				console.error('Profile sync failed:', syncErr);
				error = syncErr instanceof Error
					? `Session imported, but profile sync failed: ${syncErr.message}`
					: 'Session imported, but profile sync failed';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Import failed';
		} finally {
			importing = false;
		}
	}

	async function handleDelete() {
		error = null;
		success = null;
		deleting = true;
		try {
			await api.settings.scraperSession.delete();
			sessionExists = false;
			sessionUsername = null;
			sessionCreatedAt = null;
			success = 'Browser session removed.';
		} catch (e) {
			error = e instanceof Error ? e.message : 'Delete failed';
		} finally {
			deleting = false;
		}
	}

	function formatDate(iso: string): string {
		try {
			return new Date(iso).toLocaleDateString(undefined, {
				month: 'short',
				day: 'numeric',
				year: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return iso;
		}
	}
</script>

<div class="session-section">
	<div class="session-header">
		<Globe size={16} />
		<span class="session-title">Browser Session</span>
	</div>

	{#if error}
		<div class="message error">
			<AlertCircle size={14} />
			<span>{error}</span>
		</div>
	{/if}

	{#if success}
		<div class="message success">
			<CheckCircle size={14} />
			<span>{success}</span>
		</div>
	{/if}

	{#if sessionExists}
		<div class="session-status connected">
			<div class="status-info">
				<span class="status-dot active"></span>
				<span class="status-text">
					Session active{sessionUsername ? ` (@${sessionUsername})` : ''}
				</span>
				{#if sessionCreatedAt}
					<span class="status-date">Imported {formatDate(sessionCreatedAt)}</span>
				{/if}
			</div>
			<button
				type="button"
				class="delete-btn"
				onclick={handleDelete}
				disabled={deleting}
			>
				{#if deleting}
					<Loader2 size={14} class="spin" />
				{:else}
					<Trash2 size={14} />
				{/if}
				Remove
			</button>
		</div>
	{:else if !showForm}
		<p class="session-desc">
			Import your browser cookies to enable direct posting without API credentials.
		</p>
		<button type="button" class="import-btn" onclick={() => (showForm = true)}>
			Import Browser Session
		</button>
	{/if}

	{#if showForm && !sessionExists}
		<div class="import-form">
			<p class="form-instructions">
				Open X in your browser, then extract these cookies from DevTools
				(Application → Cookies → x.com):
			</p>
			<div class="form-field">
				<label class="field-label" for="session_auth_token">auth_token</label>
				<input
					id="session_auth_token"
					type="password"
					class="text-input"
					bind:value={authToken}
					placeholder="Paste auth_token cookie value"
				/>
			</div>
			<div class="form-field">
				<label class="field-label" for="session_ct0">ct0</label>
				<input
					id="session_ct0"
					type="password"
					class="text-input"
					bind:value={ct0}
					placeholder="Paste ct0 cookie value"
				/>
			</div>
			<div class="form-field">
				<label class="field-label" for="session_username">
					Username <span class="optional">(optional)</span>
				</label>
				<input
					id="session_username"
					type="text"
					class="text-input"
					bind:value={username}
					placeholder="@yourusername"
				/>
			</div>
			<div class="form-actions">
				<button
					type="button"
					class="import-btn primary"
					onclick={handleImport}
					disabled={importing || !authToken.trim() || !ct0.trim()}
				>
					{#if importing}
						<Loader2 size={14} class="spin" />
						Importing...
					{:else}
						Import Session
					{/if}
				</button>
				<button
					type="button"
					class="cancel-btn"
					onclick={() => (showForm = false)}
				>
					Cancel
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.session-section {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px 16px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
	}

	.session-header {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
	}

	.session-title {
		font-weight: 600;
	}

	.session-desc {
		margin: 0;
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.session-status {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.status-info {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-text-muted);
	}

	.status-dot.active {
		background: var(--color-success, #22c55e);
	}

	.status-text {
		font-size: 13px;
		color: var(--color-text);
	}

	.status-date {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.delete-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		font-size: 12px;
		border: 1px solid color-mix(in srgb, var(--color-danger, #ef4444) 30%, transparent);
		background: color-mix(in srgb, var(--color-danger, #ef4444) 8%, transparent);
		color: var(--color-danger, #ef4444);
		border-radius: 6px;
		cursor: pointer;
		transition: background 0.15s;
	}

	.delete-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-danger, #ef4444) 15%, transparent);
	}

	.import-btn {
		padding: 8px 14px;
		font-size: 13px;
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text);
		border-radius: 6px;
		cursor: pointer;
		transition: background 0.15s, border-color 0.15s;
		align-self: flex-start;
	}

	.import-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.import-btn.primary {
		background: var(--color-accent);
		color: var(--color-accent-text, #fff);
		border-color: var(--color-accent);
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.import-btn.primary:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.import-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.cancel-btn {
		padding: 8px 14px;
		font-size: 13px;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text-muted);
		border-radius: 6px;
		cursor: pointer;
	}

	.import-form {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.form-instructions {
		margin: 0;
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.form-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.field-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
	}

	.optional {
		font-weight: 400;
		color: var(--color-text-muted);
	}

	.text-input {
		padding: 8px 12px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-mono, monospace);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus {
		border-color: var(--color-accent);
	}

	.form-actions {
		display: flex;
		gap: 8px;
		margin-top: 4px;
	}

	.message {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 6px;
		font-size: 12px;
	}

	.message.error {
		background: color-mix(in srgb, var(--color-danger, #ef4444) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-danger, #ef4444) 20%, transparent);
		color: var(--color-danger, #ef4444);
	}

	.message.success {
		background: color-mix(in srgb, var(--color-success, #22c55e) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-success, #22c55e) 20%, transparent);
		color: var(--color-success, #22c55e);
	}

	:global(.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
