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
	import {
		CheckCircle2,
		Circle,
		ChevronRight,
		Zap,
		X,
		Sparkles,
		ArrowRight,
	} from 'lucide-svelte';

	interface Props {
		compact?: boolean;
	}

	let { compact = false }: Props = $props();

	let dismissed = $state(false);
	let previousTier = $state('');
	let viewTracked = $state(false);

	// Reset dismissal when tier changes; track tier transitions.
	$effect(() => {
		const current = $capabilityTier;
		if (previousTier && previousTier !== current) {
			dismissed = false;
			trackFunnel('activation:tier-changed', { from: previousTier, to: current });
		}
		previousTier = current;
	});

	// Track checklist view (once per mount, full mode only).
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

	function trackItemClick(itemId: string, completed: boolean) {
		trackFunnel('activation:checklist-item-clicked', { item_id: itemId, completed });
	}
</script>

{#if $capabilitiesLoaded && !isFullyActivated && !dismissed}
	{#if compact}
		<!-- Compact banner for secondary pages -->
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
		<!-- Full checklist card for home page -->
		<div class="checklist-card">
			<div class="checklist-header">
				<div class="header-left">
					<div class="tier-badge" style="border-color: {color}; color: {color}">
						<Zap size={13} />
						{label}
					</div>
					<div class="progress-bar-wrap">
						<div class="progress-bar" style="width: {progressPercent}%; background: {color}"></div>
					</div>
					<span class="progress-text">{completedCount} of {requiredCount}</span>
				</div>
				<button class="dismiss-btn" onclick={dismiss} aria-label="Dismiss checklist">
					<X size={16} />
				</button>
			</div>

			<div class="checklist-body">
				<!-- Next steps column -->
				<div class="next-steps">
					<h3 class="section-title">Next steps</h3>
					<div class="items-list">
						{#each items.filter((i) => !i.optional) as item (item.id)}
							<a
								href={item.href}
								class="checklist-item"
								class:completed={item.completed}
								onclick={() => trackItemClick(item.id, item.completed)}
							>
								<div class="item-check">
									{#if item.completed}
										<CheckCircle2 size={18} />
									{:else}
										<Circle size={18} />
									{/if}
								</div>
								<div class="item-content">
									<span class="item-label">{item.label}</span>
									<span class="item-desc">{item.description}</span>
								</div>
								{#if !item.completed}
									<ChevronRight size={16} class="item-arrow" />
								{/if}
							</a>
						{/each}

						<!-- Optional vault item -->
						{#each items.filter((i) => i.optional) as item (item.id)}
							<a
								href={item.href}
								class="checklist-item optional"
							>
								<div class="item-check optional-check">
									<Sparkles size={16} />
								</div>
								<div class="item-content">
									<span class="item-label">{item.label} <span class="optional-badge">optional</span></span>
									<span class="item-desc">{item.description}</span>
								</div>
								<ChevronRight size={16} class="item-arrow" />
							</a>
						{/each}
					</div>
				</div>

				<!-- Available now column -->
				<div class="available-now">
					<h3 class="section-title">Available now</h3>
					<div class="actions-list">
						{#each actions as action}
							<div class="action-item">
								<ArrowRight size={14} />
								<div class="action-content">
									<span class="action-label">{action.label}</span>
									<span class="action-desc">{action.description}</span>
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>
	{/if}
{/if}

<style>
	/* Compact banner */
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

	/* Full checklist card */
	.checklist-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 12px;
		margin-bottom: 24px;
		overflow: hidden;
	}

	.checklist-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.tier-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border: 1px solid;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 600;
		white-space: nowrap;
	}

	.progress-bar-wrap {
		width: 80px;
		height: 4px;
		background: var(--color-border);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		border-radius: 2px;
		transition: width 0.4s ease;
	}

	.progress-text {
		font-size: 12px;
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
	}

	.dismiss-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s;
	}

	.dismiss-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text-muted);
	}

	.checklist-body {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0;
	}

	@media (max-width: 720px) {
		.checklist-body {
			grid-template-columns: 1fr;
		}
	}

	/* Sections */
	.next-steps {
		padding: 16px 20px 20px;
		border-right: 1px solid var(--color-border-subtle);
	}

	@media (max-width: 720px) {
		.next-steps {
			border-right: none;
			border-bottom: 1px solid var(--color-border-subtle);
		}
	}

	.available-now {
		padding: 16px 20px 20px;
	}

	.section-title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		margin: 0 0 10px 0;
	}

	/* Checklist items */
	.items-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.checklist-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-radius: 8px;
		text-decoration: none;
		color: var(--color-text);
		transition: background 0.15s;
	}

	.checklist-item:hover {
		background: var(--color-surface-hover);
	}

	.checklist-item.completed {
		opacity: 0.5;
		pointer-events: none;
	}

	.item-check {
		flex-shrink: 0;
		color: var(--color-text-subtle);
	}

	.checklist-item.completed .item-check {
		color: var(--color-success);
	}

	.optional-check {
		color: var(--color-warning);
	}

	.item-content {
		display: flex;
		flex-direction: column;
		gap: 1px;
		flex: 1;
		min-width: 0;
	}

	.item-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.checklist-item.completed .item-label {
		text-decoration: line-through;
		color: var(--color-text-muted);
	}

	.item-desc {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.checklist-item :global(.item-arrow) {
		flex-shrink: 0;
		color: var(--color-text-subtle);
	}

	.optional-badge {
		font-size: 10px;
		font-weight: 500;
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		padding: 1px 6px;
		border-radius: 8px;
		margin-left: 4px;
		vertical-align: middle;
	}

	/* Available actions */
	.actions-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.action-item {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 6px 0;
		color: var(--color-accent);
	}

	.action-item :global(svg) {
		flex-shrink: 0;
		margin-top: 2px;
	}

	.action-content {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.action-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.action-desc {
		font-size: 11px;
		color: var(--color-text-subtle);
	}
</style>
