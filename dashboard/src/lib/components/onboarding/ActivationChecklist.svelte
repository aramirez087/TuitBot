<script lang="ts">
	import {
		capabilityTier,
		tierRank,
		tierLabel,
		tierColor,
		computeChecklistItems,
		currentTierActions,
	} from '$lib/stores/capability';
	import { capabilitiesLoaded } from '$lib/stores/runtime';
	import { trackFunnel } from '$lib/analytics/funnel';
	import { ChevronRight, X } from 'lucide-svelte';
	import ActivationFullCard from './ActivationFullCard.svelte';

	interface Props {
		compact?: boolean;
	}

	let { compact = false }: Props = $props();

	let dismissed = $state(false);
	let previousTier = $state('');
	let viewTracked = $state(false);

	$effect(() => {
		const current = $capabilityTier;
		if (previousTier && previousTier !== current) {
			dismissed = false;
			trackFunnel('activation:tier-changed', { from: previousTier, to: current });
		}
		previousTier = current;
	});

	$effect(() => {
		if (!compact && $capabilitiesLoaded && !isFullyActivated && !viewTracked) {
			viewTracked = true;
			trackFunnel('activation:checklist-viewed', {
				tier: $capabilityTier,
				completed: completedCount,
				total: requiredCount,
			});
		}
	});

	let items = $derived(computeChecklistItems($capabilityTier));
	let nextSteps = $derived(items.filter((i) => !i.completed && !i.optional));
	let completedCount = $derived(items.filter((i) => i.completed).length);
	let requiredCount = $derived(items.filter((i) => !i.optional).length);
	let nextAction = $derived(nextSteps[0] ?? null);
	let actions = $derived(currentTierActions($capabilityTier));
	let rank = $derived(tierRank($capabilityTier));
	let label = $derived(tierLabel($capabilityTier));
	let color = $derived(tierColor($capabilityTier));
	let isFullyActivated = $derived($capabilityTier === 'posting_ready');
	let progressPercent = $derived(Math.round((completedCount / requiredCount) * 100));

	function dismiss() {
		trackFunnel('activation:checklist-dismissed', { tier: $capabilityTier });
		dismissed = true;
	}
</script>

{#if $capabilitiesLoaded && !isFullyActivated && !dismissed}
	{#if compact}
		<div class="compact-banner">
			<div class="compact-left">
				<div class="tier-dot" style="background: {color}"></div>
				<span class="compact-tier">{label}</span>
				<span class="compact-sep">&middot;</span>
				<span class="compact-progress">{completedCount}/{requiredCount} steps done</span>
			</div>
			{#if nextAction}
				<a href={nextAction.href} class="compact-action">
					{nextAction.label}
					<ChevronRight size={14} />
				</a>
			{/if}
			<button class="compact-dismiss" onclick={dismiss} aria-label="Dismiss">
				<X size={14} />
			</button>
		</div>
	{:else}
		<ActivationFullCard
			{items}
			{actions}
			{completedCount}
			{requiredCount}
			{progressPercent}
			{color}
			{label}
			onDismiss={dismiss}
		/>
	{/if}
{/if}

<style>
	.compact-banner {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		margin-bottom: 16px;
	}

	.compact-left {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: 1;
	}

	.tier-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.compact-tier {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
	}

	.compact-sep {
		color: var(--color-text-subtle);
	}

	.compact-progress {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.compact-action {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		font-weight: 500;
		color: var(--color-accent);
		text-decoration: none;
		white-space: nowrap;
		transition: color 0.15s;
	}

	.compact-action:hover {
		color: var(--color-accent-hover);
	}

	.compact-dismiss {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s;
		flex-shrink: 0;
	}

	.compact-dismiss:hover {
		background: var(--color-surface-hover);
		color: var(--color-text-muted);
	}
</style>
