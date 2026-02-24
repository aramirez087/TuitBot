<script lang="ts">
	import type { StrategyReport, StrategyInputs } from '$lib/api';
	import { ArrowRight, RotateCcw } from 'lucide-svelte';

	interface Props {
		report: StrategyReport | null;
		inputs: StrategyInputs | null;
	}

	let { report, inputs }: Props = $props();

	const pillarsCount = $derived(inputs?.content_pillars.length ?? 0);
	const keywordsCount = $derived(
		(inputs?.product_keywords.length ?? 0) + (inputs?.competitor_keywords.length ?? 0)
	);
	const targetsCount = $derived(inputs?.target_accounts.length ?? 0);
	const totalOutput = $derived(
		(report?.replies_sent ?? 0) + (report?.tweets_posted ?? 0) + (report?.threads_posted ?? 0)
	);
</script>

<div class="loop-container">
	<div class="loop-header">
		<h3>Growth Loop</h3>
		<span class="loop-label">
			<RotateCcw size={12} />
			Weekly Iteration
		</span>
	</div>

	<div class="loop-flow">
		<div class="loop-node">
			<span class="node-label">Inputs</span>
			<span class="node-value">
				{pillarsCount} pillars, {keywordsCount} keywords, {targetsCount} targets
			</span>
			<a href="/settings" class="node-link">Settings</a>
		</div>

		<div class="loop-arrow"><ArrowRight size={16} /></div>

		<div class="loop-node">
			<span class="node-label">Engine</span>
			<span class="node-value">
				{inputs?.industry_topics.length ?? 0} topics
			</span>
		</div>

		<div class="loop-arrow"><ArrowRight size={16} /></div>

		<div class="loop-node">
			<span class="node-label">Outputs</span>
			<span class="node-value">
				{report?.replies_sent ?? 0} replies, {report?.tweets_posted ?? 0} tweets, {report?.threads_posted ?? 0} threads
			</span>
		</div>

		<div class="loop-arrow"><ArrowRight size={16} /></div>

		<div class="loop-node highlight">
			<span class="node-label">Metrics</span>
			<span class="node-value">
				{report?.follower_delta ?? 0 > 0 ? '+' : ''}{report?.follower_delta ?? 0} followers,
				{((report?.reply_acceptance_rate ?? 0) * 100).toFixed(0)}% acceptance
			</span>
		</div>
	</div>
</div>

<style>
	.loop-container {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 20px;
	}

	.loop-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
	}

	h3 {
		margin: 0;
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
	}

	.loop-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
	}

	.loop-flow {
		display: flex;
		align-items: center;
		gap: 8px;
		overflow-x: auto;
	}

	.loop-node {
		flex: 1;
		min-width: 140px;
		padding: 12px;
		background-color: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.loop-node.highlight {
		border-color: var(--color-accent);
		background-color: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface-active));
	}

	.node-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
	}

	.node-value {
		font-size: 12px;
		color: var(--color-text);
		line-height: 1.4;
	}

	.node-link {
		font-size: 11px;
		color: var(--color-accent);
		text-decoration: none;
	}

	.node-link:hover {
		text-decoration: underline;
	}

	.loop-arrow {
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	@media (max-width: 700px) {
		.loop-flow {
			flex-direction: column;
		}

		.loop-arrow {
			transform: rotate(90deg);
		}

		.loop-node {
			min-width: unset;
			width: 100%;
		}
	}
</style>
