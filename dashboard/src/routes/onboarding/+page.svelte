<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { api } from '$lib/api';
	import { onboardingData } from '$lib/stores/onboarding';
	import { authMode as authModeStore, claimSession } from '$lib/stores/auth';
	import { connectWs } from '$lib/stores/websocket';
	import WelcomeStep from '$lib/components/onboarding/WelcomeStep.svelte';
	import XApiStep from '$lib/components/onboarding/XApiStep.svelte';
	import LlmStep from '$lib/components/onboarding/LlmStep.svelte';
	import ProfileAnalysisState from '$lib/components/onboarding/ProfileAnalysisState.svelte';
	import PrefillProfileForm from '$lib/components/onboarding/PrefillProfileForm.svelte';
	import LanguageBrandStep from '$lib/components/onboarding/LanguageBrandStep.svelte';
	import SourcesStep from '$lib/components/onboarding/SourcesStep.svelte';
	import ValidationStep from '$lib/components/onboarding/ValidationStep.svelte';
	import ReviewStep from '$lib/components/onboarding/ReviewStep.svelte';
	import ClaimStep from '$lib/components/onboarding/ClaimStep.svelte';
	import { Zap, ArrowLeft, ArrowRight, Loader2 } from 'lucide-svelte';

	// Step flow varies by mode:
	// API mode: Welcome → X Access → LLM → Analyze → Profile → Language → Vault → Validate → Review [→ Secure]
	// Scraper mode: Welcome → X Access → Profile → LLM → Language → Vault → Validate → Review [→ Secure]
	let isScraperMode = $derived($onboardingData.provider_backend === 'scraper');
	let baseSteps = $derived(
		isScraperMode
			? ['Welcome', 'X Access', 'Profile', 'LLM', 'Language', 'Vault', 'Validate', 'Review']
			: ['Welcome', 'X Access', 'LLM', 'Analyze', 'Profile', 'Language', 'Vault', 'Validate', 'Review']
	);

	let currentStep = $state(0);
	let submitting = $state(false);
	let errorMsg = $state('');

	// Claim state — only used in web mode.
	let claimPassphrase = $state('');
	let passphraseSaved = $state(false);

	// Tauri/bearer mode skips the claim step. Web mode always shows it —
	// either as passphrase generation (fresh) or recovery guidance (already claimed).
	let isTauri = $derived($authModeStore === 'tauri');
	let alreadyClaimed = $derived($page.url.searchParams.get('claimed') === '1');
	let showClaimStep = $derived(!isTauri);
	let steps = $derived(showClaimStep ? [...baseSteps, 'Secure'] : baseSteps);
	let currentStepName = $derived(steps[currentStep] ?? '');
	let isLastStep = $derived(currentStep === steps.length - 1);
	let isClaimStep = $derived(showClaimStep && currentStep === steps.length - 1);
	let isAnalyzeStep = $derived(currentStepName === 'Analyze');

	// Prevent navigation away during claim step if passphrase is generated but not submitted.
	$effect(() => {
		if (isClaimStep && claimPassphrase && !submitting) {
			const handler = (e: BeforeUnloadEvent) => { e.preventDefault(); };
			window.addEventListener('beforeunload', handler);
			return () => window.removeEventListener('beforeunload', handler);
		}
	});

	function canAdvance(): boolean {
		const data = $onboardingData;
		switch (currentStepName) {
			case 'Welcome':
				return true;
			case 'X Access':
				if (data.provider_backend === 'scraper') return true;
				return data.client_id.trim().length > 0;
			case 'Profile':
				return (
					data.product_name.trim().length > 0 &&
					data.product_description.trim().length > 0 &&
					data.target_audience.trim().length > 0 &&
					data.product_keywords.length > 0 &&
					data.industry_topics.length > 0
				);
			case 'LLM':
				if (data.llm_provider === 'ollama') return data.llm_model.trim().length > 0;
				return data.llm_api_key.trim().length > 0 && data.llm_model.trim().length > 0;
			case 'Analyze':
				return false;
			case 'Language':
				return true;
			case 'Vault':
				return true;
			case 'Validate':
				return true;
			case 'Review':
				return true;
			case 'Secure':
				if (alreadyClaimed) return passphraseSaved;
				return claimPassphrase.trim().length >= 8 && passphraseSaved;
			default:
				return false;
		}
	}

	function next() {
		if (currentStep < steps.length - 1) {
			currentStep++;
			errorMsg = '';
		}
	}

	function back() {
		if (currentStep > 0) {
			// When going back from Profile in API mode, skip the Analyze step
			if (currentStepName === 'Profile' && !isScraperMode) {
				currentStep -= 2;
			} else {
				currentStep--;
			}
			errorMsg = '';
		}
	}

	async function submit() {
		submitting = true;
		errorMsg = '';
		let config: Record<string, unknown> = {};

		try {
			const data = $onboardingData;
			config = {
				x_api: data.provider_backend === 'scraper'
					? { provider_backend: 'scraper' }
					: {
						client_id: data.client_id,
						...(data.client_secret ? { client_secret: data.client_secret } : {}),
					},
				business: {
					product_name: data.product_name,
					product_description: data.product_description,
					...(data.product_url ? { product_url: data.product_url } : {}),
					target_audience: data.target_audience,
					product_keywords: data.product_keywords,
					industry_topics: data.industry_topics,
				},
				llm: {
					provider: data.llm_provider,
					...(data.llm_api_key ? { api_key: data.llm_api_key } : {}),
					model: data.llm_model,
					...(data.llm_base_url ? { base_url: data.llm_base_url } : {}),
				},
				approval_mode: data.approval_mode,
			};

			if (data.source_type === 'google_drive' && (data.connection_id || data.folder_id)) {
				config.content_sources = {
					sources: [{
						source_type: 'google_drive',
						path: null,
						folder_id: data.folder_id || null,
						service_account_key: null,
						connection_id: data.connection_id,
						watch: data.vault_watch,
						file_patterns: ['*.md', '*.txt'],
						loop_back_enabled: false,
						poll_interval_seconds: data.poll_interval_seconds || 300,
					}]
				};
			} else if (data.vault_path) {
				config.content_sources = {
					sources: [{
						source_type: 'local_fs',
						path: data.vault_path,
						folder_id: null,
						service_account_key: null,
						watch: data.vault_watch,
						file_patterns: ['*.md', '*.txt'],
						loop_back_enabled: data.vault_loop_back,
						poll_interval_seconds: null,
					}]
				};
			}

			// Include claim for web mode (skip if instance already claimed).
			if (showClaimStep && claimPassphrase.trim()) {
				config.claim = { passphrase: claimPassphrase.trim() };
			}

			const result = await api.settings.init(config);

			if (result.status === 'validation_failed' && result.errors) {
				errorMsg = result.errors.map((e) => `${e.field}: ${e.message}`).join('; ');
				return;
			}

			// If claim was included, establish session from response.
			if (result.csrf_token) {
				claimSession(result.csrf_token);
				connectWs();
			}

			onboardingData.reset();
			if (alreadyClaimed) {
				goto('/login');
			} else {
				goto('/content?compose=true');
			}
		} catch (e) {
			const msg = e instanceof Error ? e.message : '';
			if (msg.toLowerCase().includes('already claimed') && config.claim) {
				// Instance was claimed between page load and submit (race condition).
				// Retry without claim so the config is still created.
				try {
					delete config.claim;
					await api.settings.init(config);
					onboardingData.reset();
					goto('/login');
					return;
				} catch {
					// Retry also failed — show original error.
				}
			}
			errorMsg = msg || 'Failed to create configuration';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="onboarding">
	<div class="onboarding-header">
		<div class="logo">
			<Zap size={20} strokeWidth={2.5} />
			<span class="logo-text">Tuitbot</span>
		</div>
	</div>

	<div class="onboarding-content">
		<div class="progress">
			{#each steps as step, i}
				{#if step !== 'Analyze'}
					<div class="progress-step" class:active={i === currentStep || (step === 'Profile' && isAnalyzeStep)} class:completed={i < currentStep}>
						<div class="progress-dot">
							{#if i < currentStep}
								<span class="check-mark">&#10003;</span>
							{:else}
								{i + 1}
							{/if}
						</div>
						<span class="progress-label">{step}</span>
					</div>
					{#if i < steps.length - 1 && steps[i + 1] !== 'Analyze'}
						<div class="progress-line" class:filled={i < currentStep}></div>
					{/if}
				{/if}
			{/each}
		</div>

		<div class="step-content">
			{#if currentStepName === 'Welcome'}
				<WelcomeStep />
			{:else if currentStepName === 'X Access'}
				<XApiStep />
			{:else if currentStepName === 'LLM'}
				<LlmStep />
			{:else if currentStepName === 'Analyze'}
				<ProfileAnalysisState oncomplete={next} />
			{:else if currentStepName === 'Profile'}
				<PrefillProfileForm />
			{:else if currentStepName === 'Language'}
				<LanguageBrandStep />
			{:else if currentStepName === 'Vault'}
				<SourcesStep />
			{:else if currentStepName === 'Validate'}
				<ValidationStep />
			{:else if currentStepName === 'Review'}
				<ReviewStep />
			{:else if currentStepName === 'Secure'}
				<ClaimStep bind:passphrase={claimPassphrase} bind:saved={passphraseSaved} {alreadyClaimed} />
			{/if}
		</div>

		{#if errorMsg}
			<div class="error-banner" role="alert">{errorMsg}</div>
		{/if}

		<div class="actions">
			{#if currentStep > 0 && !isAnalyzeStep}
				<button class="btn btn-secondary" onclick={back} disabled={submitting}>
					<ArrowLeft size={16} />
					Back
				</button>
			{:else}
				<div></div>
			{/if}

			{#if isAnalyzeStep}
				<!-- Analyze step auto-advances; no button needed -->
				<div></div>
			{:else if !isLastStep}
				<button
					class="btn btn-primary"
					onclick={next}
					disabled={!canAdvance()}
				>
					{currentStep === 0 ? 'Get Started' : 'Next'}
					<ArrowRight size={16} />
				</button>
			{:else}
				<button
					class="btn btn-primary"
					onclick={submit}
					disabled={submitting || (isClaimStep && !canAdvance())}
				>
					{#if submitting}
						<span class="spinner"><Loader2 size={16} /></span>
						Creating...
					{:else}
						Start Tuitbot
						<Zap size={16} />
					{/if}
				</button>
			{/if}
		</div>
	</div>
</div>

<style>
	.onboarding {
		min-height: 100vh;
		background-color: var(--color-base);
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.onboarding-header {
		width: 100%;
		padding: 20px 32px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--color-accent);
	}

	.logo-text {
		font-size: 16px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--color-text);
	}

	.onboarding-content {
		width: 100%;
		max-width: 600px;
		padding: 40px 24px;
		display: flex;
		flex-direction: column;
		gap: 32px;
	}

	.progress {
		display: flex;
		align-items: center;
		gap: 0;
	}

	.progress-step {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.progress-dot {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 600;
		background: var(--color-surface);
		border: 2px solid var(--color-border);
		color: var(--color-text-muted);
		transition: all 0.2s;
	}

	.progress-step.active .progress-dot {
		background: var(--color-accent);
		border-color: var(--color-accent);
		color: white;
	}

	.progress-step.completed .progress-dot {
		background: var(--color-success);
		border-color: var(--color-success);
		color: white;
	}

	.check-mark {
		font-size: 14px;
	}

	.progress-label {
		font-size: 11px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.progress-step.active .progress-label {
		color: var(--color-text);
		font-weight: 500;
	}

	.progress-line {
		flex: 1;
		height: 2px;
		background: var(--color-border);
		margin: 0 4px;
		margin-bottom: 20px;
		transition: background 0.2s;
	}

	.progress-line.filled {
		background: var(--color-success);
	}

	.step-content {
		min-height: 300px;
	}

	.error-banner {
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		border-radius: 8px;
		color: var(--color-danger);
		font-size: 13px;
	}

	.actions {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 10px 20px;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--color-accent);
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.btn-primary:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.btn-secondary {
		background: var(--color-surface);
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.btn-secondary:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.spinner {
		display: inline-flex;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
