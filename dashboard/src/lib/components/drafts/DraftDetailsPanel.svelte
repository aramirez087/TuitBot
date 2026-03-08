<script lang="ts">
	import { X, Plus, Tag, Clock, FileText, Sparkles, Radio } from 'lucide-svelte';
	import type { DraftSummary, ScheduledContentItem, ContentTag } from '$lib/api/types';

	let {
		draft,
		draftSummary,
		tags,
		allTags,
		onupdatemeta,
		onassigntag,
		onunassigntag,
		oncreatetag,
		onclose
	}: {
		draft: ScheduledContentItem | null;
		draftSummary: DraftSummary | null;
		tags: ContentTag[];
		allTags: ContentTag[];
		onupdatemeta: (data: { title?: string; notes?: string }) => void;
		onassigntag: (tagId: number) => void;
		onunassigntag: (tagId: number) => void;
		oncreatetag: (name: string) => void;
		onclose: () => void;
	} = $props();

	let titleValue = $state('');
	let notesValue = $state('');
	let titleTimer: ReturnType<typeof setTimeout> | undefined;
	let notesTimer: ReturnType<typeof setTimeout> | undefined;
	let tagPickerOpen = $state(false);
	let newTagInput = $state('');
	let notesExpanded = $state(false);

	// Sync local fields when draft changes
	$effect(() => {
		if (draft) {
			titleValue = draft.title ?? '';
			notesValue = draft.notes ?? '';
			notesExpanded = !!(draft.notes && draft.notes.length > 0);
		}
	});

	const unassignedTags = $derived(
		allTags.filter((t) => !tags.some((at) => at.id === t.id))
	);

	const sourceLabel = $derived(
		draftSummary?.source === 'assist'
			? 'AI Assist'
			: draftSummary?.source === 'discovery'
				? 'Discovery'
				: 'Manual'
	);

	const statusLabel = $derived(
		draftSummary?.status === 'scheduled'
			? 'Scheduled'
			: draftSummary?.status === 'posted'
				? 'Posted'
				: draftSummary?.archived_at
					? 'Archived'
					: 'Draft'
	);

	const isReady = $derived(
		!!draftSummary &&
			(draftSummary.content_preview?.trim().length ?? 0) > 10 &&
			(draftSummary.title !== null || draftSummary.content_preview.length > 20)
	);

	function handleTitleInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		titleValue = value;
		clearTimeout(titleTimer);
		titleTimer = setTimeout(() => {
			onupdatemeta({ title: value || undefined });
		}, 800);
	}

	function handleNotesInput(e: Event) {
		const value = (e.target as HTMLTextAreaElement).value;
		notesValue = value;
		clearTimeout(notesTimer);
		notesTimer = setTimeout(() => {
			onupdatemeta({ notes: value || undefined });
		}, 800);
	}

	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			clearTimeout(titleTimer);
			onupdatemeta({ title: titleValue || undefined });
			(e.target as HTMLInputElement).blur();
		}
	}

	function handleCreateTag() {
		const name = newTagInput.trim();
		if (!name) return;
		oncreatetag(name);
		newTagInput = '';
	}

	function handleNewTagKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			handleCreateTag();
		}
		if (e.key === 'Escape') {
			tagPickerOpen = false;
		}
	}

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

{#if draftSummary}
	<div class="details-panel">
		<div class="panel-header">
			<span class="panel-title">Details</span>
			<button class="close-btn" type="button" onclick={onclose} title="Close details (Cmd+Shift+D)">
				<X size={14} />
			</button>
		</div>

		<div class="panel-body">
			<!-- Title -->
			<div class="field">
				<label class="field-label" for="draft-title">Title</label>
				<input
					id="draft-title"
					class="field-input"
					type="text"
					placeholder="Add a title..."
					value={titleValue}
					oninput={handleTitleInput}
					onkeydown={handleTitleKeydown}
				/>
			</div>

			<!-- Notes -->
			<div class="field">
				<div class="field-label-row">
					<label class="field-label" for="draft-notes">Notes</label>
					{#if !notesExpanded && !notesValue}
						<button
							class="expand-btn"
							type="button"
							onclick={() => { notesExpanded = true; }}
						>
							<Plus size={11} /> Add
						</button>
					{/if}
				</div>
				{#if notesExpanded || notesValue}
					<textarea
						id="draft-notes"
						class="field-textarea"
						placeholder="Internal notes..."
						value={notesValue}
						oninput={handleNotesInput}
						rows="3"
					></textarea>
				{/if}
			</div>

			<!-- Tags -->
			<div class="field">
				<div class="field-label-row">
					<span class="field-label">Tags</span>
				</div>
				<div class="tags-container">
					{#each tags as tag (tag.id)}
						<span class="tag-pill">
							{#if tag.color}
								<span class="tag-color" style="background: {tag.color}"></span>
							{/if}
							{tag.name}
							<button
								class="tag-remove"
								type="button"
								title="Remove tag"
								onclick={() => onunassigntag(tag.id)}
							>
								<X size={10} />
							</button>
						</span>
					{/each}
					<button
						class="tag-add-btn"
						type="button"
						onclick={() => { tagPickerOpen = !tagPickerOpen; }}
					>
						<Plus size={11} />
						{tags.length === 0 ? 'Add tag' : ''}
					</button>
				</div>

				{#if tagPickerOpen}
					<div class="tag-picker">
						{#if unassignedTags.length > 0}
							{#each unassignedTags as tag (tag.id)}
								<button
									class="tag-option"
									type="button"
									onclick={() => { onassigntag(tag.id); }}
								>
									{#if tag.color}
										<span class="tag-dot" style="background: {tag.color}"></span>
									{/if}
									{tag.name}
								</button>
							{/each}
							<div class="tag-divider"></div>
						{/if}
						<div class="tag-create-row">
							<input
								class="tag-create-input"
								type="text"
								placeholder="New tag..."
								bind:value={newTagInput}
								onkeydown={handleNewTagKeydown}
							/>
							{#if newTagInput.trim()}
								<button class="tag-create-btn" type="button" onclick={handleCreateTag}>
									<Plus size={11} />
								</button>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<!-- Metadata -->
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
						<span class="meta-value">{formatDate(draftSummary.scheduled_for)}</span>
					</div>
				{/if}
			</div>

			<!-- Ready state -->
			<div class="ready-section">
				<span class="ready-dot" class:ready={isReady}></span>
				<span class="ready-label">{isReady ? 'Ready' : 'Not ready'}</span>
			</div>
		</div>
	</div>
{:else}
	<div class="details-panel details-empty">
		<div class="panel-header">
			<span class="panel-title">Details</span>
			<button class="close-btn" type="button" onclick={onclose} title="Close details">
				<X size={14} />
			</button>
		</div>
		<div class="empty-msg">
			<Tag size={20} />
			<p>Select a draft to see details</p>
		</div>
	</div>
{/if}

<style>
	.details-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-surface);
		border-left: 1px solid var(--color-border-subtle);
		overflow-y: auto;
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
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
		text-transform: uppercase;
		letter-spacing: 0.04em;
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
		color: var(--color-text-muted);
		cursor: pointer;
		padding: 0;
	}

	.close-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.panel-body {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 14px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.field-label-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.field-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.field-input {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid transparent;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		outline: none;
		transition: all 0.15s ease;
	}

	.field-input:hover {
		border-color: var(--color-border-subtle);
	}

	.field-input:focus {
		border-color: var(--color-accent);
		background: var(--color-base);
	}

	.field-input::placeholder {
		color: var(--color-text-subtle);
	}

	.field-textarea {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid transparent;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		line-height: 1.5;
		outline: none;
		resize: vertical;
		min-height: 56px;
		max-height: 160px;
		font-family: inherit;
		transition: all 0.15s ease;
	}

	.field-textarea:hover {
		border-color: var(--color-border-subtle);
	}

	.field-textarea:focus {
		border-color: var(--color-accent);
		background: var(--color-base);
	}

	.field-textarea::placeholder {
		color: var(--color-text-subtle);
	}

	.expand-btn {
		display: flex;
		align-items: center;
		gap: 2px;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		padding: 2px 4px;
		border-radius: 3px;
		transition: all 0.1s ease;
	}

	.expand-btn:hover {
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	/* Tags */
	.tags-container {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		align-items: center;
	}

	.tag-pill {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 10px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 500;
	}

	.tag-color {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tag-remove {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		height: 14px;
		border: none;
		border-radius: 50%;
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
		padding: 0;
		opacity: 0.6;
		transition: opacity 0.1s ease;
	}

	.tag-remove:hover {
		opacity: 1;
	}

	.tag-add-btn {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 2px 8px;
		border: 1px dashed var(--color-border-subtle);
		border-radius: 10px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.tag-add-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.tag-picker {
		margin-top: 4px;
		padding: 6px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-base);
	}

	.tag-option {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 5px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		transition: background 0.1s ease;
	}

	.tag-option:hover {
		background: var(--color-surface-hover);
	}

	.tag-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tag-divider {
		height: 1px;
		background: var(--color-border-subtle);
		margin: 4px 0;
	}

	.tag-create-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.tag-create-input {
		flex: 1;
		padding: 4px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		outline: none;
	}

	.tag-create-input:focus {
		border-color: var(--color-accent);
	}

	.tag-create-input::placeholder {
		color: var(--color-text-subtle);
	}

	.tag-create-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: var(--color-accent);
		color: #fff;
		cursor: pointer;
		padding: 0;
	}

	/* Meta section */
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

	/* Ready state */
	.ready-section {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 0;
	}

	.ready-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-warning, #d29922);
		flex-shrink: 0;
	}

	.ready-dot.ready {
		background: var(--color-success, #2ea043);
	}

	.ready-label {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	/* Empty state */
	.details-empty {
		justify-content: flex-start;
	}

	.empty-msg {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 40px 20px;
		color: var(--color-text-subtle);
	}

	.empty-msg p {
		font-size: 12px;
		margin: 0;
	}
</style>
