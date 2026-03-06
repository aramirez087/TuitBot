<script lang="ts">
	import { Briefcase } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import { defaults, draft, fieldErrors, updateDraft } from '$lib/stores/settings';
</script>

{#if $draft}
<SettingsSection
	id="business"
	title="Business Profile"
	description="Your product details and keywords for discovery"
	icon={Briefcase}
	scope="account"
	scopeKey="business"
>
	<div class="field-grid">
		<div class="field">
			<label class="field-label" for="product_name">
				Product Name <span class="required">*</span>
			</label>
			<input
				id="product_name"
				type="text"
				class="text-input"
				class:has-error={$fieldErrors['business.product_name']}
				value={$draft.business.product_name}
				oninput={(e) => updateDraft('business.product_name', e.currentTarget.value)}
				placeholder="My Product"
			/>
			{#if $fieldErrors['business.product_name']}
				<span class="field-error">{$fieldErrors['business.product_name']}</span>
			{/if}
		</div>

		<div class="field">
			<label class="field-label" for="product_url">Product URL</label>
			<input
				id="product_url"
				type="url"
				class="text-input"
				value={$draft.business.product_url ?? ''}
				oninput={(e) =>
					updateDraft(
						'business.product_url',
						e.currentTarget.value || null
					)}
				placeholder="https://example.com"
			/>
		</div>

		<div class="field full-width">
			<label class="field-label" for="product_description">
				Product Description <span class="required">*</span>
			</label>
			<textarea
				id="product_description"
				class="textarea-input"
				class:has-error={$fieldErrors['business.product_description']}
				value={$draft.business.product_description}
				oninput={(e) =>
					updateDraft('business.product_description', e.currentTarget.value)}
				placeholder="A one-line description of what your product does"
				rows="2"
			></textarea>
			{#if $fieldErrors['business.product_description']}
				<span class="field-error">{$fieldErrors['business.product_description']}</span>
			{/if}
		</div>

		<div class="field full-width">
			<label class="field-label" for="target_audience">
				Target Audience <span class="required">*</span>
			</label>
			<textarea
				id="target_audience"
				class="textarea-input"
				value={$draft.business.target_audience}
				oninput={(e) =>
					updateDraft('business.target_audience', e.currentTarget.value)}
				placeholder="Describe who your product is for"
				rows="2"
			></textarea>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.business.product_keywords}
				label="Product Keywords"
				placeholder="Add keywords and press Enter"
				helpText="Keywords used for tweet discovery"
				error={$fieldErrors['business.product_keywords'] ?? ''}
				defaultValue={$defaults?.business.product_keywords}
				onchange={(tags) => updateDraft('business.product_keywords', tags)}
			/>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.business.competitor_keywords}
				label="Competitor Keywords"
				placeholder="Add competitor keywords"
				helpText="Keywords related to competitors for discovery"
				defaultValue={$defaults?.business.competitor_keywords}
				onchange={(tags) =>
					updateDraft('business.competitor_keywords', tags)}
			/>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.business.industry_topics}
				label="Industry Topics"
				placeholder="Add topics and press Enter"
				helpText="Topics for content generation"
				error={$fieldErrors['business.industry_topics'] ?? ''}
				defaultValue={$defaults?.business.industry_topics}
				onchange={(tags) =>
					updateDraft('business.industry_topics', tags)}
			/>
		</div>
	</div>
</SettingsSection>
{/if}

<style>
	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.required {
		color: var(--color-danger);
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.text-input,
	.textarea-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus,
	.textarea-input:focus {
		border-color: var(--color-accent);
	}

	.text-input.has-error,
	.textarea-input.has-error {
		border-color: var(--color-danger);
	}

	.textarea-input {
		resize: vertical;
		min-height: 60px;
		line-height: 1.5;
	}
</style>
