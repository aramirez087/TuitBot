<script lang="ts">
	import { Brain, Eye, EyeOff } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import ConnectionTest from '$lib/components/settings/ConnectionTest.svelte';
	import { draft, updateDraft, testLlmConnection } from '$lib/stores/settings';

	let showApiKey = $state(false);

	const modelSuggestions: Record<string, string> = {
		openai: 'gpt-4o-mini',
		anthropic: 'claude-sonnet-4-5-20250514',
		ollama: 'llama3.1'
	};

	const baseUrlPlaceholders: Record<string, string> = {
		openai: 'https://api.openai.com/v1',
		anthropic: 'https://api.anthropic.com/v1',
		ollama: 'http://localhost:11434/v1'
	};
</script>

{#if $draft}
<SettingsSection
	id="llm"
	title="LLM Provider"
	description="AI model configuration for content generation"
	scope="instance"
	icon={Brain}
>
	<div class="field-grid">
		<div class="field">
			<label class="field-label" for="llm_provider">Provider</label>
			<select
				id="llm_provider"
				class="select-input"
				value={$draft.llm.provider}
				onchange={(e) =>
					updateDraft('llm.provider', e.currentTarget.value)}
			>
				<option value="">Select provider...</option>
				<option value="openai">OpenAI</option>
				<option value="anthropic">Anthropic</option>
				<option value="ollama">Ollama</option>
			</select>
		</div>

		<div class="field">
			<label class="field-label" for="llm_model">Model</label>
			<input
				id="llm_model"
				type="text"
				class="text-input"
				value={$draft.llm.model}
				oninput={(e) =>
					updateDraft('llm.model', e.currentTarget.value)}
				placeholder={modelSuggestions[$draft.llm.provider] ?? 'Model name'}
			/>
			{#if $draft.llm.provider && modelSuggestions[$draft.llm.provider]}
				<span class="field-hint">
					Suggested: {modelSuggestions[$draft.llm.provider]}
				</span>
			{/if}
		</div>

		{#if $draft.llm.provider === 'openai' || $draft.llm.provider === 'anthropic'}
			<div class="field full-width">
				<label class="field-label" for="llm_api_key">API Key</label>
				<div class="password-wrapper">
					<input
						id="llm_api_key"
						type={showApiKey ? 'text' : 'password'}
						class="text-input password-input"
						value={$draft.llm.api_key ?? ''}
						oninput={(e) =>
							updateDraft(
								'llm.api_key',
								e.currentTarget.value || null
							)}
						placeholder="sk-..."
					/>
					<button
						type="button"
						class="password-toggle"
						onclick={() => (showApiKey = !showApiKey)}
						aria-label={showApiKey ? 'Hide' : 'Show'}
					>
						{#if showApiKey}
							<EyeOff size={16} />
						{:else}
							<Eye size={16} />
						{/if}
					</button>
				</div>
			</div>
		{/if}

		<div class="field full-width">
			<label class="field-label" for="llm_base_url">Base URL</label>
			<input
				id="llm_base_url"
				type="text"
				class="text-input"
				value={$draft.llm.base_url ?? ''}
				oninput={(e) =>
					updateDraft(
						'llm.base_url',
						e.currentTarget.value || null
					)}
				placeholder={baseUrlPlaceholders[$draft.llm.provider] ??
					'Custom endpoint URL'}
			/>
			<span class="field-hint">
				Leave empty to use the default endpoint
			</span>
		</div>

		<div class="field full-width">
			<ConnectionTest
				label="Test Connection"
				ontest={testLlmConnection}
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

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.text-input,
	.select-input {
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
	.select-input:focus {
		border-color: var(--color-accent);
	}

	.select-input {
		cursor: pointer;
		appearance: auto;
	}

	.password-wrapper {
		position: relative;
		display: flex;
	}

	.password-input {
		flex: 1;
		padding-right: 40px;
	}

	.password-toggle {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: none;
		color: var(--color-text-muted);
		cursor: pointer;
		border-radius: 4px;
		transition: color 0.15s;
	}

	.password-toggle:hover {
		color: var(--color-text);
	}
</style>
