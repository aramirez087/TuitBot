<script lang="ts">
	import type { ContentActivity } from '$lib/api/types';
	import {
		RotateCcw,
		Sparkles,
		CalendarClock,
		CalendarX2,
		Plus,
		Pencil,
		Archive,
		ArchiveRestore,
		Send,
		Clock,
	} from 'lucide-svelte';
	import { formatInAccountTz } from '$lib/utils/timezone';

	interface Props {
		activity: ContentActivity[];
		timezone: string;
	}

	const { activity, timezone }: Props = $props();

	const actionMeta: Record<string, { label: string; icon: typeof Plus }> = {
		created: { label: 'Created', icon: Plus },
		edited: { label: 'Edited', icon: Pencil },
		ai_rewrite: { label: 'AI Rewrite', icon: Sparkles },
		scheduled: { label: 'Scheduled', icon: CalendarClock },
		rescheduled: { label: 'Rescheduled', icon: CalendarClock },
		unscheduled: { label: 'Unscheduled', icon: CalendarX2 },
		archived: { label: 'Archived', icon: Archive },
		restored: { label: 'Restored from Archive', icon: ArchiveRestore },
		revision_restored: { label: 'Restored to Revision', icon: RotateCcw },
		posted: { label: 'Published', icon: Send },
	};

	function getAction(action: string) {
		return actionMeta[action] ?? { label: action, icon: Clock };
	}

	function relativeTime(iso: string): string {
		const now = Date.now();
		const then = new Date(iso).getTime();
		const diff = now - then;
		const mins = Math.floor(diff / 60000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		if (days < 7) return `${days}d ago`;
		return new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}

	function parseDetail(detail: string | null): Record<string, unknown> | null {
		if (!detail) return null;
		try {
			return JSON.parse(detail);
		} catch {
			return null;
		}
	}

	const tzFmt = { month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit', timeZoneName: 'short' } as const;
</script>

{#if activity.length === 0}
	<div class="empty-state">
		<Clock size={20} />
		<p>No activity yet</p>
	</div>
{:else}
	<div class="timeline">
		{#each activity as act (act.id)}
			{@const meta = getAction(act.action)}
			{@const ActionIcon = meta.icon}
			{@const detail = parseDetail(act.detail)}
			<div class="activity-item" class:ai={act.action === 'ai_rewrite'}>
				<div class="act-icon" class:ai-icon={act.action === 'ai_rewrite'}>
					<ActionIcon size={12} />
				</div>
				<div class="act-body">
					<div class="act-header">
						<span class="act-label">{meta.label}</span>
						<span class="act-time">{relativeTime(act.created_at)}</span>
					</div>
					{#if detail}
						{#if detail.scheduled_for}
							<p class="act-detail">
								{formatInAccountTz(detail.scheduled_for as string, timezone, tzFmt)}
							</p>
						{/if}
						{#if detail.from && detail.to}
							<p class="act-detail">
								{formatInAccountTz(detail.from as string, timezone, tzFmt)} →
								{formatInAccountTz(detail.to as string, timezone, tzFmt)}
							</p>
						{/if}
						{#if detail.source === 'duplicate'}
							<p class="act-detail">Duplicated from #{detail.original_id}</p>
						{/if}
						{#if detail.from_revision_id}
							<p class="act-detail">From revision #{detail.from_revision_id}</p>
						{/if}
					{/if}
				</div>
			</div>
		{/each}
	</div>
{/if}

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px 20px;
		color: var(--color-text-subtle);
		text-align: center;
	}

	.empty-state p {
		margin: 0;
		font-size: 13px;
	}

	.timeline {
		display: flex;
		flex-direction: column;
	}

	.activity-item {
		display: flex;
		gap: 10px;
		padding: 8px 14px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
	}

	.activity-item.ai {
		border-left: 2px solid color-mix(in srgb, #a855f7 60%, transparent);
	}

	.act-icon {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-border-subtle) 60%, transparent);
		color: var(--color-text-subtle);
		margin-top: 1px;
	}

	.ai-icon {
		background: color-mix(in srgb, #a855f7 15%, transparent);
		color: #a855f7;
	}

	.act-body {
		flex: 1;
		min-width: 0;
	}

	.act-header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 6px;
	}

	.act-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
	}

	.act-time {
		font-size: 10px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.act-detail {
		margin: 3px 0 0;
		font-size: 11px;
		color: var(--color-text-subtle);
		opacity: 0.8;
	}
</style>
