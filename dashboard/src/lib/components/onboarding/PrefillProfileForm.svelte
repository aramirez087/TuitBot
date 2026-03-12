<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { onboardingSession } from '$lib/stores/onboarding-session';
	import type { Confidence, InferenceProvenance, InferredField } from '$lib/api/types';
	import { api } from '$lib/api';
	import { trackFunnel } from '$lib/analytics/funnel';
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import { User, Building2, CheckCircle2, AlertTriangle, Info, Loader2, Sparkles } from 'lucide-svelte';

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

	// On mount: if X is connected and LLM is configured, re-run analysis
	// with LLM enrichment to get better target_audience, keywords, and topics.
	let llmEnrichAttempted = false;
	$effect(() => {
		if (llmEnrichAttempted) return;
		const data = $onboardingData;
		const hasLlm = data.llm_provider === 'ollama' ||
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
				onboardingSession.setInferredProfile(
					result.profile,
					result.warnings ?? []
				);
				// Re-read updated store values into local state.
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

	// Pre-compute field metadata for template use (avoids @const in plain elements)
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

	// Track which fields have been edited (fire once per field).
	const editedFields = new Set<string>();
	function trackFieldEdit(fieldName: string) {
		if (!editedFields.has(fieldName)) {
			editedFields.add(fieldName);
			trackFunnel('onboarding:profile-edited', { field: fieldName });
		}
	}

	// Detect if all inferred fields are low confidence (manual entry mode).
	let allLowConfidence = $derived(
		hasInference &&
		profile !== null &&
		[profile.product_name, profile.product_description, profile.target_audience, profile.product_keywords, profile.industry_topics]
			.filter(Boolean)
			.every((f) => f!.confidence === 'low')
	);

	function isLowConfidence(meta: ReturnType<typeof fieldMeta>): boolean {
		return meta !== null && meta.confidence === 'low';
	}
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

	{#if enriching}
		<div class="enrich-banner">
			<span class="spinner"><Loader2 size={14} /></span>
			<span>Enriching profile with AI...</span>
		</div>
	{:else if enrichDone}
		<div class="enrich-banner enrich-done">
			<Sparkles size={14} />
			<span>Profile enriched with AI — review the updated fields below.</span>
		</div>
	{/if}

	{#if xConnected && xUser}
		<div class="connected-card">
			{#if xUser.profile_image_url}
				<img src={xUser.profile_image_url} alt="" class="avatar" />
			{:else}
				<div class="avatar avatar-placeholder"></div>
			{/if}
			<div class="connected-info">
				<span class="display-name">{xUser.name}</span>
				<span class="username">@{xUser.username}</span>
			</div>
			<div class="connected-badge">
				<CheckCircle2 size={16} />
				Connected
			</div>
		</div>
	{/if}

	<div class="account-type-toggle">
		<button
			class="type-btn"
			class:selected={!isBusiness}
			onclick={() => (accountType = 'individual')}
		>
			<User size={16} />
			<span>Individual</span>
		</button>
		<button
			class="type-btn"
			class:selected={isBusiness}
			onclick={() => (accountType = 'business')}
		>
			<Building2 size={16} />
			<span>Business / Product</span>
		</button>
		{#if accountTypeMeta}
			<span class="toggle-provenance" style="color: var(--color-text-subtle)">
				<span class="confidence-dot" style="background: {accountTypeMeta.color}"></span>
				{accountTypeMeta.label}
			</span>
		{/if}
	</div>

	<div class="fields">
		<div class="field">
			<label class="field-label" for="pf-name">
				{isBusiness ? 'Product Name' : 'Name'} <span class="required">*</span>
				{#if nameMeta}
					<span class="field-provenance">
						<span class="confidence-dot" style="background: {nameMeta.color}"></span>
						{nameMeta.label}
					</span>
				{/if}
			</label>
			<input
				id="pf-name"
				type="text"
				class="field-input"
				class:low-confidence={isLowConfidence(nameMeta)}
				placeholder={isBusiness ? 'e.g. Tuitbot' : 'e.g. Alex Rivera'}
				bind:value={productName}
				oninput={() => trackFieldEdit('product_name')}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="pf-desc">
				{isBusiness ? 'Description' : 'Bio'} <span class="required">*</span>
				{#if descMeta}
					<span class="field-provenance">
						<span class="confidence-dot" style="background: {descMeta.color}"></span>
						{descMeta.label}
					</span>
				{/if}
			</label>
			<input
				id="pf-desc"
				type="text"
				class="field-input"
				class:low-confidence={isLowConfidence(descMeta)}
				placeholder={isBusiness
					? "A one-line description of what you're building"
					: 'e.g. Frontend dev who writes about React and design systems'}
				bind:value={productDescription}
				oninput={() => trackFieldEdit('product_description')}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="pf-url">
				{isBusiness ? 'Product URL' : 'Website'} <span class="optional">(optional)</span>
				{#if urlMeta}
					<span class="field-provenance">
						<span class="confidence-dot" style="background: {urlMeta.color}"></span>
						{urlMeta.label}
					</span>
				{/if}
			</label>
			<input
				id="pf-url"
				type="url"
				class="field-input"
				class:low-confidence={isLowConfidence(urlMeta)}
				placeholder={isBusiness ? 'https://yourproduct.com' : 'https://blog.example.com'}
				bind:value={productUrl}
				oninput={() => trackFieldEdit('product_url')}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="pf-audience">
				{isBusiness ? 'Target Audience' : 'Audience'} <span class="required">*</span>
				{#if audMeta}
					<span class="field-provenance">
						<span class="confidence-dot" style="background: {audMeta.color}"></span>
						{audMeta.label}
					</span>
				{/if}
			</label>
			<input
				id="pf-audience"
				type="text"
				class="field-input"
				class:low-confidence={isLowConfidence(audMeta)}
				placeholder={isBusiness
					? 'e.g. indie hackers, SaaS founders, developers'
					: 'e.g. web developers, designers, tech Twitter'}
				bind:value={targetAudience}
				oninput={() => trackFieldEdit('target_audience')}
			/>
		</div>

		<div class="field">
			{#if kwMeta}
				<div class="tag-label-row">
					<span class="field-provenance">
						<span class="confidence-dot" style="background: {kwMeta.color}"></span>
						{kwMeta.label}
					</span>
				</div>
			{/if}
			<TagInput
				value={productKeywords}
				label="Discovery Keywords"
				placeholder="Type a keyword and press Enter"
				helpText={isBusiness
					? 'Tuitbot searches for tweets containing these keywords.'
					: "Keywords that match conversations you'd naturally join."}
				onchange={(tags) => {
					productKeywords = tags;
					onboardingData.updateField('product_keywords', tags);
				}}
			/>
		</div>

		<div class="field">
			{#if topicMeta}
				<div class="tag-label-row">
					<span class="field-provenance">
						<span class="confidence-dot" style="background: {topicMeta.color}"></span>
						{topicMeta.label}
					</span>
				</div>
			{/if}
			<TagInput
				value={industryTopics}
				label="Content Topics"
				placeholder="Type a topic and press Enter"
				helpText={isBusiness
					? 'Topics for original tweets and threads.'
					: 'Topics you want to tweet and share opinions about.'}
				onchange={(tags) => {
					industryTopics = tags;
					onboardingData.updateField('industry_topics', tags);
				}}
			/>
		</div>
	</div>

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

	.connected-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
		border-radius: 8px;
	}

	.avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.avatar-placeholder {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
	}

	.connected-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		flex: 1;
		min-width: 0;
	}

	.display-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.username {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.connected-badge {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-success, #22c55e);
		flex-shrink: 0;
	}

	.account-type-toggle {
		display: flex;
		gap: 8px;
		align-items: center;
		flex-wrap: wrap;
	}

	.type-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 10px 16px;
		border: 2px solid var(--color-border);
		border-radius: 8px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.type-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-text);
	}

	.type-btn.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
		color: var(--color-text);
	}

	.toggle-provenance {
		font-size: 11px;
		display: flex;
		align-items: center;
		gap: 4px;
		flex-basis: 100%;
	}

	.fields {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.required {
		color: var(--color-danger);
	}

	.optional {
		font-weight: 400;
		color: var(--color-text-subtle);
	}

	.field-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		transition: border-color 0.15s;
	}

	.field-input.low-confidence {
		border-color: color-mix(in srgb, var(--color-warning, #eab308) 50%, var(--color-border));
		border-style: dashed;
	}

	.field-input:focus {
		outline: none;
		border-color: var(--color-accent);
		border-style: solid;
	}

	.field-input::placeholder {
		color: var(--color-text-subtle);
	}

	.field-provenance {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 400;
		color: var(--color-text-subtle);
		margin-left: auto;
	}

	.confidence-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		display: inline-block;
		flex-shrink: 0;
	}

	.tag-label-row {
		display: flex;
		justify-content: flex-end;
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

	.enrich-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, var(--color-border));
		border-radius: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.enrich-done {
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border-color: color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
		color: var(--color-success, #22c55e);
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
