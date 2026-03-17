<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { CheckCircle, Clock, AlertCircle } from 'lucide-svelte';
	import ReviewSummary from './ReviewSummary.svelte';

	interface Props {
		skippedSteps?: Set<string>;
	}

	let { skippedSteps = new Set<string>() }: Props = $props();

	let isBusiness = $derived($onboardingData.account_type === 'business');

	const sourceLabel = $derived(
		$onboardingData.source_type === 'google_drive'
			? 'Google Drive'
			: $onboardingData.source_type === 'local_fs'
				? 'Local Folder'
				: 'Not configured',
	);

	let hasXCredentials = $derived(
		$onboardingData.provider_backend === 'scraper' ||
			$onboardingData.client_id.trim().length > 0,
	);
	let hasLlmConfig = $derived(
		$onboardingData.llm_provider === 'ollama' ||
			($onboardingData.llm_api_key.trim().length > 0 &&
				$onboardingData.llm_model.trim().length > 0),
	);
	let hasContentSource = $derived(
		$onboardingData.vault_path.length > 0 ||
			$onboardingData.connection_id !== null ||
			$onboardingData.folder_id.length > 0,
	);

	let tierLabel = $derived(
		hasLlmConfig && hasXCredentials
			? 'Generation Ready'
			: hasXCredentials
				? 'Exploration Ready'
				: 'Profile Ready',
	);

	let tierDescription = $derived(
		hasLlmConfig && hasXCredentials
			? 'Full AI-powered content generation. Find conversations on X, draft replies with AI assistance, and schedule posts.'
			: hasXCredentials
				? 'Discovery mode enabled. Explore conversations on X and read analytics. Enable AI drafting by adding an LLM provider in Settings.'
				: 'Basic dashboard access only. Connect your X account and add an LLM provider in Settings to enable discovery and AI drafting.',
	);

	let deferredItems = $derived.by(() => {
		const items: Array<{ label: string; detail: string }> = [];
		if (!hasLlmConfig) {
			items.push({ label: 'LLM Provider', detail: 'Required for AI draft generation' });
		}
		if (!hasContentSource) {
			items.push({
				label: 'Content Source',
				detail: 'Vault or Drive for knowledge-grounded posts',
			});
		}
		if (skippedSteps.has('Language')) {
			items.push({ label: 'Language & Brand', detail: 'Using defaults (English, balanced voice)' });
		}
		return items;
	});
</script>

<div class="step">
	<h2 class="step-title">Review & Launch</h2>
	<p class="step-description">Review your setup. You can change any of these in Settings later.</p>

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

	<ReviewSummary
		{isBusiness}
		{hasXCredentials}
		{hasLlmConfig}
		{hasContentSource}
		{sourceLabel}
	/>
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
</style>
