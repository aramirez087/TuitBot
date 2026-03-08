<script lang="ts">
	import type { DraftSummary } from '$lib/api/types';

	let { draft, selected, onclick }: {
		draft: DraftSummary;
		selected: boolean;
		onclick: () => void;
	} = $props();

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

	const displayTitle = $derived(
		draft.title ?? (draft.content_preview?.trim() || 'Untitled draft')
	);
</script>

<button
	class="rail-entry"
	class:selected
	{onclick}
	type="button"
	aria-current={selected ? 'true' : undefined}
>
	<div class="entry-top">
		<span class="entry-title">{displayTitle}</span>
		<span class="entry-time">{relativeTime(draft.updated_at)}</span>
	</div>
	<div class="entry-meta">
		<span class="type-badge">{draft.content_type}</span>
		{#if draft.status === 'scheduled' && draft.scheduled_for}
			<span class="scheduled-badge">scheduled</span>
		{/if}
	</div>
</button>

<style>
	.rail-entry {
		display: flex;
		flex-direction: column;
		gap: 4px;
		width: 100%;
		padding: 10px 12px;
		border: none;
		border-radius: 6px;
		background: transparent;
		cursor: pointer;
		text-align: left;
		transition: background-color 0.12s ease;
	}

	.rail-entry:hover {
		background: var(--color-surface-hover);
	}

	.rail-entry.selected {
		background: var(--color-surface-active);
	}

	.entry-top {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.entry-title {
		flex: 1;
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.entry-time {
		flex-shrink: 0;
		font-size: 11px;
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
	}

	.entry-meta {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.type-badge {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		padding: 1px 6px;
		border-radius: 3px;
	}

	.scheduled-badge {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-warning, #d29922);
		background: color-mix(in srgb, var(--color-warning, #d29922) 10%, transparent);
		padding: 1px 6px;
		border-radius: 3px;
	}
</style>
