<script lang="ts">
	import type { Snippet } from 'svelte';

	/* eslint-disable @typescript-eslint/no-explicit-any */
	interface Props {
		id: string;
		title: string;
		description: string;
		icon: any;
		children: Snippet;
	}
	/* eslint-enable @typescript-eslint/no-explicit-any */

	let { id, title, description, icon, children }: Props = $props();

	const Icon = $derived(icon);
</script>

<section {id} class="settings-section">
	<div class="section-header">
		<div class="section-icon">
			<Icon size={18} />
		</div>
		<div class="section-info">
			<h2>{title}</h2>
			<p>{description}</p>
		</div>
	</div>
	<div class="section-content">
		{@render children()}
	</div>
</section>

<style>
	.settings-section {
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		overflow: hidden;
	}

	.section-header {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 20px 24px 0;
	}

	.section-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		flex-shrink: 0;
	}

	.section-info {
		flex: 1;
		min-width: 0;
	}

	h2 {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 2px;
	}

	p {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
		line-height: 1.4;
	}

	.section-content {
		padding: 20px 24px 24px;
	}
</style>
