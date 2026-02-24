<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { ExternalLink } from 'lucide-svelte';

	let clientId = $state($onboardingData.client_id);
	let clientSecret = $state($onboardingData.client_secret);

	$effect(() => {
		onboardingData.updateField('client_id', clientId);
	});

	$effect(() => {
		onboardingData.updateField('client_secret', clientSecret);
	});
</script>

<div class="step">
	<h2 class="step-title">X API Credentials</h2>
	<p class="step-description">
		Connect your X developer account. You'll need an OAuth 2.0 Client ID from the
		<a
			href="https://developer.x.com/en/portal/dashboard"
			target="_blank"
			rel="noopener noreferrer"
			class="link"
		>
			X Developer Portal <ExternalLink size={12} />
		</a>.
	</p>

	<div class="fields">
		<div class="field">
			<label class="field-label" for="client-id">Client ID <span class="required">*</span></label>
			<input
				id="client-id"
				type="text"
				class="field-input"
				placeholder="Your OAuth 2.0 Client ID"
				bind:value={clientId}
			/>
		</div>

		<div class="field">
			<label class="field-label" for="client-secret">Client Secret <span class="optional">(optional)</span></label>
			<input
				id="client-secret"
				type="password"
				class="field-input"
				placeholder="For confidential clients only"
				bind:value={clientSecret}
			/>
			<span class="field-hint">Only needed for confidential OAuth clients. Most users can skip this.</span>
		</div>
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

	.link {
		color: var(--color-accent);
		text-decoration: none;
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}

	.link:hover {
		text-decoration: underline;
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
