<script lang="ts">
	import { onMount } from 'svelte';
	import { PenLine, Send, Calendar, Trash2, Sparkles, Loader2, Plus } from 'lucide-svelte';
	import { api, type ScheduledContentItem } from '$lib/api';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { tweetWeightedLen, MAX_TWEET_CHARS } from '$lib/utils/tweetLength';

	let drafts = $state<ScheduledContentItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let editingId = $state<number | null>(null);
	let editContent = $state('');
	let creating = $state(false);
	let newContent = $state('');
	let newType = $state('tweet');
	let improvingId = $state<number | null>(null);
	let schedulingId = $state<number | null>(null);
	let scheduleTime = $state('');

	async function loadDrafts() {
		loading = true;
		error = null;
		try {
			drafts = await api.drafts.list();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load drafts';
		} finally {
			loading = false;
		}
	}

	async function createDraft() {
		if (!newContent.trim()) return;
		try {
			await api.drafts.create(newType, newContent.trim());
			newContent = '';
			creating = false;
			await loadDrafts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create draft';
		}
	}

	async function saveDraft(id: number) {
		try {
			await api.drafts.edit(id, editContent);
			editingId = null;
			await loadDrafts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save draft';
		}
	}

	async function deleteDraft(id: number) {
		try {
			await api.drafts.delete(id);
			await loadDrafts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete draft';
		}
	}

	async function publishDraft(id: number) {
		try {
			await api.drafts.publish(id);
			await loadDrafts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to publish draft';
		}
	}

	async function scheduleDraft(id: number) {
		if (!scheduleTime) return;
		try {
			await api.drafts.schedule(id, scheduleTime);
			schedulingId = null;
			scheduleTime = '';
			await loadDrafts();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to schedule draft';
		}
	}

	async function improveDraft(id: number, content: string) {
		improvingId = id;
		try {
			const result = await api.assist.improve(content);
			editingId = id;
			editContent = result.content;
		} catch (e) {
			error = e instanceof Error ? e.message : 'AI improve failed — is LLM configured?';
		} finally {
			improvingId = null;
		}
	}

	async function aiGenerate() {
		creating = true;
		try {
			const topics = await api.assist.topics();
			const topic = topics.topics[0]?.topic ?? 'general';
			const result = await api.assist.tweet(topic);
			newContent = result.content;
		} catch (e) {
			error = e instanceof Error ? e.message : 'AI generation failed — is LLM configured?';
		}
	}

	function startEdit(d: ScheduledContentItem) {
		editingId = d.id;
		editContent = d.content;
	}

	onMount(loadDrafts);
</script>

<svelte:head>
	<title>Drafts — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<h1>Drafts</h1>
	<div class="header-actions">
		<button class="ai-btn" onclick={aiGenerate}>
			<Sparkles size={14} />
			AI Generate
		</button>
		<button class="compose-btn" onclick={() => { creating = true; newContent = ''; }}>
			<Plus size={14} />
			New Draft
		</button>
	</div>
</div>

{#if creating}
	<div class="draft-create card">
		<div class="create-header">
			<select class="type-select" bind:value={newType}>
				<option value="tweet">Tweet</option>
				<option value="thread">Thread</option>
			</select>
			<span class="char-count" class:over={tweetWeightedLen(newContent) > MAX_TWEET_CHARS}>
				{tweetWeightedLen(newContent)}/{MAX_TWEET_CHARS}
			</span>
		</div>
		<textarea
			class="draft-textarea"
			placeholder="Write your draft..."
			bind:value={newContent}
			rows="4"
		></textarea>
		<div class="create-actions">
			<button class="cancel-btn" onclick={() => { creating = false; newContent = ''; }}>Cancel</button>
			<button class="save-btn" onclick={createDraft} disabled={!newContent.trim()}>Save Draft</button>
		</div>
	</div>
{/if}

{#if loading && drafts.length === 0}
	<div class="loading-state">
		<Loader2 size={20} class="spinner" />
		<span>Loading drafts...</span>
	</div>
{:else if error && drafts.length === 0}
	<ErrorState message={error} onretry={loadDrafts} />
{:else if drafts.length === 0 && !creating}
	<EmptyState
		title="No drafts yet"
		description="Create a draft to start composing content. Use AI Assist to generate ideas."
		actionLabel="New Draft"
		onaction={() => { creating = true; newContent = ''; }}
	/>
{:else}
	<div class="drafts-list">
		{#each drafts as draft (draft.id)}
			<div class="draft-card card">
				{#if editingId === draft.id}
					<textarea
						class="draft-textarea"
						bind:value={editContent}
						rows="4"
					></textarea>
					<div class="card-actions">
						<span class="char-count" class:over={tweetWeightedLen(editContent) > MAX_TWEET_CHARS}>
							{tweetWeightedLen(editContent)}/{MAX_TWEET_CHARS}
						</span>
						<div class="action-group">
							<button class="cancel-btn" onclick={() => { editingId = null; }}>Cancel</button>
							<button class="save-btn" onclick={() => saveDraft(draft.id)}>Save</button>
						</div>
					</div>
				{:else if schedulingId === draft.id}
					<p class="draft-content">{draft.content}</p>
					<div class="schedule-row">
						<input type="datetime-local" class="text-input" bind:value={scheduleTime} />
						<button class="save-btn" onclick={() => scheduleDraft(draft.id)} disabled={!scheduleTime}>Schedule</button>
						<button class="cancel-btn" onclick={() => { schedulingId = null; scheduleTime = ''; }}>Cancel</button>
					</div>
				{:else}
					<div class="card-header">
						<span class="type-badge">{draft.content_type}</span>
						<span class="draft-date">{new Date(draft.created_at).toLocaleDateString()}</span>
					</div>
					<p class="draft-content">{draft.content}</p>
					<div class="card-actions">
						<button class="icon-btn" title="Improve with AI" onclick={() => improveDraft(draft.id, draft.content)} disabled={improvingId === draft.id}>
							{#if improvingId === draft.id}
								<Loader2 size={14} class="spinner" />
							{:else}
								<Sparkles size={14} />
							{/if}
						</button>
						<button class="icon-btn" title="Edit" onclick={() => startEdit(draft)}>
							<PenLine size={14} />
						</button>
						<button class="icon-btn" title="Schedule" onclick={() => { schedulingId = draft.id; scheduleTime = ''; }}>
							<Calendar size={14} />
						</button>
						<button class="icon-btn danger" title="Delete" onclick={() => deleteDraft(draft.id)}>
							<Trash2 size={14} />
						</button>
						<div class="action-spacer"></div>
						<button class="publish-btn" onclick={() => publishDraft(draft.id)}>
							<Send size={14} />
							Publish
						</button>
					</div>
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 20px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.header-actions {
		display: flex;
		gap: 8px;
	}

	.compose-btn, .ai-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.compose-btn {
		background: var(--color-accent);
		color: #fff;
	}

	.compose-btn:hover {
		background: var(--color-accent-hover);
	}

	.ai-btn {
		background: transparent;
		border: 1px solid var(--color-accent);
		color: var(--color-accent);
	}

	.ai-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.card {
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 16px;
	}

	.draft-create {
		margin-bottom: 16px;
	}

	.create-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.type-select {
		padding: 4px 8px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 12px;
	}

	.draft-textarea {
		width: 100%;
		padding: 10px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
		resize: vertical;
		line-height: 1.5;
	}

	.draft-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.create-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}

	.drafts-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.draft-card {
		/* inherits .card */
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.type-badge {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		padding: 2px 8px;
		border-radius: 4px;
	}

	.draft-date {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.draft-content {
		font-size: 13px;
		color: var(--color-text);
		line-height: 1.5;
		margin: 0 0 12px;
		white-space: pre-wrap;
	}

	.card-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.icon-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.icon-btn.danger:hover:not(:disabled) {
		color: var(--color-danger);
		border-color: var(--color-danger);
	}

	.action-spacer {
		flex: 1;
	}

	.publish-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.publish-btn:hover {
		background: var(--color-accent-hover);
	}

	.cancel-btn {
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.cancel-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.save-btn {
		padding: 6px 14px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.action-group {
		display: flex;
		gap: 8px;
		margin-left: auto;
	}

	.schedule-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.text-input {
		padding: 6px 10px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-bg);
		color: var(--color-text);
		font-size: 13px;
	}

	.char-count {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
	}

	.char-count.over {
		color: var(--color-danger);
		font-weight: 600;
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 60px 20px;
		color: var(--color-text-muted);
		font-size: 13px;
	}

	.loading-state :global(.spinner) {
		animation: spin 1s linear infinite;
	}

	:global(.spinner) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
