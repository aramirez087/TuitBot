<script lang="ts">
	import { AlertTriangle } from 'lucide-svelte';

	interface Props {
		message?: string;
		description?: string;
		onretry?: () => void;
	}

	let { message = 'Something went wrong', description = '', onretry }: Props = $props();
</script>

<div class="error-state">
	<div class="error-icon">
		<AlertTriangle size={32} />
	</div>
	<h3 class="error-title">{message}</h3>
	{#if description}
		<p class="error-description">{description}</p>
	{/if}
	{#if onretry}
		<button class="retry-btn" onclick={onretry}>Try Again</button>
	{/if}
</div>

<style>
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 48px 24px;
		text-align: center;
	}

	.error-icon {
		color: var(--color-danger);
		opacity: 0.7;
	}

	.error-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.error-description {
		font-size: 13px;
		color: var(--color-text-muted);
		max-width: 400px;
		line-height: 1.5;
		margin: 0;
	}

	.retry-btn {
		margin-top: 8px;
		padding: 8px 20px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.retry-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}
</style>
