<script lang="ts">
	interface Props {
		saved: boolean;
	}

	let { saved = $bindable() }: Props = $props();
</script>

<h2 class="step-title">Your Instance is Secured</h2>
<p class="step-description">
	This instance already has a passphrase set. Since passphrases are stored as one-way hashes, the
	original cannot be displayed. After setup, you'll need your passphrase to log in.
</p>

<div class="recovery-paths">
	<div class="recovery-path">
		<strong>Remember your passphrase?</strong>
		<span>Use it to log in after setup completes.</span>
	</div>
	<div class="recovery-path">
		<strong>Reset from the terminal</strong>
		<code class="recovery-code">tuitbot-server --reset-passphrase</code>
		<span class="recovery-alt">Or with cargo:</span>
		<code class="recovery-code">cargo run -p tuitbot-server -- --reset-passphrase</code>
	</div>
	<div class="recovery-path">
		<strong>Reset from the dashboard</strong>
		<span>Settings &rarr; LAN Access once logged in.</span>
	</div>
</div>

<div class="info-box">
	<span class="info-icon">&#8505;</span>
	<div class="info-content">
		After setup, you'll be redirected to the login page. Use your passphrase or reset it using
		the methods above.
	</div>
</div>

<label class="save-checkbox">
	<input type="checkbox" bind:checked={saved} />
	<span>I understand how to access my instance</span>
</label>

<style>
	.step-title {
		font-size: 20px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 16px;
	}

	.step-description {
		font-size: 14px;
		color: var(--color-text-muted);
		line-height: 1.6;
		margin: 0 0 16px;
	}

	.recovery-paths {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 16px;
	}

	.recovery-path {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 12px 14px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.recovery-path strong {
		color: var(--color-text);
		font-size: 13px;
	}

	.recovery-alt {
		margin-top: 4px;
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.recovery-code {
		display: block;
		background: var(--color-base);
		padding: 8px 12px;
		border-radius: 6px;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12px;
		color: var(--color-text);
		word-break: break-all;
	}

	.info-box {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 12px 14px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
		border-radius: 8px;
		margin-bottom: 16px;
	}

	.info-icon {
		font-size: 16px;
		color: var(--color-accent);
		line-height: 1.4;
	}

	.info-content {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.save-checkbox {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: var(--color-text);
		cursor: pointer;
		padding: 8px 0;
	}

	.save-checkbox input[type='checkbox'] {
		width: 18px;
		height: 18px;
		accent-color: var(--color-accent);
		cursor: pointer;
	}

	.save-checkbox input[type='checkbox']:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}
</style>
