<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { onboardingSession } from '$lib/stores/onboarding-session';
	import type { Confidence, InferenceProvenance, InferredField } from '$lib/api/types';
	import { api } from '$lib/api';
	import { trackFunnel } from '$lib/analytics/funnel';
	import { AlertTriangle, Info } from 'lucide-svelte';
	import ProfileContextCard from './ProfileContextCard.svelte';
	import ProfileFormFields from './ProfileFormFields.svelte';

	let accountType = $state($onboardingData.account_type);
	let productName = $state($onboardingData.product_name);
	let productDescription = $state($onboardingData.product_description);
	let productUrl = $state($onboardingData.product_url);
	let targetAudience = $state($onboardingData.target_audience);
	let productKeywords = $state($onboardingData.product_keywords);
	let industryTopics = $state($onboardingData.industry_topics);

	let isBusiness = $derived(accountType === 'business');
	let profile = $derived($onboardingSession.inferred_profile);
	let hasInference = $derived(profile !== null);
	let warnings = $derived($onboardingSession.analysis_warnings);
	let xUser = $derived($onboardingSession.x_user);
	let xConnected = $derived($onboardingSession.x_connected);

	let enriching = $state(false);
	let enrichDone = $state(false);

	let llmEnrichAttempted = false;
	$effect(() => {
		if (llmEnrichAttempted) return;
		const data = $onboardingData;
		const hasLlm =
			data.llm_provider === 'ollama' ||
			(data.llm_api_key.trim().length > 0 && data.llm_model.trim().length > 0);
		if ($onboardingSession.x_connected && hasLlm) {
			llmEnrichAttempted = true;
			runLlmEnrichment(data);
		}
	});

	async function runLlmEnrichment(data: typeof $onboardingData) {
		enriching = true;
		try {
			const result = await api.onboarding.analyzeProfile({
				provider: data.llm_provider,
				...(data.llm_api_key ? { api_key: data.llm_api_key } : {}),
				model: data.llm_model,
				...(data.llm_base_url ? { base_url: data.llm_base_url } : {}),
			});
			if (result.profile && result.status === 'ok') {
				onboardingData.prefillFromInference(result.profile);
				onboardingSession.setInferredProfile(result.profile, result.warnings ?? []);
				const updated = $onboardingData;
				accountType = updated.account_type;
				productName = updated.product_name;
				productDescription = updated.product_description;
				productUrl = updated.product_url;
				targetAudience = updated.target_audience;
				productKeywords = updated.product_keywords;
				industryTopics = updated.industry_topics;
				enrichDone = true;
				trackFunnel('onboarding:llm-enrich-done');
			}
		} catch {
			// Non-fatal — heuristic values remain.
		}
		enriching = false;
	}

	let accountTypeMeta = $derived(fieldMeta(profile?.account_type));
	let nameMeta = $derived(fieldMeta(profile?.product_name));
	let descMeta = $derived(fieldMeta(profile?.product_description));
	let urlMeta = $derived(fieldMeta(profile?.product_url));
	let audMeta = $derived(fieldMeta(profile?.target_audience));
	let kwMeta = $derived(fieldMeta(profile?.product_keywords));
	let topicMeta = $derived(fieldMeta(profile?.industry_topics));

	$effect(() => { onboardingData.updateField('account_type', accountType); });
	$effect(() => { onboardingData.updateField('product_name', productName); });
	$effect(() => { onboardingData.updateField('product_description', productDescription); });
	$effect(() => { onboardingData.updateField('product_url', productUrl); });
	$effect(() => { onboardingData.updateField('target_audience', targetAudience); });

	function confidenceColor(c: Confidence): string {
		switch (c) {
			case 'high': return 'var(--color-success, #22c55e)';
			case 'medium': return 'var(--color-warning, #eab308)';
			case 'low': return 'var(--color-text-subtle)';
		}
	}

	function provenanceLabel(p: InferenceProvenance): string {
		switch (p) {
			case 'bio': return 'from bio';
			case 'tweets': return 'from tweets';
			case 'bio_and_tweets': return 'from bio + tweets';
			case 'profile_url': return 'from profile URL';
			case 'display_name': return 'from display name';
			case 'default': return 'default';
		}
	}

	function fieldMeta(field: InferredField<unknown> | undefined) {
		if (!field || !hasInference) return null;
		return {
			color: confidenceColor(field.confidence),
			label: provenanceLabel(field.provenance),
			confidence: field.confidence,
		};
	}

	const editedFields = new Set<string>();
	function trackFieldEdit(fieldName: string) {
		if (!editedFields.has(fieldName)) {
			editedFields.add(fieldName);
			trackFunnel('onboarding:profile-edited', { field: fieldName });
		}
	}

	let allLowConfidence = $derived(
		hasInference &&
			profile !== null &&
			[
				profile.product_name,
				profile.product_description,
				profile.target_audience,
				profile.product_keywords,
				profile.industry_topics,
			]
				.filter(Boolean)
				.every((f) => f!.confidence === 'low'),
	);
</script>

<div class="step">
	<h2 class="step-title">
		{hasInference ? 'Review Your Profile' : 'Your Profile'}
	</h2>
	<p class="step-description">
		{#if hasInference && allLowConfidence}
			Tell us about your product or personal brand. We couldn't infer much from your profile.
		{:else if hasInference}
			We analyzed your X account and pre-filled these fields. Edit anything that doesn't look right.
		{:else}
			Tell Tuitbot about yourself so it can find relevant conversations and generate on-brand content.
		{/if}
	</p>

	<ProfileContextCard
		{xConnected}
		{xUser}
		{accountType}
		{accountTypeMeta}
		{enriching}
		{enrichDone}
		onAccountTypeChange={(type) => (accountType = type)}
	/>

	<ProfileFormFields
		{isBusiness}
		bind:productName
		bind:productDescription
		bind:productUrl
		bind:targetAudience
		bind:productKeywords
		bind:industryTopics
		{nameMeta}
		{descMeta}
		{urlMeta}
		{audMeta}
		{kwMeta}
		{topicMeta}
		onTagChange={(field, tags) => onboardingData.updateField(field, tags)}
		onTrackEdit={trackFieldEdit}
	/>

	{#if warnings.length > 0}
		<div class="warning-banner">
			<AlertTriangle size={16} />
			<div class="warning-text">
				{#each warnings as warning}
					<p>{warning}</p>
				{/each}
			</div>
		</div>
	{:else if !hasInference}
		<div class="info-banner">
			<Info size={16} />
			<span>Enter your details to help Tuitbot find relevant conversations.</span>
		</div>
	{/if}
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

	.warning-banner {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 12px 14px;
		border: 1px solid color-mix(in srgb, var(--color-warning, #eab308) 30%, var(--color-border));
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-warning, #eab308) 6%, var(--color-surface));
		color: var(--color-warning, #eab308);
		font-size: 13px;
		line-height: 1.5;
	}

	.warning-text {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.warning-text p {
		margin: 0;
	}

	.info-banner {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 14px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 13px;
	}
</style>
