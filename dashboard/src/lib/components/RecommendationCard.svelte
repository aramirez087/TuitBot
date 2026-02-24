<script lang="ts">
	import type { Recommendation } from '$lib/api';

	interface Props {
		recommendation: Recommendation;
	}

	let { recommendation }: Props = $props();

	const priorityClass = $derived(
		recommendation.priority === 'high'
			? 'priority-high'
			: recommendation.priority === 'medium'
				? 'priority-medium'
				: 'priority-low'
	);

	const categoryLabel = $derived(
		recommendation.category.charAt(0).toUpperCase() + recommendation.category.slice(1)
	);

	const actionLink = $derived(
		recommendation.category === 'promote' || recommendation.category === 'kill'
			? '/settings'
			: recommendation.category === 'experiment'
				? '/targets'
				: null
	);
</script>

<div class="rec-card">
	<div class="rec-header">
		<span class="priority-badge {priorityClass}">{recommendation.priority}</span>
		<span class="category-label">{categoryLabel}</span>
	</div>
	<h4 class="rec-title">{recommendation.title}</h4>
	<p class="rec-description">{recommendation.description}</p>
	{#if actionLink}
		<a href={actionLink} class="rec-action">Go to {actionLink === '/settings' ? 'Settings' : 'Targets'}</a>
	{/if}
</div>

<style>
	.rec-card {
		padding: 14px 16px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	.rec-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
	}

	.priority-badge {
		padding: 2px 8px;
		border-radius: 10px;
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.priority-high {
		background-color: #f8514920;
		color: var(--color-danger);
	}

	.priority-medium {
		background-color: #d2992220;
		color: #d29922;
	}

	.priority-low {
		background-color: #58a6ff20;
		color: var(--color-accent);
	}

	.category-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.rec-title {
		margin: 0 0 4px;
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.rec-description {
		margin: 0;
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.rec-action {
		display: inline-block;
		margin-top: 8px;
		font-size: 12px;
		font-weight: 500;
		color: var(--color-accent);
		text-decoration: none;
	}

	.rec-action:hover {
		text-decoration: underline;
	}
</style>
