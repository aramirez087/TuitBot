<script lang="ts">
	import { Clock, FileText, Sparkles, Radio } from 'lucide-svelte';
	import type { DraftSummary } from '$lib/api/types';
	import { formatInAccountTz } from '$lib/utils/timezone';

	let {
		draftSummary,
		timezone = 'UTC'
	}: {
		draftSummary: DraftSummary;
		timezone?: string;
	} = $props();

	const sourceLabel = $derived(
		draftSummary.source === 'assist'
			? 'AI Assist'
			: draftSummary.source === 'discovery'
				? 'Discovery'
				: 'Manual'
	);

	const statusLabel = $derived(
		draftSummary.status === 'scheduled'
			? 'Scheduled'
			: draftSummary.status === 'posted'
				? 'Posted'
				: draftSummary.archived_at
					? 'Archived'
					: 'Draft'
	);

	function relativeTime(dateStr: string): string {
		const now = Date.now();
		const then = new Date(dateStr).getTime();
		const diffMs = now - then;
		const diffSec = Math.floor(diffMs / 1000);
		if (diffSec < 60) return 'just now';
		const diffMin = Math.floor(diffSec / 60);
		if (diffMin < 60) return `${diffMin}m ago`;
		const diffHr = Math.floor(diffMin / 60);
		if (diffHr < 24) return `${diffHr}h ago`;
		const diffDays = Math.floor(diffHr / 24);
		if (diffDays === 1) return 'yesterday';
		if (diffDays < 7) return `${diffDays}d ago`;
		const d = new Date(dateStr);
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	}

	function formatDate(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}
</script>

<div class="meta-section">
	<div class="meta-row">
		<span class="meta-label">
			<FileText size={12} />
			Type
		</span>
		<span class="meta-value type-badge">
			{draftSummary.content_type}
		</span>
	</div>
	<div class="meta-row">
		<span class="meta-label">
			<Sparkles size={12} />
			Source
		</span>
		<span class="meta-value">{sourceLabel}</span>
	</div>
	<div class="meta-row">
		<span class="meta-label">
			<Radio size={12} />
			Status
		</span>
		<span class="meta-value">{statusLabel}</span>
	</div>
	<div class="meta-row">
		<span class="meta-label">
			<Clock size={12} />
			Created
		</span>
		<span class="meta-value">{formatDate(draftSummary.created_at)}</span>
	</div>
	<div class="meta-row">
		<span class="meta-label">
			<Clock size={12} />
			Updated
		</span>
		<span class="meta-value">{relativeTime(draftSummary.updated_at)}</span>
	</div>
	{#if draftSummary.scheduled_for}
		<div class="meta-row">
			<span class="meta-label">
				<Clock size={12} />
				Scheduled
			</span>
			<span class="meta-value">{formatInAccountTz(draftSummary.scheduled_for, timezone, {
				month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit',
				timeZoneName: 'short'
			})}</span>
		</div>
	{/if}
</div>

<style>
	.meta-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.meta-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.meta-label {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.meta-value {
		font-size: 12px;
		color: var(--color-text);
		text-align: right;
	}

	.meta-value.type-badge {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		padding: 1px 6px;
		border-radius: 3px;
	}
</style>
