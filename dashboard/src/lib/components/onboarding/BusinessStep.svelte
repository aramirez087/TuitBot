<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import TagInput from '$lib/components/settings/TagInput.svelte';

	let productName = $state($onboardingData.product_name);
	let productDescription = $state($onboardingData.product_description);
	let productUrl = $state($onboardingData.product_url);
	let targetAudience = $state($onboardingData.target_audience);
	let productKeywords = $state($onboardingData.product_keywords);
	let industryTopics = $state($onboardingData.industry_topics);

	$effect(() => {
		onboardingData.updateField('product_name', productName);
	});
	$effect(() => {
		onboardingData.updateField('product_description', productDescription);
	});
	$effect(() => {
		onboardingData.updateField('product_url', productUrl);
	});
	$effect(() => {
		onboardingData.updateField('target_audience', targetAudience);
	});
</script>

<div class="step">
	<h2 class="step-title">Business Profile</h2>
	<p class="step-description">
		Tell Tuitbot about your product so it can find relevant conversations and
		generate on-brand content.
	</p>

	<div class="fields">
		<div class="field">
			<label class="field-label" for="product-name">Product Name <span class="required">*</span></label>
			<input
				id="product-name"
				type="text"
				class="field-input"
				placeholder="e.g. Tuitbot"
				bind:value={productName}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="product-desc">Description <span class="required">*</span></label>
			<input
				id="product-desc"
				type="text"
				class="field-input"
				placeholder="A one-line description of what you're building"
				bind:value={productDescription}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="product-url">Product URL <span class="optional">(optional)</span></label>
			<input
				id="product-url"
				type="url"
				class="field-input"
				placeholder="https://example.com"
				bind:value={productUrl}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="target-audience">Target Audience <span class="required">*</span></label>
			<input
				id="target-audience"
				type="text"
				class="field-input"
				placeholder="e.g. indie hackers, SaaS founders, developers"
				bind:value={targetAudience}
			/>
		</div>

		<TagInput
			value={productKeywords}
			label="Discovery Keywords"
			placeholder="Type a keyword and press Enter"
			helpText="Tuitbot searches for tweets containing these keywords."
			onchange={(tags) => {
				productKeywords = tags;
				onboardingData.updateField('product_keywords', tags);
			}}
		/>

		<TagInput
			value={industryTopics}
			label="Content Topics"
			placeholder="Type a topic and press Enter"
			helpText="Topics for original tweets and threads."
			onchange={(tags) => {
				industryTopics = tags;
				onboardingData.updateField('industry_topics', tags);
			}}
		/>
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

	.field-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.field-input::placeholder {
		color: var(--color-text-subtle);
	}
</style>
