<script lang="ts">
	import { PenLine } from 'lucide-svelte';

	let { variant, oncreate }: {
		variant: 'no-drafts' | 'no-selection';
		oncreate: () => void;
	} = $props();

	const heading = $derived(variant === 'no-drafts' ? 'Start writing' : 'Select a draft');
	const description = $derived(
		variant === 'no-drafts'
			? 'Create your first draft to begin composing.'
			: 'Choose a draft from the rail or create a new one.'
	);
</script>

<div class="empty-state">
	<div class="empty-icon">
		<PenLine size={32} strokeWidth={1.5} />
	</div>
	<h2 class="empty-heading">{heading}</h2>
	<p class="empty-description">{description}</p>
	<button class="create-btn" type="button" onclick={oncreate}>New Draft</button>
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: 40px 24px;
		text-align: center;
	}

	.empty-icon {
		color: var(--color-text-subtle);
		margin-bottom: 16px;
		opacity: 0.6;
	}

	.empty-heading {
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 6px;
	}

	.empty-description {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0 0 20px;
		max-width: 260px;
		line-height: 1.5;
	}

	.create-btn {
		padding: 8px 20px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.create-btn:hover {
		background: var(--color-accent-hover);
	}
</style>
