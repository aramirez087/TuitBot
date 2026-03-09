<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { activeGoogleDrive } from '$lib/stores/connectors';
	import { CheckCircle, Clock, AlertCircle } from 'lucide-svelte';

	interface Props {
		skippedSteps?: Set<string>;
	}

	let { skippedSteps = new Set<string>() }: Props = $props();

	let approvalMode = $state($onboardingData.approval_mode);
	let isBusiness = $derived($onboardingData.account_type === 'business');

	$effect(() => {
		onboardingData.updateField('approval_mode', approvalMode);
	});

	const sourceLabel = $derived(
		$onboardingData.source_type === 'google_drive'
			? 'Google Drive'
			: $onboardingData.source_type === 'local_fs'
				? 'Local Folder'
				: 'Not configured'
	);

	// Determine capability tier based on what's configured
	let hasXCredentials = $derived(
		$onboardingData.provider_backend === 'scraper' ||
		$onboardingData.client_id.trim().length > 0
	);
	let hasLlmConfig = $derived(
		$onboardingData.llm_provider === 'ollama' ||
		($onboardingData.llm_api_key.trim().length > 0 && $onboardingData.llm_model.trim().length > 0)
	);
	let hasContentSource = $derived(
		$onboardingData.vault_path.length > 0 ||
		$onboardingData.connection_id !== null ||
		$onboardingData.folder_id.length > 0
	);

	let tierLabel = $derived(
		hasLlmConfig && hasXCredentials
			? 'Generation Ready'
			: hasXCredentials
				? 'Exploration Ready'
				: 'Profile Ready'
	);

	let tierDescription = $derived(
		hasLlmConfig && hasXCredentials
			? 'You can discover conversations, generate AI drafts, and compose replies.'
			: hasXCredentials
				? 'You can discover and explore conversations. Add an LLM provider in Settings to enable AI drafts.'
				: 'Dashboard access is available. Configure X credentials and an LLM in Settings to unlock more features.'
	);

	// Deferred items
	let deferredItems = $derived.by(() => {
		const items: Array<{ label: string; detail: string }> = [];
		if (!hasLlmConfig) {
			items.push({ label: 'LLM Provider', detail: 'Required for AI draft generation' });
		}
		if (!hasContentSource) {
			items.push({ label: 'Content Source', detail: 'Vault or Drive for knowledge-grounded posts' });
		}
		if (skippedSteps.has('Language')) {
			items.push({ label: 'Language & Brand', detail: 'Using defaults (English, balanced voice)' });
		}
		return items;
	});
</script>

<div class="step">
	<h2 class="step-title">Review & Launch</h2>
	<p class="step-description">
		Review your setup. You can change any of these in Settings later.
	</p>

	{#if deferredItems.length > 0}
		<div class="tier-card">
			<div class="tier-header">
				<AlertCircle size={18} />
				<span class="tier-label">{tierLabel}</span>
			</div>
			<p class="tier-description">{tierDescription}</p>
			<div class="deferred-list">
				{#each deferredItems as item}
					<div class="deferred-item">
						<Clock size={14} />
						<span class="deferred-label">{item.label}</span>
						<span class="deferred-detail">{item.detail}</span>
					</div>
				{/each}
			</div>
		</div>
	{:else}
		<div class="tier-card tier-full">
			<div class="tier-header">
				<CheckCircle size={18} />
				<span class="tier-label">{tierLabel}</span>
			</div>
			<p class="tier-description">{tierDescription}</p>
		</div>
	{/if}

	<div class="summary">
		{#if hasXCredentials}
			<div class="summary-section">
				<h3 class="summary-heading">
					X API
					<span class="status-badge configured"><CheckCircle size={12} /> Configured</span>
				</h3>
				{#if $onboardingData.provider_backend === 'scraper'}
					<div class="summary-row">
						<span class="summary-label">Mode</span>
						<span class="summary-value">No-Key (Scraper)</span>
					</div>
				{:else}
					<div class="summary-row">
						<span class="summary-label">Client ID</span>
						<span class="summary-value">{$onboardingData.client_id || '(not set)'}</span>
					</div>
					<div class="summary-row">
						<span class="summary-label">Client Secret</span>
						<span class="summary-value">{$onboardingData.client_secret ? '(set)' : '(none)'}</span>
					</div>
				{/if}
			</div>
		{/if}

		<div class="summary-section">
			<h3 class="summary-heading">
				{isBusiness ? 'Business' : 'Profile'}
				<span class="status-badge configured"><CheckCircle size={12} /> Configured</span>
			</h3>
			<div class="summary-row">
				<span class="summary-label">{isBusiness ? 'Product' : 'Name'}</span>
				<span class="summary-value">{$onboardingData.product_name || '(not set)'}</span>
			</div>
			<div class="summary-row">
				<span class="summary-label">{isBusiness ? 'Description' : 'Bio'}</span>
				<span class="summary-value">{$onboardingData.product_description || '(not set)'}</span>
			</div>
			{#if $onboardingData.product_url}
				<div class="summary-row">
					<span class="summary-label">{isBusiness ? 'Product URL' : 'Website'}</span>
					<span class="summary-value">{$onboardingData.product_url}</span>
				</div>
			{/if}
			<div class="summary-row">
				<span class="summary-label">Audience</span>
				<span class="summary-value">{$onboardingData.target_audience || '(not set)'}</span>
			</div>
			<div class="summary-row">
				<span class="summary-label">Keywords</span>
				<span class="summary-value">{$onboardingData.product_keywords.join(', ') || '(none)'}</span>
			</div>
			<div class="summary-row">
				<span class="summary-label">Topics</span>
				<span class="summary-value">{$onboardingData.industry_topics.join(', ') || '(none)'}</span>
			</div>
		</div>

		<div class="summary-section">
			<h3 class="summary-heading">
				LLM Provider
				{#if hasLlmConfig}
					<span class="status-badge configured"><CheckCircle size={12} /> Configured</span>
				{:else}
					<span class="status-badge deferred"><Clock size={12} /> Set up later</span>
				{/if}
			</h3>
			{#if hasLlmConfig}
				<div class="summary-row">
					<span class="summary-label">Provider</span>
					<span class="summary-value">{$onboardingData.llm_provider}</span>
				</div>
				<div class="summary-row">
					<span class="summary-label">Model</span>
					<span class="summary-value">{$onboardingData.llm_model}</span>
				</div>
				<div class="summary-row">
					<span class="summary-label">API Key</span>
					<span class="summary-value">{$onboardingData.llm_api_key ? '(set)' : '(none)'}</span>
				</div>
				{#if $onboardingData.llm_base_url}
					<div class="summary-row">
						<span class="summary-label">Base URL</span>
						<span class="summary-value">{$onboardingData.llm_base_url}</span>
					</div>
				{/if}
			{:else}
				<div class="summary-row">
					<span class="summary-label">Status</span>
					<span class="summary-value deferred-text">Configure in Settings to enable AI generation</span>
				</div>
			{/if}
		</div>

		<div class="summary-section">
			<h3 class="summary-heading">
				Content Source
				{#if hasContentSource}
					<span class="status-badge configured"><CheckCircle size={12} /> Configured</span>
				{:else}
					<span class="status-badge deferred"><Clock size={12} /> Set up later</span>
				{/if}
			</h3>
			{#if hasContentSource}
				<div class="summary-row">
					<span class="summary-label">Source Type</span>
					<span class="summary-value">{sourceLabel}</span>
				</div>
				{#if $onboardingData.source_type === 'local_fs' && $onboardingData.vault_path}
					<div class="summary-row">
						<span class="summary-label">Vault Path</span>
						<span class="summary-value">{$onboardingData.vault_path}</span>
					</div>
				{/if}
				{#if $onboardingData.source_type === 'google_drive'}
					<div class="summary-row">
						<span class="summary-label">Account</span>
						<span class="summary-value">
							{$activeGoogleDrive?.account_email ?? ($onboardingData.connection_id ? 'Connected' : 'Not connected')}
						</span>
					</div>
					{#if $onboardingData.folder_id}
						<div class="summary-row">
							<span class="summary-label">Folder ID</span>
							<span class="summary-value">{$onboardingData.folder_id}</span>
						</div>
					{/if}
				{/if}
			{:else}
				<div class="summary-row">
					<span class="summary-label">Status</span>
					<span class="summary-value deferred-text">Configure in Settings for knowledge-grounded posts</span>
				</div>
			{/if}
		</div>

		<div class="summary-section">
			<h3 class="summary-heading">Settings</h3>
			<div class="approval-toggle">
				<label class="toggle-label">
					<input
						type="checkbox"
						class="toggle-checkbox"
						bind:checked={approvalMode}
					/>
					<span class="toggle-switch"></span>
					<span class="toggle-text">Approval Mode</span>
				</label>
				<span class="toggle-hint">
					When enabled, posts are queued for your review before being published.
					Recommended for new users.
				</span>
			</div>
		</div>
	</div>
</div>

<style>
	.step {
		display: flex;
		flex-direction: column;
		gap: 20px;
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
		line-height: 1.5;
		margin: 0;
	}

	.tier-card {
		padding: 16px;
		background: color-mix(in srgb, var(--color-warning) 6%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-warning) 20%, var(--color-border-subtle));
		border-radius: 10px;
	}

	.tier-card.tier-full {
		background: color-mix(in srgb, var(--color-success) 6%, var(--color-surface));
		border-color: color-mix(in srgb, var(--color-success) 20%, var(--color-border-subtle));
	}

	.tier-header {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--color-warning);
	}

	.tier-full .tier-header {
		color: var(--color-success);
	}

	.tier-label {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.tier-description {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 6px 0 0;
		line-height: 1.5;
	}

	.deferred-list {
		margin-top: 12px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.deferred-item {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.deferred-label {
		font-weight: 600;
		color: var(--color-text);
	}

	.deferred-detail {
		color: var(--color-text-subtle);
	}

	.summary {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.summary-section {
		padding: 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	.summary-heading {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0 0 12px 0;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 500;
		text-transform: none;
		letter-spacing: 0;
		padding: 2px 8px;
		border-radius: 10px;
	}

	.status-badge.configured {
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
	}

	.status-badge.deferred {
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
	}

	.summary-row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		padding: 4px 0;
		font-size: 13px;
	}

	.summary-label {
		color: var(--color-text-muted);
	}

	.summary-value {
		color: var(--color-text);
		font-weight: 500;
		text-align: right;
		max-width: 60%;
		word-break: break-word;
	}

	.deferred-text {
		color: var(--color-text-subtle);
		font-style: italic;
		font-weight: 400;
	}

	.approval-toggle {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.toggle-label {
		display: flex;
		align-items: center;
		gap: 10px;
		cursor: pointer;
	}

	.toggle-checkbox {
		display: none;
	}

	.toggle-switch {
		width: 36px;
		height: 20px;
		background: var(--color-border);
		border-radius: 10px;
		position: relative;
		transition: background 0.2s;
		flex-shrink: 0;
	}

	.toggle-switch::after {
		content: '';
		width: 16px;
		height: 16px;
		background: white;
		border-radius: 50%;
		position: absolute;
		top: 2px;
		left: 2px;
		transition: transform 0.2s;
	}

	.toggle-checkbox:checked + .toggle-switch {
		background: var(--color-accent);
	}

	.toggle-checkbox:checked + .toggle-switch::after {
		transform: translateX(16px);
	}

	.toggle-text {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
	}

	.toggle-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.5;
		padding-left: 46px;
	}
</style>
