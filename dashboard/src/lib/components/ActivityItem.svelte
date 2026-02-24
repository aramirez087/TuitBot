<script lang="ts">
	import {
		Search,
		MessageSquare,
		FileText,
		BookOpen,
		Bell,
		Trash2,
		RefreshCw,
		BarChart3,
		UserPlus
	} from 'lucide-svelte';
	import type { ActionLogEntry } from '$lib/api';
	import ScoreBreakdown from './ScoreBreakdown.svelte';

	interface Props {
		action: ActionLogEntry;
	}

	let { action }: Props = $props();

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

	/* eslint-disable @typescript-eslint/no-explicit-any */
	const iconMap: Record<string, any> = {
		search: Search,
		reply: MessageSquare,
		discovery_reply: MessageSquare,
		mention_reply: MessageSquare,
		target_reply: MessageSquare,
		tweet: FileText,
		thread: BookOpen,
		mention_check: Bell,
		cleanup: Trash2,
		auth_refresh: RefreshCw,
		analytics: BarChart3,
		target_follow: UserPlus
	};
	/* eslint-enable @typescript-eslint/no-explicit-any */

	const Icon = $derived(iconMap[action.action_type] ?? Search);

	const statusClass = $derived(
		action.status === 'success'
			? 'status-success'
			: action.status === 'failure'
				? 'status-error'
				: 'status-skipped'
	);

	const typeLabel = $derived(action.action_type.replace(/_/g, ' '));
</script>

<div class="activity-item">
	<div class="item-dot {statusClass}">
		<Icon size={14} />
	</div>
	<div class="item-body">
		<div class="item-header">
			<span class="item-type">{typeLabel}</span>
			<span class="item-badge {statusClass}">{action.status}</span>
			<span class="item-time">{relativeTime(action.created_at)}</span>
		</div>
		{#if action.message}
			<p class="item-message">{action.message}</p>
		{/if}
		<ScoreBreakdown metadata={action.metadata} />
	</div>
</div>

<style>
	.activity-item {
		display: flex;
		gap: 12px;
		padding: 12px 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		transition: background-color 0.1s ease;
	}

	.activity-item:hover {
		background-color: var(--color-surface-hover);
	}

	.activity-item:last-child {
		border-bottom: none;
	}

	.item-dot {
		width: 30px;
		height: 30px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 2px;
	}

	.item-dot.status-success {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.item-dot.status-error {
		background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.item-dot.status-skipped {
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		color: var(--color-warning);
	}

	.item-body {
		flex: 1;
		min-width: 0;
	}

	.item-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.item-type {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		text-transform: capitalize;
	}

	.item-badge {
		font-size: 11px;
		font-weight: 600;
		padding: 1px 6px;
		border-radius: 4px;
	}

	.item-badge.status-success {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.item-badge.status-error {
		background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.item-badge.status-skipped {
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		color: var(--color-warning);
	}

	.item-time {
		margin-left: auto;
		font-size: 11px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.item-message {
		margin: 4px 0 0;
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.4;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
