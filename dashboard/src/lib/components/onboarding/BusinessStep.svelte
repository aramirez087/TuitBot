<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import { User, Building2 } from 'lucide-svelte';

	let accountType = $state($onboardingData.account_type);
	let productName = $state($onboardingData.product_name);
	let productDescription = $state($onboardingData.product_description);
	let productUrl = $state($onboardingData.product_url);
	let targetAudience = $state($onboardingData.target_audience);
	let productKeywords = $state($onboardingData.product_keywords);
	let industryTopics = $state($onboardingData.industry_topics);

	let isBusiness = $derived(accountType === 'business');

	$effect(() => {
		onboardingData.updateField('account_type', accountType);
	});
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
	<h2 class="step-title">Your Profile</h2>
	<p class="step-description">
		Tell Tuitbot about yourself so it can find relevant conversations and
		generate on-brand content.
	</p>

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
	</div>

	<div class="fields">
		<div class="field">
			<label class="field-label" for="product-name">
				{isBusiness ? 'Product Name' : 'Name'} <span class="required">*</span>
			</label>
			<input
				id="product-name"
				type="text"
				class="field-input"
				placeholder={isBusiness ? 'e.g. Tuitbot' : 'e.g. Alex Rivera'}
				bind:value={productName}
			/>
			{#if !isBusiness}
				<span class="field-hint">Your name or the handle you go by on X.</span>
			{/if}
		</div>

		<div class="field">
			<label class="field-label" for="product-desc">
				{isBusiness ? 'Description' : 'Bio'} <span class="required">*</span>
			</label>
			<input
				id="product-desc"
				type="text"
				class="field-input"
				placeholder={isBusiness
					? 'A one-line description of what you\'re building'
					: 'e.g. Frontend dev who writes about React and design systems'}
				bind:value={productDescription}
			/>
			{#if !isBusiness}
				<span class="field-hint">What do you tweet about? This shapes your content voice.</span>
			{/if}
		</div>

		<div class="field">
			<label class="field-label" for="product-url">
				{isBusiness ? 'Product URL' : 'Website'} <span class="optional">(optional)</span>
			</label>
			<input
				id="product-url"
				type="url"
				class="field-input"
				placeholder={isBusiness ? 'https://yourproduct.com' : 'https://blog.example.com'}
				bind:value={productUrl}
			/>
			{#if !isBusiness}
				<span class="field-hint">Blog, portfolio, or personal site.</span>
			{/if}
		</div>

		<div class="field">
			<label class="field-label" for="target-audience">
				{isBusiness ? 'Target Audience' : 'Audience'} <span class="required">*</span>
			</label>
			<input
				id="target-audience"
				type="text"
				class="field-input"
				placeholder={isBusiness
					? 'e.g. indie hackers, SaaS founders, developers'
					: 'e.g. web developers, designers, tech Twitter'}
				bind:value={targetAudience}
			/>
			{#if !isBusiness}
				<span class="field-hint">Who do you want to connect with?</span>
			{/if}
		</div>

		<TagInput
			value={productKeywords}
			label="Discovery Keywords"
			placeholder="Type a keyword and press Enter"
			helpText={isBusiness
				? 'Tuitbot searches for tweets containing these keywords.'
				: 'Keywords that match conversations you\'d naturally join.'}
			onchange={(tags) => {
				productKeywords = tags;
				onboardingData.updateField('product_keywords', tags);
			}}
		/>

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

	.account-type-toggle {
		display: flex;
		gap: 8px;
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

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}
</style>
