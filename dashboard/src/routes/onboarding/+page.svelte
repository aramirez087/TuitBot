<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { api } from '$lib/api';
	import { onboardingData } from '$lib/stores/onboarding';
	import { authMode as authModeStore } from '$lib/stores/auth';
	import { trackFunnel } from '$lib/analytics/funnel';
	import WelcomeStep from '$lib/components/onboarding/WelcomeStep.svelte';
	import XApiStep from '$lib/components/onboarding/XApiStep.svelte';
	import LlmStep from '$lib/components/onboarding/LlmStep.svelte';
	import PrefillProfileForm from '$lib/components/onboarding/PrefillProfileForm.svelte';
	import LanguageBrandStep from '$lib/components/onboarding/LanguageBrandStep.svelte';
	import SourcesStep from '$lib/components/onboarding/SourcesStep.svelte';
	import ValidationStep from '$lib/components/onboarding/ValidationStep.svelte';
	import ReviewStep from '$lib/components/onboarding/ReviewStep.svelte';
	import ClaimStep from '$lib/components/onboarding/ClaimStep.svelte';
	import { Zap, RefreshCw } from 'lucide-svelte';
	import { onboardingSession } from '$lib/stores/onboarding-session';
	import { submitOnboarding } from '$lib/utils/submitOnboarding';
	import OnboardingStepNav from './OnboardingStepNav.svelte';
	import OnboardingActions from './OnboardingActions.svelte';

	// Optional steps that can be skipped during progressive activation.
	const OPTIONAL_STEPS = new Set(['LLM', 'Language', 'Vault', 'Validate']);

	// Step flow varies by mode:
	// API mode: Welcome → X Access → LLM → Profile → Language → Vault → Validate → Review [→ Secure]
	// Scraper mode: Welcome → X Access → Profile → LLM → Language → Vault → Validate → Review [→ Secure]
	let isScraperMode = $derived($onboardingData.provider_backend === 'scraper');
	let baseSteps = $derived(
		isScraperMode
			? ['Welcome', 'X Access', 'Profile', 'LLM', 'Language', 'Vault', 'Validate', 'Review']
			: ['Welcome', 'X Access', 'LLM', 'Profile', 'Language', 'Vault', 'Validate', 'Review']
	);

	let currentStep = $state(0);
	let submitting = $state(false);
	let errorMsg = $state('');
	let skippedSteps = $state(new Set<string>());
	let hasServerClientId = $state(false);

	$effect(() => {
		api.settings.configStatus().then((status) => {
			hasServerClientId = status.has_x_client_id;
		}).catch(() => { /* non-fatal */ });
	});

	let claimPassphrase = $state('');
	let passphraseSaved = $state(false);

	let isTauri = $derived($authModeStore === 'tauri');
	let alreadyClaimed = $derived($page.url.searchParams.get('claimed') === '1');
	let showClaimStep = $derived(!isTauri);
	let steps = $derived(showClaimStep ? [...baseSteps, 'Secure'] : baseSteps);
	let currentStepName = $derived(steps[currentStep] ?? '');
	let isLastStep = $derived(currentStep === steps.length - 1);
	let isClaimStep = $derived(showClaimStep && currentStep === steps.length - 1);

	let canSkipToFinish = $derived(
		currentStepName === 'Profile' || OPTIONAL_STEPS.has(currentStepName)
	);

	let hasLlmConfig = $derived(
		$onboardingData.llm_provider === 'ollama' ||
		($onboardingData.llm_api_key.trim().length > 0 && $onboardingData.llm_model.trim().length > 0)
	);

	let unsupportedMode = $state('');

	let prevTrackedStep = $state(-1);
	$effect(() => {
		if (currentStep !== prevTrackedStep) {
			if (currentStep === 0 && prevTrackedStep === -1) {
				trackFunnel('onboarding:started', { mode: isScraperMode ? 'scraper' : 'api' });
			}
			if (currentStep > 0) {
				trackFunnel('onboarding:step-entered', { step: currentStepName, index: currentStep });
			}
			prevTrackedStep = currentStep;
		}
	});

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
			case 'Welcome': return true;
			case 'X Access':
				if (data.provider_backend === 'scraper') return true;
				if (hasServerClientId) return $onboardingSession.x_connected;
				return data.client_id.trim().length > 0;
			case 'Profile':
				return (
					data.product_name.trim().length > 0 &&
					data.product_description.trim().length > 0 &&
					data.target_audience.trim().length > 0 &&
					data.product_keywords.length > 0 &&
					data.industry_topics.length > 0
				);
			case 'LLM': return true;
			case 'Language': return true;
			case 'Vault': return true;
			case 'Validate': return true;
			case 'Review': return true;
			case 'Secure':
				if (alreadyClaimed) return passphraseSaved;
				return claimPassphrase.trim().length >= 8 && passphraseSaved;
			default: return false;
		}
	}

	function next() {
		if (currentStep < steps.length - 1) { currentStep++; errorMsg = ''; }
	}

	function back() {
		if (currentStep > 0) { currentStep--; errorMsg = ''; }
	}

	function skipToFinish() {
		const reviewIdx = steps.indexOf('Review');
		if (reviewIdx < 0) return;
		const skipped: string[] = [];
		for (let i = currentStep; i < reviewIdx; i++) {
			const stepName = steps[i];
			if (OPTIONAL_STEPS.has(stepName)) {
				skippedSteps = new Set([...skippedSteps, stepName]);
				skipped.push(stepName);
			}
		}
		trackFunnel('onboarding:step-skipped', { from_step: currentStepName, skipped });
		currentStep = reviewIdx;
		errorMsg = '';
	}

	async function submit() {
		submitting = true;
		errorMsg = '';
		try {
			const result = await submitOnboarding({
				data: $onboardingData,
				showClaimStep,
				claimPassphrase,
				alreadyClaimed,
			});
			if (result.kind === 'error') { errorMsg = result.message; return; }
			onboardingData.reset();
			goto(result.to);
		} finally {
			submitting = false;
		}
	}
</script>

{#if unsupportedMode}
	<div class="onboarding">
		<div class="onboarding-header">
			<div class="logo">
				<Zap size={20} strokeWidth={2.5} />
				<span class="logo-text">Tuitbot</span>
			</div>
		</div>
		<div class="onboarding-content">
			<div class="unsupported-banner" role="alert">
				<p>This deployment mode (<code>{unsupportedMode}</code>) is not supported for browser onboarding.</p>
				<p>Use <code>tuitbot init</code> from the command line instead.</p>
			</div>
		</div>
	</div>
{:else}
	<div class="onboarding">
		<div class="onboarding-header">
			<div class="logo">
				<Zap size={20} strokeWidth={2.5} />
				<span class="logo-text">Tuitbot</span>
			</div>
		</div>

		<div class="onboarding-content">
			<OnboardingStepNav {steps} {currentStep} {skippedSteps} />

			<div class="step-content">
				{#if currentStepName === 'Welcome'}
					<WelcomeStep />
				{:else if currentStepName === 'X Access'}
					<XApiStep {hasServerClientId} />
				{:else if currentStepName === 'LLM'}
					<LlmStep />
				{:else if currentStepName === 'Profile'}
					<PrefillProfileForm />
				{:else if currentStepName === 'Language'}
					<LanguageBrandStep />
				{:else if currentStepName === 'Vault'}
					<SourcesStep />
				{:else if currentStepName === 'Validate'}
					<ValidationStep {hasLlmConfig} />
				{:else if currentStepName === 'Review'}
					<ReviewStep {skippedSteps} />
				{:else if currentStepName === 'Secure'}
					<ClaimStep bind:passphrase={claimPassphrase} bind:saved={passphraseSaved} {alreadyClaimed} />
				{/if}
			</div>

			{#if errorMsg}
				<div class="error-banner" role="alert">
					<span>{errorMsg}</span>
					<button type="button" class="error-retry-btn" onclick={submit} disabled={submitting}>
						<RefreshCw size={14} />
						Retry
					</button>
				</div>
			{/if}

			<OnboardingActions
				{currentStep}
				{isLastStep}
				{isClaimStep}
				{canSkipToFinish}
				advanceAllowed={canAdvance()}
				{submitting}
				onBack={back}
				onNext={next}
				onSkip={skipToFinish}
				onSubmit={submit}
			/>
		</div>
	</div>
{/if}

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

	.step-content {
		min-height: 300px;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		border-radius: 8px;
		color: var(--color-danger);
		font-size: 13px;
	}

	.error-retry-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: 1px solid color-mix(in srgb, var(--color-danger) 40%, transparent);
		border-radius: 6px;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.15s;
	}

	.error-retry-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.error-retry-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.unsupported-banner {
		padding: 24px;
		background: color-mix(in srgb, var(--color-warning) 8%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-warning) 25%, var(--color-border));
		border-radius: 10px;
		text-align: center;
	}

	.unsupported-banner p {
		margin: 0 0 8px;
		font-size: 14px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.unsupported-banner p:last-child {
		margin-bottom: 0;
	}

	.unsupported-banner code {
		background: var(--color-surface);
		padding: 2px 6px;
		border-radius: 4px;
		font-size: 13px;
	}
</style>
