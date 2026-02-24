<script lang="ts">
	import { fly } from 'svelte/transition';
	import {
		MessageSquare,
		FileText,
		BookOpen,
		CheckCircle,
		XCircle,
		Pencil,
		X
	} from 'lucide-svelte';
	import type { ApprovalItem } from '$lib/api';

	interface Props {
		item: ApprovalItem;
		focused: boolean;
		editing: boolean;
		onApprove: (id: number) => void;
		onReject: (id: number) => void;
		onStartEdit: (id: number) => void;
		onSaveEdit: (id: number, content: string) => void;
		onCancelEdit: () => void;
	}

	let {
		item,
		focused,
		editing,
		onApprove,
		onReject,
		onStartEdit,
		onSaveEdit,
		onCancelEdit
	}: Props = $props();

	let editContent = $state('');
	let textareaEl: HTMLTextAreaElement | undefined = $state();

	const charCount = $derived(editing ? editContent.length : item.generated_content.length);
	const isOverLimit = $derived(charCount > 280);

	/* eslint-disable @typescript-eslint/no-explicit-any */
	const iconMap: Record<string, any> = {
		reply: MessageSquare,
		tweet: FileText,
		thread_tweet: BookOpen
	};
	/* eslint-enable @typescript-eslint/no-explicit-any */

	const Icon = $derived(iconMap[item.action_type] ?? FileText);
	const typeLabel = $derived(item.action_type.replace(/_/g, ' '));

	const statusClass = $derived(
		item.status === 'pending'
			? 'status-pending'
			: item.status === 'approved'
				? 'status-approved'
				: 'status-rejected'
	);

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

	function handleStartEdit() {
		editContent = item.generated_content;
		onStartEdit(item.id);
	}

	function handleSave() {
		if (editContent.trim() && !isOverLimit) {
			onSaveEdit(item.id, editContent.trim());
		}
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onCancelEdit();
		} else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			handleSave();
		}
	}

	$effect(() => {
		if (editing && textareaEl) {
			textareaEl.focus();
			textareaEl.setSelectionRange(textareaEl.value.length, textareaEl.value.length);
		}
	});
</script>

<div class="card" class:focused class:editing transition:fly={{ x: 300, duration: 250 }}>
	<div class="card-icon {statusClass}">
		<Icon size={16} />
	</div>

	<div class="card-body">
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

		{#if editing}
			<div class="editor">
				<textarea
					bind:this={textareaEl}
					bind:value={editContent}
					class="editor-textarea"
					rows="4"
					onkeydown={handleEditKeydown}
				></textarea>
				<div class="editor-footer">
					<span class="char-count" class:over-limit={isOverLimit}>
						{charCount}/280
					</span>
					<div class="editor-actions">
						<button class="editor-save" onclick={handleSave} disabled={!editContent.trim() || isOverLimit}>
							Save
						</button>
						<button class="editor-cancel" onclick={onCancelEdit}>
							Cancel
						</button>
					</div>
				</div>
			</div>
		{:else}
			<div class="card-content">
				<p class="content-text">{item.generated_content}</p>
				<span class="char-count" class:over-limit={isOverLimit}>{charCount}/280</span>
			</div>
		{/if}

		<div class="card-meta">
			{#if item.topic}
				<span class="meta-tag topic">{item.topic}</span>
			{/if}
			{#if item.archetype}
				<span class="meta-tag archetype">{item.archetype}</span>
			{/if}
		</div>

		{#if item.status === 'pending' && !editing}
			<div class="card-actions">
				<button class="action-btn approve" onclick={() => onApprove(item.id)}>
					<CheckCircle size={14} />
					Approve
					<kbd>a</kbd>
				</button>
				<button class="action-btn reject" onclick={() => onReject(item.id)}>
					<XCircle size={14} />
					Reject
					<kbd>r</kbd>
				</button>
				<button class="action-btn edit" onclick={handleStartEdit}>
					<Pencil size={14} />
					Edit
					<kbd>e</kbd>
				</button>
			</div>
		{:else if item.status !== 'pending'}
			<div class="card-actions-readonly">
				{#if item.status === 'approved'}
					<span class="readonly-badge approved"><CheckCircle size={12} /> Approved</span>
				{:else}
					<span class="readonly-badge rejected"><X size={12} /> Rejected</span>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.card {
		display: flex;
		gap: 12px;
		padding: 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		transition: background-color 0.1s ease;
		position: relative;
	}

	.card:last-child {
		border-bottom: none;
	}

	.card:hover {
		background-color: var(--color-surface-hover);
	}

	.card.focused {
		background-color: var(--color-surface-active);
		border-left: 3px solid var(--color-accent);
		padding-left: 13px;
	}

	.card.editing {
		background-color: var(--color-surface-active);
	}

	.card-icon {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 2px;
	}

	.card-icon.status-pending {
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		color: var(--color-warning);
	}

	.card-icon.status-approved {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.card-icon.status-rejected {
		background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.card-body {
		flex: 1;
		min-width: 0;
	}

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

	.card-content {
		margin-bottom: 8px;
	}

	.content-text {
		margin: 0 0 4px;
		font-size: 13px;
		color: var(--color-text);
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.char-count {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-variant-numeric: tabular-nums;
	}

	.char-count.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	.editor {
		margin-bottom: 8px;
	}

	.editor-textarea {
		width: 100%;
		padding: 10px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background-color: var(--color-base);
		color: var(--color-text);
		font-family: var(--font-mono);
		font-size: 13px;
		line-height: 1.5;
		resize: vertical;
		outline: none;
		box-sizing: border-box;
	}

	.editor-textarea:focus {
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 30%, transparent);
	}

	.editor-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: 8px;
	}

	.editor-actions {
		display: flex;
		gap: 6px;
	}

	.editor-save {
		padding: 4px 12px;
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		background-color: var(--color-accent);
		color: white;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: opacity 0.15s ease;
	}

	.editor-save:hover:not(:disabled) {
		opacity: 0.9;
	}

	.editor-save:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.editor-cancel {
		padding: 4px 12px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background-color: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.editor-cancel:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}

	.card-meta {
		display: flex;
		gap: 6px;
		margin-bottom: 10px;
		flex-wrap: wrap;
	}

	.meta-tag {
		font-size: 11px;
		font-weight: 500;
		padding: 2px 8px;
		border-radius: 4px;
	}

	.meta-tag.topic {
		background-color: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.meta-tag.archetype {
		background-color: color-mix(in srgb, var(--color-text-subtle) 15%, transparent);
		color: var(--color-text-muted);
	}

	.card-actions {
		display: flex;
		gap: 8px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background-color: var(--color-surface);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn kbd {
		font-size: 10px;
		padding: 0 4px;
		border-radius: 3px;
		background-color: var(--color-base);
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
		border: 1px solid var(--color-border-subtle);
	}

	.action-btn.approve {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 30%, var(--color-border));
	}

	.action-btn.approve:hover {
		background-color: color-mix(in srgb, var(--color-success) 10%, transparent);
	}

	.action-btn.reject {
		color: var(--color-danger);
		border-color: color-mix(in srgb, var(--color-danger) 30%, var(--color-border));
	}

	.action-btn.reject:hover {
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.action-btn.edit {
		color: var(--color-accent);
		border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
	}

	.action-btn.edit:hover {
		background-color: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.card-actions-readonly {
		display: flex;
	}

	.readonly-badge {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		font-weight: 600;
		padding: 3px 8px;
		border-radius: 4px;
	}

	.readonly-badge.approved {
		background-color: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.readonly-badge.rejected {
		background-color: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}
</style>
