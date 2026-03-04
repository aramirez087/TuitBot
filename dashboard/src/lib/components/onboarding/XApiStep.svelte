<script lang="ts">
	import { onboardingData } from '$lib/stores/onboarding';
	import { deploymentMode } from '$lib/stores/runtime';
	import { ExternalLink, Copy, Check, CheckCircle2, XCircle } from 'lucide-svelte';

	const CALLBACK_URL = 'http://127.0.0.1:8080/callback';

	let clientId = $state($onboardingData.client_id);
	let clientSecret = $state($onboardingData.client_secret);
	let copied = $state(false);

	let selectedMode = $derived($onboardingData.provider_backend === 'scraper' ? 'scraper' : 'x_api');
	let isCloud = $derived($deploymentMode === 'cloud');

	$effect(() => {
		onboardingData.updateField('client_id', clientId);
	});

	$effect(() => {
		onboardingData.updateField('client_secret', clientSecret);
	});

	function setMode(mode: string) {
		onboardingData.updateField('provider_backend', mode === 'x_api' ? '' : mode);
	}

	function copyCallbackUrl() {
		navigator.clipboard.writeText(CALLBACK_URL);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="step">
	<h2 class="step-title">X Access</h2>
	<p class="step-description">
		Choose how Tuitbot connects to X.
	</p>

	{#if !isCloud}
		<div class="mode-selector">
			<button
				type="button"
				class="mode-card"
				class:selected={selectedMode === 'x_api'}
				onclick={() => setMode('x_api')}
			>
				<div class="mode-radio" class:checked={selectedMode === 'x_api'}></div>
				<div class="mode-info">
					<span class="mode-label">Official X API <span class="mode-badge">Recommended</span></span>
					<span class="mode-desc">Full features. Post, discover, and engage. Requires Client ID from the Developer Portal.</span>
				</div>
			</button>
			<button
				type="button"
				class="mode-card"
				class:selected={selectedMode === 'scraper'}
				onclick={() => setMode('scraper')}
			>
				<div class="mode-radio" class:checked={selectedMode === 'scraper'}></div>
				<div class="mode-info">
					<span class="mode-label">Local No-Key Mode</span>
					<span class="mode-desc">Get started instantly. No API credentials needed. Discovery and drafting. Read-only by default.</span>
				</div>
			</button>
		</div>
	{/if}

	{#if selectedMode === 'x_api' || isCloud}
		<div class="setup-guide">
			<p class="guide-heading">Quick setup (~2 minutes)</p>
			<ol class="guide-steps">
				<li>Go to the <a href="https://developer.x.com/en/portal/dashboard" target="_blank" rel="noopener noreferrer" class="link">X Developer Portal <ExternalLink size={10} /></a> and create a Project &amp; App (or select an existing one)</li>
				<li>Under <strong>User authentication settings</strong>, enable OAuth 2.0</li>
				<li>
					Set App type to <strong>Native App</strong> and paste this as your Callback URL:
					<span class="callback-url">
						<code>{CALLBACK_URL}</code>
						<button class="copy-btn" onclick={copyCallbackUrl} title="Copy to clipboard">
							{#if copied}
								<Check size={13} />
							{:else}
								<Copy size={13} />
							{/if}
						</button>
					</span>
				</li>
				<li>Copy the <strong>Client ID</strong> from the "Keys and tokens" tab</li>
			</ol>
		</div>

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
	{:else}
		<div class="feature-matrix">
			<p class="matrix-heading">What you can do</p>
			<ul class="matrix-list">
				<li class="available"><CheckCircle2 size={14} /> Search and discover tweets</li>
				<li class="available"><CheckCircle2 size={14} /> Score conversations for relevance</li>
				<li class="available"><CheckCircle2 size={14} /> Draft replies and original content</li>
				<li class="available"><CheckCircle2 size={14} /> Plan and preview threads</li>
				<li class="unavailable"><XCircle size={14} /> Post tweets and replies</li>
				<li class="unavailable"><XCircle size={14} /> Mentions and home timeline</li>
			</ul>
			<p class="matrix-footer">
				You can switch to the Official X API anytime in Settings.
			</p>
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

	.mode-selector {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.mode-card {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 14px 16px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		cursor: pointer;
		text-align: left;
		transition: border-color 0.15s, background 0.15s;
	}

	.mode-card:hover {
		border-color: var(--color-border-hover, var(--color-text-muted));
	}

	.mode-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, var(--color-base));
	}

	.mode-radio {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 2px solid var(--color-border);
		flex-shrink: 0;
		margin-top: 2px;
		transition: border-color 0.15s;
	}

	.mode-radio.checked {
		border-color: var(--color-accent);
		background: var(--color-accent);
		box-shadow: inset 0 0 0 3px var(--color-base);
	}

	.mode-info {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.mode-label {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
	}

	.mode-badge {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		padding: 1px 6px;
		border-radius: 3px;
		margin-left: 4px;
		vertical-align: middle;
	}

	.mode-desc {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.4;
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

	.setup-guide {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 14px 16px;
	}

	.guide-heading {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin: 0 0 10px;
	}

	.guide-steps {
		margin: 0;
		padding-left: 20px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.guide-steps strong {
		color: var(--color-text);
	}

	.callback-url {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin-top: 4px;
	}

	.callback-url code {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		padding: 3px 8px;
		font-size: 12px;
		color: var(--color-accent);
		font-family: monospace;
		user-select: all;
	}

	.copy-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 3px;
		background: transparent;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: color 0.15s, border-color 0.15s;
	}

	.copy-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	.feature-matrix {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 14px 16px;
	}

	.matrix-heading {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin: 0 0 10px;
	}

	.matrix-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.matrix-list li {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		line-height: 1.4;
	}

	.matrix-list li.available {
		color: var(--color-success, #22c55e);
	}

	.matrix-list li.unavailable {
		color: var(--color-text-subtle);
	}

	.matrix-footer {
		margin: 12px 0 0;
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
	}
</style>
