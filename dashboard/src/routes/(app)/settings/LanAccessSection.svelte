<script lang="ts">
	import { onMount } from 'svelte';
	import { Wifi } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import { api } from '$lib/api';
	import LanPassphraseGroup from './LanPassphraseGroup.svelte';

	let bindHost = $state('127.0.0.1');
	let bindPort = $state(3001);
	let lanEnabled = $state(false);
	let localIp = $state<string | null>(null);
	let passphraseConfigured = $state(false);

	let loading = $state(true);
	let toggling = $state(false);
	let restartRequired = $state(false);

	onMount(() => {
		loadStatus();
	});

	async function loadStatus() {
		loading = true;
		try {
			const status = await api.lan.status();
			bindHost = status.bind_host;
			bindPort = status.bind_port;
			lanEnabled = status.lan_enabled;
			localIp = status.local_ip;
			passphraseConfigured = status.passphrase_configured;
		} catch (e) {
			console.error('Failed to load LAN status', e);
		}
		loading = false;
	}

	async function handleToggle() {
		toggling = true;
		try {
			const newHost = lanEnabled ? '127.0.0.1' : '0.0.0.0';
			await api.lan.toggle(newHost);
			lanEnabled = !lanEnabled;
			restartRequired = true;
		} catch (e) {
			console.error('Failed to toggle LAN mode', e);
		}
		toggling = false;
	}
</script>

<SettingsSection
	id="lan"
	title="LAN Access"
	description="Access the dashboard from any device on your network"
	icon={Wifi}
	scope="instance"
>
	{#if loading}
		<div class="loading-text">Loading...</div>
	{:else}
		<!-- Server Status -->
		<div class="status-group">
			<h3 class="group-title">Server Status</h3>
			<div class="status-table">
				<div class="status-row">
					<span class="status-label">Bind Address</span>
					<span class="status-value mono">{bindHost}:{bindPort}</span>
				</div>
				<div class="status-row">
					<span class="status-label">LAN IP</span>
					<span class="status-value mono">{localIp ?? 'unavailable'}</span>
				</div>
				<div class="status-row">
					<span class="status-label">Passphrase</span>
					<span class="status-value">
						{#if passphraseConfigured}
							<span class="status-badge configured">Configured</span>
						{:else}
							<span class="status-badge not-configured">Not configured</span>
						{/if}
					</span>
				</div>
			</div>
		</div>

		<!-- LAN Mode Toggle -->
		<div class="status-group">
			<h3 class="group-title">LAN Mode</h3>
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="field-label">Enable LAN Access</span>
					<span class="field-hint">
						Bind to 0.0.0.0 so other devices on your network can reach the dashboard
					</span>
				</div>
				<button
					type="button"
					class="toggle"
					class:active={lanEnabled}
					onclick={handleToggle}
					disabled={toggling}
					role="switch"
					aria-checked={lanEnabled}
					aria-label="Toggle LAN access"
				>
					<span class="toggle-track">
						<span class="toggle-thumb"></span>
					</span>
				</button>
			</div>
			{#if restartRequired}
				<div class="notice">Requires server restart to take effect</div>
			{/if}
		</div>

		<!-- Passphrase -->
		<div class="status-group">
			<h3 class="group-title">Passphrase</h3>
			<LanPassphraseGroup bind:passphraseConfigured />
		</div>
	{/if}
</SettingsSection>

<style>
	.loading-text {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.status-group {
		margin-bottom: 20px;
	}

	.status-group:last-child {
		margin-bottom: 0;
	}

	.group-title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-subtle);
		margin: 0 0 10px;
	}

	.status-table {
		background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		overflow: hidden;
	}

	.status-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 14px;
	}

	.status-row + .status-row {
		border-top: 1px solid var(--color-border-subtle);
	}

	.status-label {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.status-value {
		font-size: 13px;
		color: var(--color-text);
	}

	.mono {
		font-family: var(--font-mono, ui-monospace, monospace);
	}

	.status-badge {
		font-size: 12px;
		font-weight: 500;
		padding: 2px 8px;
		border-radius: 10px;
	}

	.status-badge.configured {
		background: color-mix(in srgb, var(--color-success, #22c55e) 15%, transparent);
		color: var(--color-success, #22c55e);
	}

	.status-badge.not-configured {
		background: color-mix(in srgb, var(--color-text-subtle) 15%, transparent);
		color: var(--color-text-subtle);
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.toggle {
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.toggle-track {
		display: flex;
		align-items: center;
		width: 42px;
		height: 24px;
		padding: 2px;
		background: var(--color-border);
		border-radius: 12px;
		transition: background 0.2s;
	}

	.toggle.active .toggle-track {
		background: var(--color-accent);
	}

	.toggle-thumb {
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.toggle.active .toggle-thumb {
		transform: translateX(18px);
	}

	.toggle:focus-visible .toggle-track {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.notice {
		margin-top: 8px;
		font-size: 12px;
		color: var(--color-warning, #f59e0b);
		padding: 8px 12px;
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 25%, transparent);
		border-radius: 6px;
	}
</style>
