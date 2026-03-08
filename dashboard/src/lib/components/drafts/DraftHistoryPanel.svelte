<script lang="ts">
	import type { ContentRevision, ContentActivity } from '$lib/api/types';
	import {
		RotateCcw,
		Sparkles,
		CalendarClock,
		CalendarX2,
		Plus,
		Save,
		Pencil,
		Archive,
		ArchiveRestore,
		Send,
		Clock,
		X
	} from 'lucide-svelte';

	let {
		revisions = [],
		activity = [],
		onrestore,
		onclose
	}: {
		revisions: ContentRevision[];
		activity: ContentActivity[];
		onrestore: (revisionId: number) => void;
		onclose: () => void;
	} = $props();

	let activeTab = $state<'revisions' | 'activity'>('revisions');
	let confirmingId = $state<number | null>(null);

	const triggerMeta: Record<string, { label: string; icon: typeof Save }> = {
		manual: { label: 'Manual Checkpoint', icon: Save },
		schedule: { label: 'Before Scheduling', icon: CalendarClock },
		unschedule: { label: 'Before Unscheduling', icon: CalendarX2 },
		ai_rewrite: { label: 'AI Rewrite', icon: Sparkles },
		pre_restore: { label: 'Before Restore', icon: RotateCcw },
		created: { label: 'Initial Version', icon: Plus }
	};

	const actionMeta: Record<string, { label: string; icon: typeof Plus }> = {
		created: { label: 'Created', icon: Plus },
		edited: { label: 'Edited', icon: Pencil },
		ai_rewrite: { label: 'AI Rewrite', icon: Sparkles },
		scheduled: { label: 'Scheduled', icon: CalendarClock },
		unscheduled: { label: 'Unscheduled', icon: CalendarX2 },
		archived: { label: 'Archived', icon: Archive },
		restored: { label: 'Restored from Archive', icon: ArchiveRestore },
		revision_restored: { label: 'Restored to Revision', icon: RotateCcw },
		posted: { label: 'Published', icon: Send }
	};

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
		return new Date(iso).toLocaleDateString(undefined, {
			month: 'short',
			day: 'numeric'
		});
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

	function cancelConfirm() {
		confirmingId = null;
	}

	function getTrigger(kind: string) {
		return triggerMeta[kind] ?? { label: kind, icon: Clock };
	}

	function getAction(action: string) {
		return actionMeta[action] ?? { label: action, icon: Clock };
	}

	function parseDetail(detail: string | null): Record<string, unknown> | null {
		if (!detail) return null;
		try {
			return JSON.parse(detail);
		} catch {
			return null;
		}
	}
</script>

<div class="history-panel">
	<header class="panel-header">
		<span class="panel-title">History</span>
		<button type="button" class="close-btn" onclick={onclose} aria-label="Close history">
			<X size={14} />
		</button>
	</header>

	<div class="tab-bar" role="tablist">
		<button
			type="button"
			role="tab"
			class="tab-btn"
			class:active={activeTab === 'revisions'}
			aria-selected={activeTab === 'revisions'}
			onclick={() => activeTab = 'revisions'}
		>
			Revisions
			{#if revisions.length > 0}
				<span class="tab-count">{revisions.length}</span>
			{/if}
		</button>
		<button
			type="button"
			role="tab"
			class="tab-btn"
			class:active={activeTab === 'activity'}
			aria-selected={activeTab === 'activity'}
			onclick={() => activeTab = 'activity'}
		>
			Activity
			{#if activity.length > 0}
				<span class="tab-count">{activity.length}</span>
			{/if}
		</button>
	</div>

	<div class="panel-body">
		{#if activeTab === 'revisions'}
			{#if revisions.length === 0}
				<div class="empty-state">
					<Clock size={20} />
					<p>No revisions yet</p>
					<p class="empty-hint">Revisions are created when you schedule, unschedule, or use AI rewrites.</p>
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
										<button type="button" class="restore-btn confirm" onclick={() => handleRestore(rev.id)}>
											Confirm
										</button>
										<button type="button" class="cancel-btn" onclick={cancelConfirm}>
											Cancel
										</button>
									{:else}
										<button type="button" class="restore-btn" onclick={() => handleRestore(rev.id)}>
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
		{:else}
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
											{new Date(detail.scheduled_for as string).toLocaleString(undefined, {
												month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit'
											})}
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
		{/if}
	</div>
</div>

<style>
	.history-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-surface);
		border-left: 1px solid var(--color-border-subtle);
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.panel-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		letter-spacing: -0.01em;
	}

	.close-btn {
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
	}

	.close-btn:hover {
		background: var(--color-border-subtle);
		color: var(--color-text);
	}

	.tab-bar {
		display: flex;
		gap: 0;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.tab-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 5px;
		padding: 8px 0;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: color 0.15s, border-color 0.15s;
	}

	.tab-btn:hover {
		color: var(--color-text);
	}

	.tab-btn.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.tab-count {
		font-size: 10px;
		padding: 1px 5px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
		font-weight: 600;
	}

	.panel-body {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}

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

	/* --- Revision items --- */

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

	.rev-icon, .act-icon {
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

	.rev-body, .act-body {
		flex: 1;
		min-width: 0;
	}

	.rev-header, .act-header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 6px;
	}

	.rev-label, .act-label {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
	}

	.rev-time, .act-time {
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

	/* --- Activity items --- */

	.activity-item {
		display: flex;
		gap: 10px;
		padding: 8px 14px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
	}

	.activity-item.ai {
		border-left: 2px solid color-mix(in srgb, #a855f7 60%, transparent);
	}

	.act-detail {
		margin: 3px 0 0;
		font-size: 11px;
		color: var(--color-text-subtle);
		opacity: 0.8;
	}
</style>
