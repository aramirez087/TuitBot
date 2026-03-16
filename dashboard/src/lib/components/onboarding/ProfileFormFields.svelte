<script lang="ts">
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import type { Confidence } from '$lib/api/types';

	export interface FieldMeta {
		color: string;
		label: string;
		confidence: Confidence;
	}

	interface Props {
		isBusiness: boolean;
		productName: string;
		productDescription: string;
		productUrl: string;
		targetAudience: string;
		productKeywords: string[];
		industryTopics: string[];
		nameMeta: FieldMeta | null;
		descMeta: FieldMeta | null;
		urlMeta: FieldMeta | null;
		audMeta: FieldMeta | null;
		kwMeta: FieldMeta | null;
		topicMeta: FieldMeta | null;
		onTagChange: (field: 'product_keywords' | 'industry_topics', tags: string[]) => void;
		onTrackEdit: (fieldName: string) => void;
	}

	let {
		isBusiness,
		productName = $bindable(),
		productDescription = $bindable(),
		productUrl = $bindable(),
		targetAudience = $bindable(),
		productKeywords = $bindable(),
		industryTopics = $bindable(),
		nameMeta,
		descMeta,
		urlMeta,
		audMeta,
		kwMeta,
		topicMeta,
		onTagChange,
		onTrackEdit,
	}: Props = $props();

	function isLowConfidence(meta: FieldMeta | null): boolean {
		return meta !== null && meta.confidence === 'low';
	}
</script>

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
			oninput={() => onTrackEdit('product_name')}
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
			oninput={() => onTrackEdit('product_description')}
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
			oninput={() => onTrackEdit('product_url')}
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
			oninput={() => onTrackEdit('target_audience')}
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
				onTagChange('product_keywords', tags);
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
				onTagChange('industry_topics', tags);
			}}
		/>
	</div>
</div>

<style>
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
</style>
