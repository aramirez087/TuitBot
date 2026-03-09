<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { onboardingSession } from '$lib/stores/onboarding-session';
	import { api } from '$lib/api';
	import { trackFunnel } from '$lib/analytics/funnel';
	import { Loader2, CheckCircle2, AlertTriangle } from 'lucide-svelte';

	interface Props {
		oncomplete: () => void;
	}

	let { oncomplete }: Props = $props();

	let phase = $state(0);
	let error = $state('');
	let done = $state(false);

	const phases = [
		'Fetching your X profile...',
		'Reading your recent tweets...',
		'Analyzing your content strategy...'
	];

	$effect(() => {
		if (done || $onboardingSession.analyzing) return;
		runAnalysis();
	});

	let sparseProfile = $state(false);

	async function runAnalysis() {
		onboardingSession.setAnalyzing(true);
		error = '';
		phase = 0;
		sparseProfile = false;

		const hasLlm = !!$onboardingData.llm_api_key || $onboardingData.llm_provider === 'ollama';
		trackFunnel('onboarding:analysis-started', { has_llm: hasLlm });

		const t1 = setTimeout(() => { phase = 1; }, 800);
		const t2 = setTimeout(() => { phase = 2; }, 1800);

		try {
			const data = $onboardingData;
			const llmConfig = data.llm_api_key || data.llm_provider === 'ollama'
				? {
					provider: data.llm_provider,
					api_key: data.llm_api_key || undefined,
					model: data.llm_model,
					base_url: data.llm_base_url || undefined
				}
				: undefined;

			const result = await api.onboarding.analyzeProfile(llmConfig);

			clearTimeout(t1);
			clearTimeout(t2);
			phase = 2;

			if (result.status === 'x_api_error') {
				error = "Couldn't access your X account. Your tokens may have expired.";
				trackFunnel('onboarding:analysis-error', { error: 'x_api_error' });
				onboardingSession.setAnalyzing(false);
				return;
			}

			if (result.status === 'llm_error') {
				// LLM failed but we may have partial data — auto-continue.
				if (result.profile) {
					onboardingSession.setInferredProfile(result.profile, result.warnings ?? []);
					onboardingData.prefillFromInference(result.profile);
				}
				trackFunnel('onboarding:analysis-success', {
					status: 'llm_error_partial',
					fields_inferred: result.profile ? Object.keys(result.profile).length : 0,
					warnings: result.warnings?.length ?? 0,
				});
				done = true;
				onboardingSession.setAnalyzing(false);
				await new Promise((r) => setTimeout(r, 600));
				oncomplete();
				return;
			}

			if (result.profile) {
				onboardingSession.setInferredProfile(result.profile, result.warnings ?? []);
				onboardingData.prefillFromInference(result.profile);

				// Detect sparse profiles (all low confidence or partial status).
				if (result.status === 'partial') {
					sparseProfile = true;
				}
			}

			trackFunnel('onboarding:analysis-success', {
				status: result.status,
				fields_inferred: result.profile ? Object.keys(result.profile).length : 0,
				warnings: result.warnings?.length ?? 0,
			});

			done = true;
			onboardingSession.setAnalyzing(false);

			await new Promise((r) => setTimeout(r, 600));
			oncomplete();
		} catch (e) {
			clearTimeout(t1);
			clearTimeout(t2);
			const msg = e instanceof Error ? e.message : 'Analysis failed';
			// Differentiate network errors from other failures.
			if (e instanceof TypeError && msg.includes('fetch')) {
				error = "Can't reach the server. Check your connection.";
			} else {
				error = msg;
			}
			trackFunnel('onboarding:analysis-error', { error });
			onboardingSession.setAnalyzing(false);
		}
	}

	function continueManually() {
		trackFunnel('onboarding:analysis-skipped', { had_error: !!error });
		done = true;
		oncomplete();
	}

	function retry() {
		error = '';
		done = false;
		runAnalysis();
	}
</script>

<div class="step">
	<h2 class="step-title">Analyzing Your Profile</h2>
	<p class="step-description">
		Analyzing your X profile to pre-fill your business details. This only takes a moment.
	</p>

	<div class="progress-card">
		{#each phases as label, i}
			<div class="phase-item" class:active={phase === i && !error && !done} class:done={phase > i || done}>
				<div class="phase-icon">
					{#if phase > i || done}
						<CheckCircle2 size={20} />
					{:else if phase === i && !error}
						<span class="spinner"><Loader2 size={20} /></span>
					{:else}
						<div class="phase-dot"></div>
					{/if}
				</div>
				<span class="phase-label">{label}</span>
			</div>
		{/each}
	</div>

	{#if error}
		<div class="error-card">
			<div class="error-header">
				<AlertTriangle size={18} />
				<span>{error}</span>
			</div>
			<div class="error-actions">
				<button type="button" class="btn-retry" onclick={retry}>Try again</button>
				<button type="button" class="btn-skip" onclick={continueManually}>Continue manually</button>
			</div>
		</div>
	{/if}

	{#if sparseProfile && done}
		<div class="sparse-notice">
			We couldn't find much on your profile. You'll fill in the details next.
		</div>
	{/if}
</div>

<style>
	.step {
		display: flex;
		flex-direction: column;
		gap: 24px;
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

	.progress-card {
		padding: 24px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 10px;
		background: var(--color-surface);
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.phase-item {
		display: flex;
		align-items: center;
		gap: 14px;
		color: var(--color-text-subtle);
		transition: color 0.3s;
	}

	.phase-item.active {
		color: var(--color-text);
	}

	.phase-item.done {
		color: var(--color-success, #22c55e);
	}

	.phase-icon {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.phase-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-border);
	}

	.phase-label {
		font-size: 14px;
		font-weight: 500;
	}

	.spinner {
		display: inline-flex;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.error-card {
		padding: 16px;
		border: 1px solid color-mix(in srgb, var(--color-warning, #eab308) 30%, var(--color-border));
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-warning, #eab308) 6%, var(--color-surface));
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.error-header {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		color: var(--color-warning, #eab308);
		font-size: 13px;
		line-height: 1.5;
	}

	.error-actions {
		display: flex;
		gap: 10px;
	}

	.btn-retry,
	.btn-skip {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.btn-retry {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text);
	}

	.btn-retry:hover {
		background: var(--color-surface-hover);
	}

	.btn-skip {
		background: transparent;
		border: 1px solid transparent;
		color: var(--color-text-muted);
	}

	.btn-skip:hover {
		color: var(--color-text);
	}

	.sparse-notice {
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-warning, #eab308) 6%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-warning, #eab308) 20%, var(--color-border));
		border-radius: 8px;
		color: var(--color-text-muted);
		font-size: 13px;
		line-height: 1.5;
	}
</style>
