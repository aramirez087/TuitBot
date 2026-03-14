<script lang="ts">
	import { Clock } from 'lucide-svelte';
	import type { ApprovalItem } from '$lib/api';

	interface Props {
		item: ApprovalItem;
		statusClass: string;
		typeLabel: string;
		scheduledLabel: string | null;
	}

	const { item, statusClass, typeLabel, scheduledLabel }: Props = $props();

	function relativeTime(iso: string): string {
		const diff = Date.now() - new Date(iso).getTime();
		const mins = Math.floor(diff / 60_000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ago`;
	}
</script>

<div class="card-header">
	<span class="card-type">{typeLabel}</span>
	<span class="card-badge {statusClass}">{item.status}</span>
	{#if item.score > 0}
		<span class="card-score">{Math.round(item.score)} pts</span>
	{/if}
	<span class="card-time">{relativeTime(item.created_at)}</span>
</div>

{#if item.action_type === 'reply' && item.target_author}
	<div class="card-context">
		<span class="context-label">Replying to</span>
		<span class="context-author">@{item.target_author.replace(/^@/, '')}</span>
	</div>
{/if}

{#if scheduledLabel}
	<div class="card-schedule">
		<Clock size={12} />
		<span>Scheduled for {scheduledLabel}</span>
	</div>
{/if}

<style>
	.card-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
	}

	.card-type {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		text-transform: capitalize;
	}

	.card-badge {
		font-size: 11px;
		font-weight: 600;
		padding: 1px 6px;
		border-radius: 4px;
	}

	.card-badge.status-pending {
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		color: var(--color-warning);
	}

	.card-badge.status-approved {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.card-badge.status-rejected {
		background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.card-badge.status-scheduled {
		background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
	}

	.card-score {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-accent);
		font-variant-numeric: tabular-nums;
	}

	.card-time {
		margin-left: auto;
		font-size: 11px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.card-context {
		margin-bottom: 8px;
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.context-label {
		margin-right: 4px;
	}

	.context-author {
		color: var(--color-accent);
		font-weight: 600;
	}

	.card-schedule {
		display: flex;
		align-items: center;
		gap: 5px;
		margin-bottom: 8px;
		padding: 4px 10px;
		border-radius: 5px;
		background-color: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
		font-size: 12px;
		font-weight: 500;
		width: fit-content;
	}
</style>
