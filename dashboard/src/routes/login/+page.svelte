<script lang="ts">
	import { goto } from '$app/navigation';
	import { login } from '$lib/stores/auth';
	import { connectWs } from '$lib/stores/websocket';
	import { Zap, Loader2, LogIn } from 'lucide-svelte';

	let passphrase = $state('');
	let error = $state('');
	let submitting = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!passphrase.trim() || submitting) return;

		error = '';
		submitting = true;

		try {
			await login(passphrase.trim());
			connectWs(); // Cookie-based WS auth
			goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
		} finally {
			submitting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			handleSubmit(e);
		}
	}
</script>

<div class="login-page">
	<div class="login-card">
		<div class="logo">
			<Zap size={32} />
			<h1>Tuitbot</h1>
		</div>

		<p class="subtitle">Enter the passphrase shown in your server terminal</p>

		<form onsubmit={handleSubmit}>
			<div class="input-group">
				<label for="passphrase">Passphrase</label>
				<input
					id="passphrase"
					type="text"
					bind:value={passphrase}
					onkeydown={handleKeydown}
					placeholder="word1 word2 word3 word4"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					disabled={submitting}
				/>
			</div>

			{#if error}
				<div class="error-msg">{error}</div>
			{/if}

			<button type="submit" class="login-btn" disabled={!passphrase.trim() || submitting}>
				{#if submitting}
					<Loader2 size={16} class="spin" />
					Verifying...
				{:else}
					<LogIn size={16} />
					Sign in
				{/if}
			</button>
		</form>

		<p class="hint">
			Start the server with <code>--host 0.0.0.0</code> to access from other devices on your network.
		</p>
	</div>
</div>

<style>
	.login-page {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		background-color: var(--color-base);
		padding: 24px;
	}

	.login-card {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 40px;
		width: 100%;
		max-width: 400px;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 8px;
		color: var(--color-accent);
	}

	.logo h1 {
		font-size: 24px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text);
	}

	.subtitle {
		color: var(--color-text-muted);
		margin: 0 0 24px;
		font-size: 14px;
		line-height: 1.5;
	}

	form {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.input-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.input-group label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.input-group input {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 10px 14px;
		font-size: 15px;
		font-family: var(--font-mono);
		color: var(--color-text);
		outline: none;
		transition: border-color 0.15s;
		letter-spacing: 0.5px;
	}

	.input-group input:focus {
		border-color: var(--color-accent);
	}

	.input-group input:disabled {
		opacity: 0.6;
	}

	.error-msg {
		background: rgba(248, 81, 73, 0.1);
		border: 1px solid var(--color-danger);
		border-radius: 8px;
		padding: 10px 14px;
		font-size: 13px;
		color: var(--color-danger);
	}

	.login-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		background: var(--color-accent);
		color: #fff;
		border: none;
		border-radius: 8px;
		padding: 10px 20px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
	}

	.login-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.login-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.hint {
		margin: 20px 0 0;
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.5;
	}

	.hint code {
		background: var(--color-base);
		padding: 2px 6px;
		border-radius: 4px;
		font-family: var(--font-mono);
		font-size: 11px;
	}

	:global(.spin) {
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
