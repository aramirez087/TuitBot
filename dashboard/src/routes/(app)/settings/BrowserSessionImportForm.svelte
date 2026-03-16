<script lang="ts">
	import { Loader2, AlertCircle, CheckCircle } from 'lucide-svelte';
	import { api } from '$lib/api';
	import { syncAccountProfile } from '$lib/stores/accounts';
	import { getAccountId } from '$lib/api/http';
	import { reloadCapabilities } from '$lib/stores/runtime';
	import { updateDraft } from '$lib/stores/settings';

	interface ImportResult {
		username: string | null;
		created_at: string | null;
	}

	interface Props {
		onImportSuccess: (result: ImportResult) => void;
		onCancel: () => void;
	}

	const { onImportSuccess, onCancel }: Props = $props();

	let authToken = $state('');
	let ct0 = $state('');
	let username = $state('');
	let importing = $state(false);
	let error = $state<string | null>(null);
	let success = $state<string | null>(null);

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
				username: username.trim() || undefined,
			});
			if (result.backend_updated) {
				updateDraft('x_api.provider_backend', 'scraper');
			}
			reloadCapabilities();
			try {
				await syncAccountProfile(getAccountId());
			} catch (syncErr) {
				console.error('Profile sync failed:', syncErr);
				error =
					syncErr instanceof Error
						? `Session imported, but profile sync failed: ${syncErr.message}`
						: 'Session imported, but profile sync failed';
				importing = false;
				onImportSuccess({ username: result.username ?? null, created_at: result.created_at ?? null });
				return;
			}
			onImportSuccess({ username: result.username ?? null, created_at: result.created_at ?? null });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Import failed';
		} finally {
			importing = false;
		}
	}
</script>

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

<div class="import-form">
	<p class="form-instructions">
		Open X in your browser, then extract these cookies from DevTools (Application → Cookies →
		x.com):
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
		<button type="button" class="cancel-btn" onclick={onCancel}>Cancel</button>
	</div>
</div>

<style>
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

	.import-btn {
		padding: 8px 14px;
		font-size: 13px;
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text);
		border-radius: 6px;
		cursor: pointer;
		transition: background 0.15s, border-color 0.15s;
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
