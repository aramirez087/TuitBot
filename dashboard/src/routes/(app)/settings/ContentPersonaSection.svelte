<script lang="ts">
	import { MessageCircle } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import { draft, updateDraft } from '$lib/stores/settings';
</script>

{#if $draft}
<SettingsSection
	id="persona"
	title="Content Persona"
	description="Shape the personality and voice of your generated content"
	icon={MessageCircle}
	scope="account"
	scopeKey="business"
>
	<div class="field-grid">
		<div class="field full-width">
			<label class="field-label" for="brand_voice">Brand Voice</label>
			<textarea
				id="brand_voice"
				class="textarea-input"
				value={$draft.business.brand_voice ?? ''}
				oninput={(e) =>
					updateDraft(
						'business.brand_voice',
						e.currentTarget.value || null
					)}
				placeholder="Describe the personality and tone for all generated content"
				rows="3"
			></textarea>
		</div>

		<div class="field full-width">
			<label class="field-label" for="reply_style">Reply Style</label>
			<textarea
				id="reply_style"
				class="textarea-input"
				value={$draft.business.reply_style ?? ''}
				oninput={(e) =>
					updateDraft(
						'business.reply_style',
						e.currentTarget.value || null
					)}
				placeholder="Style guidelines specific to replies"
				rows="2"
			></textarea>
		</div>

		<div class="field full-width">
			<label class="field-label" for="content_style">Content Style</label>
			<textarea
				id="content_style"
				class="textarea-input"
				value={$draft.business.content_style ?? ''}
				oninput={(e) =>
					updateDraft(
						'business.content_style',
						e.currentTarget.value || null
					)}
				placeholder="Style guidelines for original tweets and threads"
				rows="2"
			></textarea>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.business.persona_opinions}
				label="Persona Opinions"
				placeholder="Add opinions the persona holds"
				helpText="Used to add variety and authenticity to content"
				onchange={(tags) =>
					updateDraft('business.persona_opinions', tags)}
			/>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.business.persona_experiences}
				label="Persona Experiences"
				placeholder="Add experiences the persona can reference"
				helpText="Keeps content authentic and relatable"
				onchange={(tags) =>
					updateDraft('business.persona_experiences', tags)}
			/>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.business.content_pillars}
				label="Content Pillars"
				placeholder="Add core themes"
				helpText="Broad themes the account focuses on"
				onchange={(tags) =>
					updateDraft('business.content_pillars', tags)}
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
		resize: vertical;
		min-height: 60px;
		line-height: 1.5;
	}

	.textarea-input:focus {
		border-color: var(--color-accent);
	}
</style>
