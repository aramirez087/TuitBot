<script lang="ts">
	import { Eye, EyeOff } from 'lucide-svelte';
	import { draft, updateDraft } from '$lib/stores/settings';

	let showClientSecret = $state(false);
</script>

<div class="field-grid">
	<div class="field full-width info-banner">
		<p>
			OAuth authentication is managed via <code>tuitbot auth</code> in the CLI. Configure your
			Client ID and Secret below, then run the auth command to complete the OAuth flow.
		</p>
	</div>

	<div class="field">
		<label class="field-label" for="client_id">Client ID</label>
		<input
			id="client_id"
			type="text"
			class="text-input"
			value={$draft?.x_api.client_id ?? ''}
			oninput={(e) => updateDraft('x_api.client_id', e.currentTarget.value)}
			placeholder="Your X API Client ID"
		/>
	</div>

	<div class="field">
		<label class="field-label" for="client_secret">Client Secret</label>
		<div class="password-wrapper">
			<input
				id="client_secret"
				type={showClientSecret ? 'text' : 'password'}
				class="text-input password-input"
				value={$draft?.x_api.client_secret ?? ''}
				oninput={(e) =>
					updateDraft('x_api.client_secret', e.currentTarget.value || null)}
				placeholder="Optional for public clients"
			/>
			<button
				type="button"
				class="password-toggle"
				onclick={() => (showClientSecret = !showClientSecret)}
				aria-label={showClientSecret ? 'Hide' : 'Show'}
			>
				{#if showClientSecret}<EyeOff size={16} />{:else}<Eye size={16} />{/if}
			</button>
		</div>
	</div>

	<div class="field">
		<label class="field-label" for="auth_mode">Auth Mode</label>
		<select
			id="auth_mode"
			class="select-input"
			value={$draft?.auth.mode ?? 'manual'}
			onchange={(e) => updateDraft('auth.mode', e.currentTarget.value)}
		>
			<option value="manual">Manual</option>
			<option value="local_callback">Local Callback</option>
		</select>
	</div>
</div>

<style>
	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.text-input,
	.select-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus,
	.select-input:focus {
		border-color: var(--color-accent);
	}

	.select-input {
		cursor: pointer;
		appearance: auto;
	}

	.password-wrapper {
		position: relative;
		display: flex;
	}

	.password-input {
		flex: 1;
		padding-right: 40px;
	}

	.password-toggle {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: none;
		color: var(--color-text-muted);
		cursor: pointer;
		border-radius: 4px;
		transition: color 0.15s;
	}

	.password-toggle:hover {
		color: var(--color-text);
	}

	.info-banner {
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 6px;
	}

	.info-banner p {
		margin: 0;
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.info-banner code {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
		padding: 1px 6px;
		border-radius: 3px;
		font-size: 12px;
		font-family: var(--font-mono);
	}
</style>
