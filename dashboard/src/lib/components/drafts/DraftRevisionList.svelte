<script lang="ts">
	import type { ContentRevision } from '$lib/api/types';
	import {
		RotateCcw,
		Sparkles,
		CalendarClock,
		CalendarX2,
		Plus,
		Save,
		Clock,
	} from 'lucide-svelte';

	interface Props {
		revisions: ContentRevision[];
		onrestore: (revisionId: number) => void;
	}

	const { revisions, onrestore }: Props = $props();

	let confirmingId = $state<number | null>(null);

	const triggerMeta: Record<string, { label: string; icon: typeof Save }> = {
		manual: { label: 'Manual Checkpoint', icon: Save },
		schedule: { label: 'Before Scheduling', icon: CalendarClock },
		unschedule: { label: 'Before Unscheduling', icon: CalendarX2 },
		reschedule: { label: 'Before Reschedule', icon: CalendarClock },
		ai_rewrite: { label: 'AI Rewrite', icon: Sparkles },
		pre_restore: { label: 'Before Restore', icon: RotateCcw },
		created: { label: 'Initial Version', icon: Plus },
	};

	function getTrigger(kind: string) {
		return triggerMeta[kind] ?? { label: kind, icon: Clock };
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

	function contentPreview(content: string): string {
		const trimmed = content.trim();
		if (trimmed.length <= 80) return trimmed;
		return trimmed.slice(0, 77) + '...';
	}

	function handleRestore(revId: number) {
		if (confirmingId === revId) {
			onrestore(revId);
			confirmingId = null;
		} else {
			confirmingId = revId;
		}
	}
</script>

{#if revisions.length === 0}
	<div class="empty-state">
		<Clock size={20} />
		<p>No revisions yet</p>
		<p class="empty-hint">
			Revisions are created when you schedule, unschedule, or use AI rewrites.
		</p>
	</div>
{:else}
	<div class="timeline">
		{#each revisions as rev (rev.id)}
			{@const trigger = getTrigger(rev.trigger_kind)}
			{@const TriggerIcon = trigger.icon}
			<div class="revision-item" class:ai={rev.trigger_kind === 'ai_rewrite'}>
				<div class="rev-icon" class:ai-icon={rev.trigger_kind === 'ai_rewrite'}>
					<TriggerIcon size={12} />
				</div>
				<div class="rev-body">
					<div class="rev-header">
						<span class="rev-label">{trigger.label}</span>
						<span class="rev-time">{relativeTime(rev.created_at)}</span>
					</div>
					<p class="rev-preview">{contentPreview(rev.content)}</p>
					<div class="rev-actions">
						{#if confirmingId === rev.id}
							<span class="confirm-msg">Current state will be saved first.</span>
							<button
								type="button"
								class="restore-btn confirm"
								onclick={() => handleRestore(rev.id)}
							>
								Confirm
							</button>
							<button
								type="button"
								class="cancel-btn"
								onclick={() => (confirmingId = null)}
							>
								Cancel
							</button>
						{:else}
							<button
								type="button"
								class="restore-btn"
								onclick={() => handleRestore(rev.id)}
							>
								<RotateCcw size={11} />
								Restore
							</button>
						{/if}
					</div>
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

	.empty-hint {
		font-size: 11px !important;
		opacity: 0.7;
		max-width: 200px;
		line-height: 1.4;
	}

	.timeline {
		display: flex;
		flex-direction: column;
	}

	.revision-item {
		display: flex;
		gap: 10px;
		padding: 10px 14px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
		transition: background 0.1s;
	}

	.revision-item:hover {
		background: color-mix(in srgb, var(--color-border-subtle) 30%, transparent);
	}

	.revision-item.ai {
		border-left: 2px solid color-mix(in srgb, #a855f7 60%, transparent);
	}

	.rev-icon {
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

	.rev-body {
		flex: 1;
		min-width: 0;
	}

	.rev-header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 6px;
	}

	.rev-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
	}

	.rev-time {
		font-size: 10px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.rev-preview {
		margin: 4px 0 6px;
		font-size: 11px;
		color: var(--color-text-subtle);
		line-height: 1.4;
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.rev-actions {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}

	.restore-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		transition: all 0.12s;
	}

	.restore-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.restore-btn.confirm {
		border-color: var(--color-accent);
		background: var(--color-accent);
		color: white;
	}

	.cancel-btn {
		padding: 3px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
	}

	.cancel-btn:hover {
		color: var(--color-text);
	}

	.confirm-msg {
		font-size: 10px;
		color: var(--color-text-subtle);
		width: 100%;
		margin-bottom: 2px;
	}

	@media (prefers-reduced-motion: reduce) {
		.revision-item {
			transition: none;
		}
	}
</style>
