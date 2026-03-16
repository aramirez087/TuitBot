<script lang="ts">
	import { Globe, Trash2, CheckCircle, AlertCircle, Loader2 } from 'lucide-svelte';
	import { api } from '$lib/api';
	import { onMount } from 'svelte';
	import BrowserSessionImportForm from './BrowserSessionImportForm.svelte';

	let sessionExists = $state(false);
	let sessionUsername = $state<string | null>(null);
	let sessionCreatedAt = $state<string | null>(null);

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

	function handleImportSuccess(result: { username: string | null; created_at: string | null }) {
		sessionExists = true;
		sessionUsername = result.username;
		sessionCreatedAt = result.created_at;
		success = 'Browser session imported. You can now publish tweets directly.';
		showForm = false;
	}

	function formatDate(iso: string): string {
		try {
			return new Date(iso).toLocaleDateString(undefined, {
				month: 'short',
				day: 'numeric',
				year: 'numeric',
				hour: '2-digit',
				minute: '2-digit',
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
			<button type="button" class="delete-btn" onclick={handleDelete} disabled={deleting}>
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
		<BrowserSessionImportForm
			onImportSuccess={handleImportSuccess}
			onCancel={() => (showForm = false)}
		/>
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
