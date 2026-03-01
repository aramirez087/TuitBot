<script lang="ts">
	import { AlertTriangle, Loader2 } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import { api } from '$lib/api';
	import { clearSession } from '$lib/stores/auth';
	import { resetStores } from '$lib/stores/settings';
	import { disconnectWs } from '$lib/stores/websocket';
	import { goto } from '$app/navigation';

	const CONFIRMATION_PHRASE = 'RESET TUITBOT';

	let confirmationText = $state('');
	let resetting = $state(false);
	let errorMsg = $state('');

	let canReset = $derived(confirmationText === CONFIRMATION_PHRASE && !resetting);

	async function handleReset() {
		resetting = true;
		errorMsg = '';
		try {
			await api.settings.factoryReset(confirmationText);
			clearSession();
			resetStores();
			disconnectWs();
			try {
				await goto('/onboarding');
			} catch {
				window.location.href = '/onboarding';
			}
		} catch (e) {
			errorMsg = e instanceof Error ? e.message : 'Factory reset failed';
			resetting = false;
		}
	}
</script>

<div class="danger-zone">
	<SettingsSection
		id="danger"
		title="Danger Zone"
		description="Irreversible actions that erase all Tuitbot data"
		icon={AlertTriangle}
	>
		<div class="warning-block">
			<p class="warning-text">
				Factory reset deletes all Tuitbot-managed data and returns the app to the
				onboarding screen. This action cannot be undone.
			</p>

			<div class="lists-row">
				<div class="list-block">
					<h3 class="list-title deleted">What gets deleted</h3>
					<ul class="list">
						<li>All table data (targets, replies, tweets, analytics)</li>
						<li>Configuration file and passphrase</li>
						<li>All sessions and media files</li>
						<li>Running automations</li>
					</ul>
				</div>
				<div class="list-block">
					<h3 class="list-title preserved">What is preserved</h3>
					<ul class="list">
						<li>Database schema and migrations</li>
						<li>API token (Tauri desktop mode)</li>
						<li>Backup files</li>
					</ul>
				</div>
			</div>
		</div>

		<div class="confirm-group">
			<label class="confirm-label" for="factory-reset-confirmation">
				Type <code>{CONFIRMATION_PHRASE}</code> to confirm
			</label>
			<input
				id="factory-reset-confirmation"
				type="text"
				class="confirm-input"
				placeholder={CONFIRMATION_PHRASE}
				bind:value={confirmationText}
				disabled={resetting}
				autocomplete="off"
				spellcheck="false"
			/>
			<button
				class="reset-button"
				onclick={handleReset}
				disabled={!canReset}
			>
				{#if resetting}
					<Loader2 size={14} class="spinning" />
					Resetting...
				{:else}
					<AlertTriangle size={14} />
					Factory Reset
				{/if}
			</button>
			{#if errorMsg}
				<p class="error-text">{errorMsg}</p>
			{/if}
		</div>
	</SettingsSection>
</div>

<style>
	/* Override SettingsSection icon and border for danger theme */
	.danger-zone :global(.settings-section) {
		border-color: color-mix(in srgb, var(--color-danger) 40%, var(--color-border-subtle));
	}

	.danger-zone :global(.section-icon) {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	/* Warning block */
	.warning-block {
		margin-bottom: 20px;
	}

	.warning-text {
		font-size: 13px;
		line-height: 1.5;
		color: var(--color-text-muted);
		margin: 0 0 16px;
	}

	.lists-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}

	.list-block {
		padding: 12px 14px;
		background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
	}

	.list-title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin: 0 0 8px;
	}

	.list-title.deleted {
		color: var(--color-danger);
	}

	.list-title.preserved {
		color: var(--color-success, #22c55e);
	}

	.list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.list li {
		font-size: 12px;
		color: var(--color-text-subtle);
		padding: 3px 0;
		line-height: 1.4;
	}

	.list li::before {
		content: '-';
		margin-right: 6px;
		color: var(--color-text-subtle);
	}

	/* Confirmation input and button */
	.confirm-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.confirm-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.confirm-label code {
		padding: 2px 6px;
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		border-radius: 4px;
		font-size: 12px;
		font-family: var(--font-mono, ui-monospace, monospace);
		color: var(--color-danger);
	}

	.confirm-input {
		padding: 8px 12px;
		font-size: 13px;
		font-family: var(--font-mono, ui-monospace, monospace);
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		max-width: 300px;
		transition: border-color 0.15s;
	}

	.confirm-input:focus {
		outline: none;
		border-color: var(--color-danger);
	}

	.confirm-input:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.reset-button {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		align-self: flex-start;
		padding: 8px 16px;
		font-size: 13px;
		font-weight: 500;
		color: white;
		background: var(--color-danger);
		border: none;
		border-radius: 6px;
		cursor: pointer;
		transition: opacity 0.15s;
	}

	.reset-button:hover:not(:disabled) {
		opacity: 0.9;
	}

	.reset-button:focus-visible {
		outline: 2px solid var(--color-danger);
		outline-offset: 2px;
	}

	.reset-button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.error-text {
		font-size: 12px;
		color: var(--color-danger);
		margin: 0;
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
