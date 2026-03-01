<script lang="ts">
	import type { CalendarItem } from '$lib/api';
	import { MessageSquare, FileText, GitBranch, BarChart3, X, Pencil } from 'lucide-svelte';

	let {
		item,
		oncancel,
		onedit
	}: {
		item: CalendarItem;
		oncancel?: (id: number) => void;
		onedit?: (id: number) => void;
	} = $props();

	let expanded = $state(false);

	const typeColors: Record<string, string> = {
		tweet: 'var(--color-accent)',
		thread: '#a371f7',
		reply: 'var(--color-success)'
	};

	const typeLabels: Record<string, string> = {
		tweet: 'Tweet',
		thread: 'Thread',
		reply: 'Reply'
	};

	const statusLabels: Record<string, string> = {
		posted: 'Posted',
		sent: 'Posted',
		pending: 'Pending',
		scheduled: 'Scheduled',
		cancelled: 'Cancelled',
		failed: 'Failed'
	};

	const color = $derived(typeColors[item.content_type] || 'var(--color-text-muted)');
	const preview = $derived(
		item.content.length > 40 ? item.content.slice(0, 40) + '...' : item.content
	);
	const isPosted = $derived(item.status === 'posted' || item.status === 'sent');
	const isScheduled = $derived(item.status === 'scheduled');
	const canEdit = $derived(isScheduled && item.source === 'manual');

	const TypeIcon = $derived(
		item.content_type === 'reply' ? MessageSquare : item.content_type === 'thread' ? GitBranch : FileText
	);

	function toggleExpanded(event?: Event) {
		event?.stopPropagation();
		expanded = !expanded;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter' && event.key !== ' ') return;
		event.preventDefault();
		event.stopPropagation();
		expanded = !expanded;
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
	class="content-item"
	class:expanded
	class:posted={isPosted}
	class:scheduled={isScheduled}
	style:--item-color={color}
	onclick={toggleExpanded}
	onkeydown={handleKeydown}
	role="button"
	tabindex="0"
>
	<div class="item-header">
		<span class="item-icon">
			<TypeIcon size={12} />
		</span>
		<span class="item-preview">{preview}</span>
	</div>

	{#if expanded}
		<div class="item-details">
			<div class="item-full-content">{item.content}</div>

			<div class="item-meta">
				<span class="item-badge">{typeLabels[item.content_type] || item.content_type}</span>
				<span class="item-status">{statusLabels[item.status] || item.status}</span>
				{#if item.topic}
					<span class="item-topic">{item.topic}</span>
				{/if}
				{#if item.target_author}
					<span class="item-author">@{item.target_author}</span>
				{/if}
			</div>

			{#if isPosted && item.performance_score != null}
				<div class="item-performance">
					<BarChart3 size={12} />
					<span>Score: {item.performance_score.toFixed(1)}</span>
				</div>
			{/if}

			{#if canEdit}
				<div class="item-actions">
					{#if onedit}
						<button class="action-btn edit" onclick={(e) => { e.stopPropagation(); onedit(item.id); }}>
							<Pencil size={12} />
							Edit
						</button>
					{/if}
					{#if oncancel}
						<button class="action-btn cancel" onclick={(e) => { e.stopPropagation(); oncancel(item.id); }}>
							<X size={12} />
							Cancel
						</button>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.content-item {
		display: block;
		width: 100%;
		text-align: left;
		padding: 4px 8px;
		border-radius: 4px;
		border: 1px solid transparent;
		background: color-mix(in srgb, var(--item-color) 10%, transparent);
		cursor: pointer;
		transition: all 0.15s ease;
		font-size: 11px;
		line-height: 1.4;
		color: var(--color-text);
	}

	.content-item:hover {
		background: color-mix(in srgb, var(--item-color) 18%, transparent);
	}

	.content-item.posted {
		border-left: 2px solid var(--item-color);
	}

	.content-item.scheduled {
		border: 1px dashed var(--item-color);
	}

	.item-header {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
	}

	.item-icon {
		color: var(--item-color);
		flex-shrink: 0;
		display: flex;
		align-items: center;
	}

	.item-preview {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.item-details {
		margin-top: 6px;
		padding-top: 6px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.item-full-content {
		font-size: 12px;
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
		margin-bottom: 8px;
	}

	.item-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-items: center;
		font-size: 10px;
	}

	.item-badge {
		background: color-mix(in srgb, var(--item-color) 20%, transparent);
		color: var(--item-color);
		padding: 1px 6px;
		border-radius: 10px;
		font-weight: 500;
	}

	.item-status,
	.item-topic,
	.item-author {
		color: var(--color-text-muted);
	}

	.item-performance {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-top: 6px;
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.item-actions {
		display: flex;
		gap: 8px;
		margin-top: 8px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.action-btn.cancel:hover {
		border-color: var(--color-danger);
		color: var(--color-danger);
	}
</style>
