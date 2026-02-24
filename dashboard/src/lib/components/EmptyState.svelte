<script lang="ts">
	import { Inbox } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		description?: string;
		actionLabel?: string;
		onaction?: () => void;
		icon?: Snippet;
	}

	let { title, description = '', actionLabel = '', onaction, icon }: Props = $props();
</script>

<div class="empty-state">
	<div class="empty-icon">
		{#if icon}
			{@render icon()}
		{:else}
			<Inbox size={40} strokeWidth={1.2} />
		{/if}
	</div>
	<h3 class="empty-title">{title}</h3>
	{#if description}
		<p class="empty-description">{description}</p>
	{/if}
	{#if actionLabel && onaction}
		<button class="action-btn" onclick={onaction}>{actionLabel}</button>
	{/if}
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 64px 24px;
		text-align: center;
	}

	.empty-icon {
		color: var(--color-text-subtle);
		opacity: 0.5;
	}

	.empty-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.empty-description {
		font-size: 13px;
		color: var(--color-text-muted);
		max-width: 400px;
		line-height: 1.5;
		margin: 0;
	}

	.action-btn {
		margin-top: 8px;
		padding: 8px 20px;
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		color: white;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: filter 0.15s;
	}

	.action-btn:hover {
		filter: brightness(1.1);
	}
</style>
