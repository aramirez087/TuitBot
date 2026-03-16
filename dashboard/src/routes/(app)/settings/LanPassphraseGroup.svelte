<script lang="ts">
	import { Copy, Check, RefreshCw } from 'lucide-svelte';
	import { api } from '$lib/api';

	interface Props {
		passphraseConfigured: boolean;
	}

	let { passphraseConfigured = $bindable() }: Props = $props();

	let revealedPassphrase = $state<string | null>(null);
	let copied = $state(false);
	let confirmingReset = $state(false);
	let resetting = $state(false);
	let resetError = $state('');

	let hideTimeout: ReturnType<typeof setTimeout> | null = null;
	let copyTimeout: ReturnType<typeof setTimeout> | null = null;
	let confirmTimeout: ReturnType<typeof setTimeout> | null = null;

	function initiateReset() {
		if (confirmingReset) {
			handleResetPassphrase();
			confirmingReset = false;
			if (confirmTimeout) clearTimeout(confirmTimeout);
		} else {
			confirmingReset = true;
			confirmTimeout = setTimeout(() => { confirmingReset = false; }, 3000);
		}
	}

	async function handleResetPassphrase() {
		resetting = true;
		resetError = '';
		try {
			const result = await api.lan.resetPassphrase();
			revealedPassphrase = result.passphrase;
			passphraseConfigured = true;
			copied = false;
			if (hideTimeout) clearTimeout(hideTimeout);
			hideTimeout = setTimeout(() => { revealedPassphrase = null; }, 30000);
		} catch (e) {
			console.error('Failed to reset passphrase', e);
			resetError = 'Failed to reset passphrase. Please try again.';
		}
		resetting = false;
	}

	async function copyPassphrase() {
		if (!revealedPassphrase) return;
		try {
			await navigator.clipboard.writeText(revealedPassphrase);
			copied = true;
			if (copyTimeout) clearTimeout(copyTimeout);
			copyTimeout = setTimeout(() => { copied = false; }, 2000);
		} catch {
			// Clipboard API not available.
		}
	}
</script>

<div class="passphrase-group">
	{#if revealedPassphrase}
		<div class="passphrase-reveal">
			<code class="passphrase-text">{revealedPassphrase}</code>
			<button
				class="copy-btn"
				onclick={copyPassphrase}
				title="Copy passphrase"
				aria-label="Copy passphrase to clipboard"
			>
				{#if copied}<Check size={14} />{:else}<Copy size={14} />{/if}
			</button>
		</div>
		<span class="field-hint">Save this passphrase — it will be hidden in 30 seconds</span>
	{/if}

	<button
		class="reset-btn"
		class:confirming={confirmingReset}
		onclick={initiateReset}
		disabled={resetting}
		aria-label="Reset passphrase"
	>
		<RefreshCw size={14} class={resetting ? 'spinning' : ''} />
		{#if resetting}
			Resetting...
		{:else if confirmingReset}
			Confirm Reset
		{:else}
			Reset Passphrase
		{/if}
	</button>
	{#if resetError}
		<span class="field-error">{resetError}</span>
	{/if}
	<span class="field-hint">
		Generate a new passphrase for browser login. All existing sessions will continue working.
	</span>
</div>

<style>
	.passphrase-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.passphrase-reveal {
		display: flex;
		align-items: center;
		gap: 8px;
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

	.copy-btn:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.reset-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
		align-self: flex-start;
	}

	.reset-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
	}

	.reset-btn:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.reset-btn.confirming {
		border-color: var(--color-danger);
		color: var(--color-danger);
	}

	.reset-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	:global(.spinning) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
