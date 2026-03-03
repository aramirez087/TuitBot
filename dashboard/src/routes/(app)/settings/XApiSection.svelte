<script lang="ts">
	import { Key, Eye, EyeOff, AlertTriangle } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import { draft, updateDraft } from '$lib/stores/settings';
	import { deploymentMode } from '$lib/stores/runtime';

	let showClientSecret = $state(false);
	let showAdvanced = $state(false);

	let selectedMode = $derived(
		$draft?.x_api.provider_backend === 'scraper' ? 'scraper' : 'x_api'
	);

	let isCloud = $derived($deploymentMode === 'cloud');

	function setMode(mode: string) {
		updateDraft('x_api.provider_backend', mode === 'x_api' ? '' : mode);
		if (mode === 'scraper') {
			showAdvanced = false;
		}
	}
</script>

{#if $draft}
<SettingsSection
	id="xapi"
	title="X Access"
	description="How Tuitbot connects to X (Twitter)"
	icon={Key}
>
	<div class="section-body">
		{#if !isCloud}
			<div class="mode-selector">
				<button
					type="button"
					class="mode-card"
					class:selected={selectedMode === 'x_api'}
					onclick={() => setMode('x_api')}
				>
					<div class="mode-radio" class:checked={selectedMode === 'x_api'}></div>
					<div class="mode-info">
						<span class="mode-label">Official X API</span>
						<span class="mode-desc">Full features with API credentials</span>
					</div>
				</button>
				<button
					type="button"
					class="mode-card"
					class:selected={selectedMode === 'scraper'}
					onclick={() => setMode('scraper')}
				>
					<div class="mode-radio" class:checked={selectedMode === 'scraper'}></div>
					<div class="mode-info">
						<span class="mode-label">Local No-Key Mode</span>
						<span class="mode-desc">No credentials needed</span>
					</div>
				</button>
			</div>
		{/if}

		{#if selectedMode === 'x_api' || isCloud}
			<div class="field-grid">
				<div class="field full-width info-banner">
					<p>
						OAuth authentication is managed via <code>tuitbot auth</code> in the CLI.
						Configure your Client ID and Secret below, then run the auth command to complete the OAuth flow.
					</p>
				</div>

				<div class="field">
					<label class="field-label" for="client_id">Client ID</label>
					<input
						id="client_id"
						type="text"
						class="text-input"
						value={$draft.x_api.client_id}
						oninput={(e) =>
							updateDraft('x_api.client_id', e.currentTarget.value)}
						placeholder="Your X API Client ID"
					/>
				</div>

				<div class="field">
					<label class="field-label" for="client_secret">
						Client Secret
					</label>
					<div class="password-wrapper">
						<input
							id="client_secret"
							type={showClientSecret ? 'text' : 'password'}
							class="text-input password-input"
							value={$draft.x_api.client_secret ?? ''}
							oninput={(e) =>
								updateDraft(
									'x_api.client_secret',
									e.currentTarget.value || null
								)}
							placeholder="Optional for public clients"
						/>
						<button
							type="button"
							class="password-toggle"
							onclick={() => (showClientSecret = !showClientSecret)}
							aria-label={showClientSecret ? 'Hide' : 'Show'}
						>
							{#if showClientSecret}
								<EyeOff size={16} />
							{:else}
								<Eye size={16} />
							{/if}
						</button>
					</div>
				</div>

				<div class="field">
					<label class="field-label" for="auth_mode">Auth Mode</label>
					<select
						id="auth_mode"
						class="select-input"
						value={$draft.auth.mode}
						onchange={(e) =>
							updateDraft('auth.mode', e.currentTarget.value)}
					>
						<option value="manual">Manual</option>
						<option value="local_callback">Local Callback</option>
					</select>
				</div>
			</div>
		{:else}
			<div class="info-banner scraper-banner">
				<p>
					Run discovery and drafting without API credentials.
					Read-only by default. Some features like posting, mentions, and analytics are unavailable.
					Switch to Official X API anytime for full capabilities.
				</p>
			</div>

			<div class="advanced-section">
				<button
					type="button"
					class="advanced-toggle"
					onclick={() => (showAdvanced = !showAdvanced)}
				>
					<span class="toggle-arrow" class:open={showAdvanced}>&#9656;</span>
					Advanced
				</button>

				{#if showAdvanced}
					<div class="advanced-body">
						<label class="toggle-row">
							<input
								type="checkbox"
								checked={$draft.x_api.scraper_allow_mutations}
								onchange={(e) =>
									updateDraft('x_api.scraper_allow_mutations', e.currentTarget.checked)}
							/>
							<span class="toggle-label">Allow write operations</span>
						</label>
						<div class="warning-banner">
							<AlertTriangle size={14} />
							<p>
								Posting via Local No-Key Mode carries elevated risk of account restrictions.
								The Official X API is recommended for posting.
							</p>
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</SettingsSection>
{/if}

<style>
	.section-body {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.mode-selector {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}

	.mode-card {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 12px 14px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		cursor: pointer;
		text-align: left;
		transition: border-color 0.15s, background 0.15s;
	}

	.mode-card:hover {
		border-color: var(--color-border-hover, var(--color-text-muted));
	}

	.mode-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, var(--color-base));
	}

	.mode-radio {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 2px solid var(--color-border);
		flex-shrink: 0;
		margin-top: 2px;
		transition: border-color 0.15s;
	}

	.mode-radio.checked {
		border-color: var(--color-accent);
		background: var(--color-accent);
		box-shadow: inset 0 0 0 3px var(--color-base);
	}

	.mode-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.mode-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.mode-desc {
		font-size: 12px;
		color: var(--color-text-muted);
	}

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

	.scraper-banner {
		background: color-mix(in srgb, var(--color-text-muted) 6%, transparent);
		border-color: color-mix(in srgb, var(--color-text-muted) 15%, transparent);
	}

	.advanced-section {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.advanced-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		padding: 0;
		transition: color 0.15s;
	}

	.advanced-toggle:hover {
		color: var(--color-text);
	}

	.toggle-arrow {
		display: inline-block;
		transition: transform 0.15s;
		font-size: 11px;
	}

	.toggle-arrow.open {
		transform: rotate(90deg);
	}

	.advanced-body {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding-left: 4px;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}

	.toggle-row input[type='checkbox'] {
		accent-color: var(--color-accent);
	}

	.toggle-label {
		font-size: 13px;
		color: var(--color-text);
	}

	.warning-banner {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 10px 12px;
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 20%, transparent);
		border-radius: 6px;
		color: var(--color-warning, #f59e0b);
	}

	.warning-banner p {
		margin: 0;
		font-size: 12px;
		line-height: 1.5;
		color: var(--color-text-muted);
	}
</style>
