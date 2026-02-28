<script lang="ts">
	import { onMount } from 'svelte';
	import { Wifi, Copy, Check, RefreshCw } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import { api } from '$lib/api';

	let bindHost = $state('127.0.0.1');
	let bindPort = $state(3001);
	let lanEnabled = $state(false);
	let localIp = $state<string | null>(null);
	let passphraseConfigured = $state(false);

	let loading = $state(true);
	let toggling = $state(false);
	let resetting = $state(false);
	let restartRequired = $state(false);

	let revealedPassphrase = $state<string | null>(null);
	let copied = $state(false);
	let hideTimeout: ReturnType<typeof setTimeout> | null = null;
	let copyTimeout: ReturnType<typeof setTimeout> | null = null;

	onMount(() => {
		loadStatus();
		return () => {
			if (hideTimeout) clearTimeout(hideTimeout);
			if (copyTimeout) clearTimeout(copyTimeout);
		};
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

	async function handleResetPassphrase() {
		resetting = true;
		try {
			const result = await api.lan.resetPassphrase();
			revealedPassphrase = result.passphrase;
			passphraseConfigured = true;
			copied = false;

			if (hideTimeout) clearTimeout(hideTimeout);
			hideTimeout = setTimeout(() => {
				revealedPassphrase = null;
			}, 30000);
		} catch (e) {
			console.error('Failed to reset passphrase', e);
		}
		resetting = false;
	}

	async function copyPassphrase() {
		if (!revealedPassphrase) return;
		try {
			await navigator.clipboard.writeText(revealedPassphrase);
			copied = true;
			if (copyTimeout) clearTimeout(copyTimeout);
			copyTimeout = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			// Clipboard API not available.
		}
	}
</script>

<SettingsSection
	id="lan"
	title="LAN Access"
	description="Access the dashboard from any device on your network"
	icon={Wifi}
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
				<div class="notice">
					Requires server restart to take effect
				</div>
			{/if}
		</div>

		<!-- Passphrase Reset -->
		<div class="status-group">
			<h3 class="group-title">Passphrase</h3>

			{#if revealedPassphrase}
				<div class="passphrase-reveal">
					<code class="passphrase-text">{revealedPassphrase}</code>
					<button class="copy-btn" onclick={copyPassphrase} title="Copy passphrase">
						{#if copied}
							<Check size={14} />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>
				<span class="field-hint">
					Save this passphrase — it will be hidden in 30 seconds
				</span>
			{/if}

			<button
				class="reset-btn"
				onclick={handleResetPassphrase}
				disabled={resetting}
			>
				<RefreshCw size={14} class={resetting ? 'spinning' : ''} />
				{resetting ? 'Resetting...' : 'Reset Passphrase'}
			</button>
			<span class="field-hint">
				Generate a new passphrase for web/LAN login
			</span>
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

	/* Toggle */
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

	/* Notice */
	.notice {
		margin-top: 8px;
		font-size: 12px;
		color: var(--color-warning, #f59e0b);
		padding: 8px 12px;
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 25%, transparent);
		border-radius: 6px;
	}

	/* Passphrase */
	.passphrase-reveal {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
	}

	.passphrase-text {
		flex: 1;
		padding: 10px 14px;
		background: color-mix(in srgb, var(--color-accent) 8%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
		border-radius: 6px;
		font-size: 15px;
		font-family: var(--font-mono, ui-monospace, monospace);
		color: var(--color-text);
		letter-spacing: 0.02em;
	}

	.copy-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: border-color 0.15s, color 0.15s;
	}

	.copy-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.reset-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 8px;
		padding: 8px 14px;
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
	}

	.reset-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
	}

	.reset-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	:global(.spinning) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
