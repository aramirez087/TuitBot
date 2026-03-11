<script lang="ts">
	import { fly } from 'svelte/transition';
	import {
		MessageSquare,
		FileText,
		BookOpen,
		CheckCircle,
		XCircle,
		Pencil,
		X,
		Film,
		ShieldAlert,
		Clock
	} from 'lucide-svelte';
	import { api, type ApprovalItem } from '$lib/api';
	import { formatInAccountTz } from '$lib/utils/timezone';
	import ApprovalEditHistory from './ApprovalEditHistory.svelte';
	import ApprovalRejectDialog from './ApprovalRejectDialog.svelte';

	interface Props {
		item: ApprovalItem;
		focused: boolean;
		editing: boolean;
		timezone?: string;
		onApprove: (id: number) => void;
		onReject: (id: number, notes?: string) => void;
		onStartEdit: (id: number) => void;
		onSaveEdit: (id: number, content: string) => void;
		onCancelEdit: () => void;
	}

	let {
		item,
		focused,
		editing,
		timezone = 'UTC',
		onApprove,
		onReject,
		onStartEdit,
		onSaveEdit,
		onCancelEdit
	}: Props = $props();

	let editContent = $state('');
	let textareaEl: HTMLTextAreaElement | undefined = $state();
	let showRejectDialog = $state(false);

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

	// Media paths from the approval item.
	const mediaPaths = $derived(item.media_paths ?? []);

	// Detected risks parsed from the item.
	const risks = $derived(
		Array.isArray(item.detected_risks) ? item.detected_risks : []
	);

	const statusClass = $derived(
		item.status === 'pending'
			? 'status-pending'
			: item.status === 'approved'
				? 'status-approved'
				: item.status === 'scheduled'
					? 'status-scheduled'
					: 'status-rejected'
	);

	const scheduledLabel = $derived(
		item.scheduled_for
			? formatInAccountTz(item.scheduled_for, timezone, {
					month: 'short',
					day: 'numeric',
					hour: '2-digit',
					minute: '2-digit',
					timeZoneName: 'short'
				})
			: null
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

		{#if scheduledLabel}
			<div class="card-schedule">
				<Clock size={12} />
				<span>Scheduled for {scheduledLabel}</span>
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

		{#if mediaPaths.length > 0}
			<div class="card-media-previews">
				{#each mediaPaths as mediaPath}
					{#if mediaPath.endsWith('.mp4')}
						<!-- svelte-ignore a11y_media_has_caption -->
						<video src={api.media.fileUrl(mediaPath)} class="media-thumb-img"></video>
						<span class="media-thumb-badge"><Film size={10} /></span>
					{:else}
						<img src={api.media.fileUrl(mediaPath)} alt="Attached" class="media-thumb-img" />
					{/if}
				{/each}
			</div>
		{/if}

		<div class="card-meta">
			{#if item.topic}
				<span class="meta-tag topic">{item.topic}</span>
			{/if}
			{#if item.archetype}
				<span class="meta-tag archetype">{item.archetype}</span>
			{/if}
			{#if item.reason}
				<span class="meta-tag reason">{item.reason}</span>
			{/if}
		</div>

		{#if risks.length > 0}
			<div class="card-risks">
				<ShieldAlert size={11} />
				{#each risks as risk}
					<span class="risk-chip">{risk}</span>
				{/each}
			</div>
		{/if}

		{#if item.qa_score > 0}
			<div class="card-qa">
				<span
					class="qa-score-badge"
					class:qa-good={item.qa_score > 80}
					class:qa-warn={item.qa_score > 60 && item.qa_score <= 80}
					class:qa-bad={item.qa_score <= 60}
				>
					QA {Math.round(item.qa_score)}
				</span>
				{#if item.qa_hard_flags?.length > 0}
					<span class="qa-flag-count hard">{item.qa_hard_flags.length} hard</span>
				{/if}
				{#if item.qa_soft_flags?.length > 0}
					<span class="qa-flag-count soft">{item.qa_soft_flags.length} soft</span>
				{/if}
				{#if item.qa_override_by}
					<span class="qa-override">override by {item.qa_override_by}</span>
				{/if}
			</div>
		{/if}

		{#if item.status !== 'pending' && (item.reviewed_by || item.review_notes)}
			<div class="card-review-info">
				{#if item.reviewed_by}
					<span class="review-by">Reviewed by {item.reviewed_by}</span>
				{/if}
				{#if item.review_notes}
					<span class="review-notes">{item.review_notes}</span>
				{/if}
			</div>
		{/if}

		<ApprovalEditHistory approvalId={item.id} />

		{#if showRejectDialog && item.status === 'pending'}
			<ApprovalRejectDialog
				itemId={item.id}
				onConfirm={(id, notes) => {
					showRejectDialog = false;
					onReject(id, notes || undefined);
				}}
				onCancel={() => (showRejectDialog = false)}
			/>
		{:else if item.status === 'pending' && !editing}
			<div class="card-actions">
				<button class="action-btn approve" onclick={() => onApprove(item.id)}>
					<CheckCircle size={14} />
					Approve
					<kbd>a</kbd>
				</button>
				<button class="action-btn reject" onclick={() => (showRejectDialog = true)}>
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
				{:else if item.status === 'scheduled'}
					<span class="readonly-badge scheduled"><Clock size={12} /> Scheduled</span>
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

	.card-icon.status-scheduled {
		background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
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

	.card-media-previews {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 8px;
		position: relative;
	}

	.media-thumb-img {
		width: 64px;
		height: 64px;
		object-fit: cover;
		border-radius: 6px;
		border: 1px solid var(--color-border-subtle);
	}

	.media-thumb-badge {
		position: absolute;
		bottom: 4px;
		left: 4px;
		display: flex;
		align-items: center;
		padding: 1px 4px;
		border-radius: 3px;
		background: rgba(0, 0, 0, 0.7);
		color: #fff;
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

	.meta-tag.reason {
		background-color: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	.card-risks {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-bottom: 8px;
		color: var(--color-warning);
		flex-wrap: wrap;
	}

	.risk-chip {
		font-size: 10px;
		font-weight: 500;
		padding: 1px 6px;
		border-radius: 3px;
		background-color: color-mix(in srgb, var(--color-warning) 10%, transparent);
		color: var(--color-warning);
	}

	.card-qa {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 8px;
		flex-wrap: wrap;
	}

	.qa-score-badge {
		font-size: 11px;
		font-weight: 700;
		padding: 2px 8px;
		border-radius: 4px;
		font-variant-numeric: tabular-nums;
	}

	.qa-score-badge.qa-good {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.qa-score-badge.qa-warn {
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		color: var(--color-warning);
	}

	.qa-score-badge.qa-bad {
		background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.qa-flag-count {
		font-size: 10px;
		font-weight: 600;
		padding: 1px 6px;
		border-radius: 3px;
	}

	.qa-flag-count.hard {
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	.qa-flag-count.soft {
		background-color: color-mix(in srgb, var(--color-warning) 10%, transparent);
		color: var(--color-warning);
	}

	.qa-override {
		font-size: 10px;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	.card-review-info {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
		font-size: 11px;
	}

	.review-by {
		color: var(--color-text-subtle);
		font-weight: 500;
	}

	.review-notes {
		color: var(--color-text-muted);
		font-style: italic;
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

	.readonly-badge.scheduled {
		background-color: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}
</style>
