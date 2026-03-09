<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { api } from '$lib/api';
	import { CheckCircle, XCircle, Loader2, RotateCcw, Info } from 'lucide-svelte';

	interface Props {
		hasLlmConfig?: boolean;
	}

	let { hasLlmConfig = false }: Props = $props();

	let testing = $state(false);
	let result = $state<{ success: boolean; error?: string; latency_ms?: number } | null>(null);

	async function runTest() {
		testing = true;
		result = null;
		try {
			const data = $onboardingData;
			result = await api.settings.testLlm({
				provider: data.llm_provider,
				api_key: data.llm_api_key || null,
				model: data.llm_model,
				base_url: data.llm_base_url || null
			});
		} catch (e) {
			result = {
				success: false,
				error: e instanceof Error ? e.message : 'Connection test failed'
			};
		} finally {
			testing = false;
		}
	}

	// Auto-run test on mount only if LLM is configured.
	$effect(() => {
		if (hasLlmConfig && !result && !testing) {
			runTest();
		}
	});
</script>

<div class="step">
	<h2>Validation</h2>
	<p class="step-description">
		{#if hasLlmConfig}
			Testing your LLM connection to make sure everything is configured correctly.
		{:else}
			Review your current configuration before finishing.
		{/if}
	</p>

	{#if hasLlmConfig}
		<div class="test-card">
			{#if testing}
				<div class="test-status testing">
					<Loader2 size={24} class="spinner" />
					<span>Testing LLM connection...</span>
				</div>
			{:else if result?.success}
				<div class="test-status success">
					<CheckCircle size={24} />
					<div class="test-details">
						<span class="test-title">Connection successful</span>
						{#if result.latency_ms}
							<span class="test-meta">Response time: {result.latency_ms}ms</span>
						{/if}
					</div>
				</div>
			{:else if result}
				<div class="test-status failure">
					<XCircle size={24} />
					<div class="test-details">
						<span class="test-title">Connection failed</span>
						<span class="test-error">{result.error}</span>
					</div>
				</div>
				<div class="test-actions">
					<button class="retry-btn" onclick={runTest}>
						<RotateCcw size={14} />
						Retry
					</button>
				</div>
			{/if}
		</div>
	{:else}
		<div class="info-card">
			<div class="info-status">
				<Info size={22} />
				<div class="info-details">
					<span class="info-title">LLM not configured</span>
					<span class="info-text">
						Content generation will be available after you set up an LLM provider
						in Settings. You can still explore content and use the dashboard.
					</span>
				</div>
			</div>
		</div>
	{/if}

	<div class="summary">
		<h3>Configuration Summary</h3>
		{#if hasLlmConfig}
			<div class="summary-row">
				<span class="summary-label">Provider</span>
				<span class="summary-value">{$onboardingData.llm_provider}</span>
			</div>
			<div class="summary-row">
				<span class="summary-label">Model</span>
				<span class="summary-value">{$onboardingData.llm_model}</span>
			</div>
		{:else}
			<div class="summary-row">
				<span class="summary-label">LLM Provider</span>
				<span class="summary-value deferred">Set up later</span>
			</div>
		{/if}
		<div class="summary-row">
			<span class="summary-label">Language</span>
			<span class="summary-value">{$onboardingData.language === 'en' ? 'English' : $onboardingData.language === 'es' ? 'Spanish' : 'Bilingual'}</span>
		</div>
		<div class="summary-row">
			<span class="summary-label">Brand Voice</span>
			<span class="summary-value" style="text-transform: capitalize">{$onboardingData.brand_voice}</span>
		</div>
	</div>
</div>

<style>
	.step {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	h2 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	h3 {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 12px;
	}

	.step-description {
		font-size: 14px;
		color: var(--color-text-muted);
		margin: -16px 0 0;
		line-height: 1.5;
	}

	.test-card, .info-card {
		padding: 24px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 10px;
		background: var(--color-surface);
	}

	.test-status, .info-status {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.test-status.testing {
		color: var(--color-text-muted);
	}

	.test-status.success {
		color: var(--color-success);
	}

	.test-status.failure {
		color: var(--color-danger);
	}

	.info-status {
		color: var(--color-accent);
		align-items: flex-start;
	}

	.test-details, .info-details {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.test-title, .info-title {
		font-size: 15px;
		font-weight: 600;
	}

	.info-title {
		color: var(--color-text);
	}

	.info-text {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.test-meta {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.test-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.test-actions {
		margin-top: 14px;
	}

	.retry-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.retry-btn:hover {
		background: var(--color-surface-hover);
	}

	.summary {
		padding: 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: var(--color-surface);
	}

	.summary-row {
		display: flex;
		justify-content: space-between;
		padding: 6px 0;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.summary-row:last-child {
		border-bottom: none;
	}

	.summary-label {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.summary-value {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.summary-value.deferred {
		color: var(--color-text-subtle);
		font-style: italic;
	}

	:global(.spinner) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
