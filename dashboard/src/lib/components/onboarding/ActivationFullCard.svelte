<script lang="ts">
	import {
		CheckCircle2,
		Circle,
		ChevronRight,
		Zap,
		X,
		Sparkles,
		ArrowRight,
	} from 'lucide-svelte';
	import { trackFunnel } from '$lib/analytics/funnel';

	interface ChecklistItem {
		id: string;
		href: string;
		label: string;
		description: string;
		completed: boolean;
		optional: boolean;
	}

	interface TierAction {
		label: string;
		description: string;
	}

	interface Props {
		items: ChecklistItem[];
		actions: TierAction[];
		completedCount: number;
		requiredCount: number;
		progressPercent: number;
		color: string;
		label: string;
		onDismiss: () => void;
	}

	const {
		items,
		actions,
		completedCount,
		requiredCount,
		progressPercent,
		color,
		label,
		onDismiss,
	}: Props = $props();

	function trackItemClick(itemId: string, completed: boolean) {
		trackFunnel('activation:checklist-item-clicked', { item_id: itemId, completed });
	}
</script>

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
		<button class="dismiss-btn" onclick={onDismiss} aria-label="Dismiss checklist">
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

				{#each items.filter((i) => i.optional) as item (item.id)}
					<a href={item.href} class="checklist-item optional">
						<div class="item-check optional-check">
							<Sparkles size={16} />
						</div>
						<div class="item-content">
							<span class="item-label"
								>{item.label} <span class="optional-badge">optional</span></span
							>
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

<style>
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

	@media (prefers-reduced-motion: reduce) {
		.progress-bar,
		.checklist-item,
		.dismiss-btn {
			transition: none;
		}
	}
</style>
