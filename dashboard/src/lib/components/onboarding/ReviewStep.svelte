<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { activeGoogleDrive } from '$lib/stores/connectors';

	let approvalMode = $state($onboardingData.approval_mode);

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
</script>

<div class="step">
	<h2 class="step-title">Review & Launch</h2>
	<p class="step-description">
		Review your configuration before starting Tuitbot.
	</p>

	<div class="summary">
		<div class="summary-section">
			<h3 class="summary-heading">X API</h3>
			<div class="summary-row">
				<span class="summary-label">Client ID</span>
				<span class="summary-value">{$onboardingData.client_id || '(not set)'}</span>
			</div>
			<div class="summary-row">
				<span class="summary-label">Client Secret</span>
				<span class="summary-value">{$onboardingData.client_secret ? '(set)' : '(none)'}</span>
			</div>
		</div>

		<div class="summary-section">
			<h3 class="summary-heading">Business</h3>
			<div class="summary-row">
				<span class="summary-label">Product</span>
				<span class="summary-value">{$onboardingData.product_name || '(not set)'}</span>
			</div>
			<div class="summary-row">
				<span class="summary-label">Description</span>
				<span class="summary-value">{$onboardingData.product_description || '(not set)'}</span>
			</div>
			{#if $onboardingData.product_url}
				<div class="summary-row">
					<span class="summary-label">URL</span>
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
			<h3 class="summary-heading">LLM Provider</h3>
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
		</div>

		<div class="summary-section">
			<h3 class="summary-heading">Content Source</h3>
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
			{#if !$onboardingData.vault_path && !$onboardingData.connection_id && !$onboardingData.folder_id}
				<div class="summary-row">
					<span class="summary-label">Status</span>
					<span class="summary-value">(skipped -- configure later in Settings)</span>
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
