<script lang="ts">
	import { ExternalLink, Copy, Check } from 'lucide-svelte';

	interface Props {
		callbackUrl: string;
		copied: boolean;
		onCopy: () => void;
	}

	const { callbackUrl, copied, onCopy }: Props = $props();
</script>

<div class="setup-guide">
	<p class="guide-heading">Quick setup (~2 minutes)</p>
	<ol class="guide-steps">
		<li>
			Go to the <a
				href="https://developer.x.com/en/portal/dashboard"
				target="_blank"
				rel="noopener noreferrer"
				class="link"
				>X Developer Portal <ExternalLink size={10} /></a
			> and create a Project &amp; App (or select an existing one)
		</li>
		<li>
			Under <strong>User authentication settings</strong>, enable OAuth 2.0
		</li>
		<li>
			Set App type to <strong>Native App</strong> and paste this as your Callback URL:
			<span class="callback-url">
				<code>{callbackUrl}</code>
				<button class="copy-btn" onclick={onCopy} title="Copy to clipboard">
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

<style>
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
		transition:
			color 0.15s,
			border-color 0.15s;
	}

	.copy-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
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
</style>
