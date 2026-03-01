<script lang="ts">
	import { Cloud, Check, AlertTriangle, Loader2, Unlink } from 'lucide-svelte';
	import {
		activeGoogleDrive,
		expiredGoogleDrive,
		linkingState,
		linkError,
		startLink,
		disconnectConnection,
		loadConnections,
		connectionsLoaded
	} from '$lib/stores/connectors';
	import { capabilities } from '$lib/stores/runtime';
	import { onMount } from 'svelte';

	interface Props {
		onconnected?: (connectionId: number, email: string) => void;
		ondisconnected?: () => void;
	}

	let { onconnected, ondisconnected }: Props = $props();
	let disconnecting = $state(false);

	const driveConfigured = $derived($capabilities?.google_drive ?? true);

	onMount(() => {
		if (!$connectionsLoaded) {
			loadConnections();
		}
	});

	async function handleConnect(force?: boolean) {
		const id = await startLink(force);
		if (id != null) {
			const email = $activeGoogleDrive?.account_email ?? 'unknown';
			onconnected?.(id, email);
		}
	}

	async function handleDisconnect() {
		const conn = $activeGoogleDrive ?? $expiredGoogleDrive;
		if (!conn) return;
		disconnecting = true;
		await disconnectConnection(conn.id);
		disconnecting = false;
		ondisconnected?.();
	}
</script>

<div class="drive-card">
	{#if !driveConfigured}
		<!-- No GCP credentials configured on server -->
		<div class="card-state card-unconfigured">
			<Cloud size={20} />
			<div class="card-content">
				<span class="card-title">Google Drive connector not configured</span>
				<span class="card-hint">
					The server needs Google OAuth credentials to enable Drive linking.
					See the deployment docs for setup instructions.
				</span>
			</div>
		</div>
	{:else if $linkingState === 'linking'}
		<!-- OAuth flow in progress -->
		<div class="card-state card-linking">
			<span class="spinner"><Loader2 size={20} /></span>
			<div class="card-content">
				<span class="card-title">Waiting for Google authorization...</span>
				<span class="card-hint">
					Complete the sign-in in the popup window.
				</span>
			</div>
			<button
				type="button"
				class="card-btn card-btn-secondary"
				onclick={() => linkingState.set('idle')}
			>
				Cancel
			</button>
		</div>
	{:else if $activeGoogleDrive}
		<!-- Connected and active -->
		<div class="card-state card-connected">
			<Check size={20} />
			<div class="card-content">
				<span class="card-title">
					{$activeGoogleDrive.account_email ?? 'Google Drive'}
				</span>
				<span class="card-badge badge-active">Connected</span>
			</div>
			<button
				type="button"
				class="card-btn card-btn-danger"
				onclick={handleDisconnect}
				disabled={disconnecting}
			>
				{#if disconnecting}
					<span class="spinner"><Loader2 size={14} /></span>
				{:else}
					<Unlink size={14} />
				{/if}
				Disconnect
			</button>
		</div>
	{:else if $expiredGoogleDrive}
		<!-- Connection expired / revoked -->
		<div class="card-state card-expired">
			<AlertTriangle size={20} />
			<div class="card-content">
				<span class="card-title">
					{$expiredGoogleDrive.account_email ?? 'Google Drive'}
				</span>
				<span class="card-badge badge-expired">Expired</span>
				<span class="card-hint">
					Authorization has expired or been revoked. Reconnect to resume syncing.
				</span>
			</div>
			<button
				type="button"
				class="card-btn card-btn-primary"
				onclick={() => handleConnect(true)}
			>
				Reconnect
			</button>
		</div>
	{:else if $linkingState === 'error'}
		<!-- Error state -->
		<div class="card-state card-error">
			<AlertTriangle size={20} />
			<div class="card-content">
				<span class="card-title">Connection failed</span>
				{#if $linkError}
					<span class="card-hint card-hint-error">{$linkError}</span>
				{/if}
			</div>
			<button
				type="button"
				class="card-btn card-btn-primary"
				onclick={() => handleConnect()}
			>
				Try Again
			</button>
		</div>
	{:else}
		<!-- Idle -- no connection -->
		<div class="card-state card-idle">
			<Cloud size={20} />
			<div class="card-content">
				<span class="card-title">Connect Google Drive</span>
				<span class="card-hint">
					Link your Google account to sync content from Drive.
				</span>
			</div>
			<button
				type="button"
				class="card-btn card-btn-primary"
				onclick={() => handleConnect()}
			>
				<Cloud size={14} />
				Connect
			</button>
		</div>
	{/if}
</div>

<style>
	.drive-card {
		border: 1px solid var(--color-border);
		border-radius: 8px;
		overflow: hidden;
	}

	.card-state {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 14px 16px;
		color: var(--color-text-muted);
	}

	.card-connected {
		background: color-mix(in srgb, var(--color-success) 5%, transparent);
		border-color: color-mix(in srgb, var(--color-success) 20%, transparent);
		color: var(--color-success);
	}

	.card-expired {
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 5%, transparent);
		color: var(--color-warning, #f59e0b);
	}

	.card-error {
		background: color-mix(in srgb, var(--color-danger) 5%, transparent);
		color: var(--color-danger);
	}

	.card-unconfigured {
		background: color-mix(in srgb, var(--color-text-muted) 5%, transparent);
	}

	.card-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.card-title {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
	}

	.card-hint {
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.4;
	}

	.card-hint-error {
		color: var(--color-danger);
	}

	.card-badge {
		display: inline-block;
		padding: 1px 8px;
		border-radius: 10px;
		font-size: 11px;
		font-weight: 600;
		width: fit-content;
	}

	.badge-active {
		background: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.badge-expired {
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 15%, transparent);
		color: var(--color-warning, #f59e0b);
	}

	.card-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.15s;
		flex-shrink: 0;
	}

	.card-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.card-btn-primary {
		background: var(--color-accent);
		color: white;
	}

	.card-btn-primary:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.card-btn-secondary {
		background: var(--color-surface);
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
	}

	.card-btn-secondary:hover:not(:disabled) {
		background: var(--color-surface-hover);
	}

	.card-btn-danger {
		background: transparent;
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
	}

	.card-btn-danger:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border-color: var(--color-danger);
		color: var(--color-danger);
	}

	.spinner {
		display: inline-flex;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
