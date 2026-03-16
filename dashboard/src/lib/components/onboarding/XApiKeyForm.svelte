<script lang="ts">
	interface Props {
		clientId: string;
		clientSecret: string;
	}

	let { clientId = $bindable(''), clientSecret = $bindable('') }: Props = $props();
</script>

<div class="fields">
	<div class="field">
		<label class="field-label" for="client-id"
			>Client ID <span class="required">*</span></label
		>
		<input
			id="client-id"
			type="text"
			class="field-input"
			placeholder="Your OAuth 2.0 Client ID"
			bind:value={clientId}
		/>
	</div>

	<div class="field">
		<label class="field-label" for="client-secret"
			>Client Secret <span class="optional">(optional)</span></label
		>
		<input
			id="client-secret"
			type="password"
			class="field-input"
			placeholder="For confidential clients only"
			bind:value={clientSecret}
		/>
		<span class="field-hint"
			>Only needed for confidential OAuth clients. Most users can skip this.</span
		>
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
