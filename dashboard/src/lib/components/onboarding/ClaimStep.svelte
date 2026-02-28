<script lang="ts">
	import { generatePassphrase } from '$lib/wordlist';
	import { Copy, Check, RefreshCw } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let {
		passphrase = $bindable(''),
		saved = $bindable(false),
	}: {
		passphrase: string;
		saved: boolean;
	} = $props();

	let useCustom = $state(false);
	let customValue = $state('');
	let copied = $state(false);
	let copyTimeout: ReturnType<typeof setTimeout> | null = null;

	let generatedPhrase = $state('');

	onMount(() => {
		generatedPhrase = generatePassphrase();
		passphrase = generatedPhrase;
		return () => {
			if (copyTimeout) clearTimeout(copyTimeout);
		};
	});

	function regenerate() {
		generatedPhrase = generatePassphrase();
		if (!useCustom) {
			passphrase = generatedPhrase;
		}
		copied = false;
	}

	function toggleCustom() {
		useCustom = !useCustom;
		if (useCustom) {
			passphrase = customValue;
		} else {
			passphrase = generatedPhrase;
		}
		saved = false;
	}

	function handleCustomInput(e: Event) {
		const target = e.target as HTMLInputElement;
		customValue = target.value;
		if (useCustom) {
			passphrase = customValue;
		}
		saved = false;
	}

	async function copyToClipboard() {
		const text = useCustom ? customValue : generatedPhrase;
		if (!text) return;
		try {
			await navigator.clipboard.writeText(text);
			copied = true;
			if (copyTimeout) clearTimeout(copyTimeout);
			copyTimeout = setTimeout(() => {
				copied = false;
			}, 2000);
		} catch {
			// Clipboard API not available.
		}
	}

	let isValid = $derived(passphrase.trim().length >= 8);
</script>

<div class="step">
	<h2 class="step-title">Secure Your Instance</h2>
	<p class="step-description">
		This passphrase protects your dashboard when accessing it from a browser.
		You'll need it to log in again if your session expires.
	</p>

	{#if !useCustom}
		<div class="passphrase-display">
			<code class="passphrase-text">{generatedPhrase}</code>
			<div class="passphrase-actions">
				<button
					type="button"
					class="icon-btn"
					onclick={regenerate}
					aria-label="Generate new passphrase"
					title="Generate new passphrase"
				>
					<RefreshCw size={16} />
				</button>
				<button
					type="button"
					class="icon-btn"
					onclick={copyToClipboard}
					aria-label="Copy passphrase to clipboard"
					title="Copy to clipboard"
				>
					{#if copied}
						<Check size={16} />
					{:else}
						<Copy size={16} />
					{/if}
				</button>
			</div>
		</div>
	{/if}

	<button type="button" class="toggle-custom" onclick={toggleCustom}>
		{useCustom ? 'Use generated passphrase instead' : 'Or type your own'}
	</button>

	{#if useCustom}
		<div class="custom-input-group">
			<label for="custom-passphrase">Custom passphrase</label>
			<input
				id="custom-passphrase"
				type="text"
				value={customValue}
				oninput={handleCustomInput}
				placeholder="Minimum 8 characters"
				autocomplete="off"
				autocapitalize="off"
				spellcheck="false"
			/>
			{#if customValue && customValue.trim().length < 8}
				<span class="validation-hint">Minimum 8 characters ({customValue.trim().length}/8)</span>
			{/if}
		</div>
	{/if}

	<div class="warning-box">
		<span class="warning-icon">&#9888;</span>
		<div class="warning-content">
			<strong>Save this passphrase</strong> — it cannot be recovered later.
			You can reset it from Settings &rarr; LAN Access or via the CLI.
		</div>
	</div>

	<label class="save-checkbox">
		<input type="checkbox" bind:checked={saved} disabled={!isValid} />
		<span>I've saved my passphrase</span>
	</label>
</div>

<style>
	.step {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.step-title {
		font-size: 20px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.step-description {
		font-size: 14px;
		color: var(--color-text-muted);
		line-height: 1.6;
		margin: 0;
	}

	.passphrase-display {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.passphrase-text {
		flex: 1;
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-accent) 8%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
		border-radius: 8px;
		font-size: 16px;
		font-family: var(--font-mono, ui-monospace, monospace);
		color: var(--color-text);
		letter-spacing: 0.03em;
	}

	.passphrase-actions {
		display: flex;
		gap: 4px;
	}

	.icon-btn {
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

	.icon-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.icon-btn:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.toggle-custom {
		background: none;
		border: none;
		padding: 0;
		font-size: 13px;
		color: var(--color-accent);
		cursor: pointer;
		text-align: left;
	}

	.toggle-custom:hover {
		text-decoration: underline;
	}

	.toggle-custom:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.custom-input-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.custom-input-group label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.custom-input-group input {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 10px 14px;
		font-size: 15px;
		font-family: var(--font-mono, ui-monospace, monospace);
		color: var(--color-text);
		outline: none;
		transition: border-color 0.15s, box-shadow 0.15s;
	}

	.custom-input-group input:focus-visible {
		border-color: var(--color-accent);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 20%, transparent);
	}

	.validation-hint {
		font-size: 12px;
		color: var(--color-danger);
	}

	.warning-box {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 12px 14px;
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 25%, transparent);
		border-radius: 8px;
	}

	.warning-icon {
		font-size: 16px;
		color: var(--color-warning, #f59e0b);
		line-height: 1.4;
	}

	.warning-content {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.warning-content strong {
		color: var(--color-text);
	}

	.save-checkbox {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: var(--color-text);
		cursor: pointer;
		padding: 8px 0;
	}

	.save-checkbox input[type="checkbox"] {
		width: 18px;
		height: 18px;
		accent-color: var(--color-accent);
		cursor: pointer;
	}

	.save-checkbox input[type="checkbox"]:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.save-checkbox input[type="checkbox"]:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}
</style>
