<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import ConnectionTest from '$lib/components/settings/ConnectionTest.svelte';
	import { api } from '$lib/api';

	let provider = $state($onboardingData.llm_provider);
	let apiKey = $state($onboardingData.llm_api_key);
	let model = $state($onboardingData.llm_model);
	let baseUrl = $state($onboardingData.llm_base_url);

	const providers = [
		{ value: 'openai', label: 'OpenAI', defaultModel: 'gpt-4o-mini' },
		{ value: 'anthropic', label: 'Anthropic', defaultModel: 'claude-sonnet-4-6' },
		{ value: 'ollama', label: 'Ollama (local)', defaultModel: 'llama3.2' },
	];

	function onProviderChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		provider = value;
		onboardingData.updateField('llm_provider', value);

		const p = providers.find((p) => p.value === value);
		if (p) {
			model = p.defaultModel;
			onboardingData.updateField('llm_model', p.defaultModel);
		}

		if (value === 'ollama') {
			baseUrl = 'http://localhost:11434/v1';
			onboardingData.updateField('llm_base_url', 'http://localhost:11434/v1');
			apiKey = '';
			onboardingData.updateField('llm_api_key', '');
		} else {
			baseUrl = '';
			onboardingData.updateField('llm_base_url', '');
		}
	}

	$effect(() => {
		onboardingData.updateField('llm_api_key', apiKey);
	});
	$effect(() => {
		onboardingData.updateField('llm_model', model);
	});
	$effect(() => {
		onboardingData.updateField('llm_base_url', baseUrl);
	});
</script>

<div class="step">
	<h2 class="step-title">LLM Provider</h2>
	<p class="step-description">
		Choose which AI model generates your content. OpenAI and Anthropic require an API
		key; Ollama runs locally for free.
	</p>

	<div class="fields">
		<div class="field">
			<label class="field-label" for="llm-provider">Provider <span class="required">*</span></label>
			<select
				id="llm-provider"
				class="field-select"
				value={provider}
				onchange={onProviderChange}
			>
				{#each providers as p}
					<option value={p.value}>{p.label}</option>
				{/each}
			</select>
		</div>

		{#if provider !== 'ollama'}
			<div class="field">
				<label class="field-label" for="llm-key">API Key <span class="required">*</span></label>
				<input
					id="llm-key"
					type="password"
					class="field-input"
					placeholder={provider === 'openai' ? 'sk-...' : 'sk-ant-...'}
					bind:value={apiKey}
				/>
			</div>
		{/if}

		<div class="field">
			<label class="field-label" for="llm-model">Model</label>
			<input
				id="llm-model"
				type="text"
				class="field-input"
				bind:value={model}
			/>
		</div>

		{#if provider === 'ollama'}
			<div class="field">
				<label class="field-label" for="llm-url">Base URL</label>
				<input
					id="llm-url"
					type="text"
					class="field-input"
					placeholder="http://localhost:11434/v1"
					bind:value={baseUrl}
				/>
			</div>
		{/if}

		<ConnectionTest
			label="Test Connection"
			ontest={() =>
				api.settings.testLlm({
					provider,
					api_key: apiKey || null,
					model,
					base_url: baseUrl || null,
				})
			}
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

	.field-input,
	.field-select {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		transition: border-color 0.15s;
	}

	.field-input:focus,
	.field-select:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.field-input::placeholder {
		color: var(--color-text-subtle);
	}
</style>
