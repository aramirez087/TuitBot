<script lang="ts">
	import { Film, ShieldAlert } from 'lucide-svelte';
	import { api, type ApprovalItem } from '$lib/api';

	interface Props {
		item: ApprovalItem;
		editing: boolean;
		editContent: string;
		isOverLimit: boolean;
		charCount: number;
		mediaPaths: string[];
		risks: string[];
		textareaEl?: HTMLTextAreaElement;
		onSave: () => void;
		onCancelEdit: () => void;
		onEditKeydown: (e: KeyboardEvent) => void;
	}

	let {
		item,
		editing,
		editContent = $bindable(''),
		isOverLimit,
		charCount,
		mediaPaths,
		risks,
		textareaEl = $bindable(undefined),
		onSave,
		onCancelEdit,
		onEditKeydown
	}: Props = $props();
</script>

{#if editing}
	<div class="editor">
		<textarea
			bind:this={textareaEl}
			bind:value={editContent}
			class="editor-textarea"
			rows="4"
			onkeydown={onEditKeydown}
		></textarea>
		<div class="editor-footer">
			<span class="char-count" class:over-limit={isOverLimit}>
				{charCount}/280
			</span>
			<div class="editor-actions">
				<button
					class="editor-save"
					onclick={onSave}
					disabled={!editContent.trim() || isOverLimit}
				>
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

<style>
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
</style>
