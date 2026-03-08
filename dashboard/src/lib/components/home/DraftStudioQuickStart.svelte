<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { PenLine, Clock, ArrowRight, FileText } from 'lucide-svelte';
	import { api } from '$lib/api';
	import type { DraftSummary } from '$lib/api/types';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

	let recentDrafts = $state<DraftSummary[]>([]);
	let loading = $state(true);
	let creating = $state(false);

	async function loadRecent() {
		loading = true;
		try {
			const all = await api.draftStudio.list({ status: 'draft' });
			recentDrafts = all.slice(0, 5);
		} catch {
			recentDrafts = [];
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadRecent();
		const handler = () => loadRecent();
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});

	async function handleNewDraft() {
		if (creating) return;
		creating = true;
		try {
			const result = await api.draftStudio.create({ content_type: 'tweet' });
			console.info('[draft-studio]', { event: 'draft_created', id: result.id, source: 'home-quickstart' });
			goto(`/drafts?id=${result.id}`);
		} catch {
			goto('/drafts?new=true');
		} finally {
			creating = false;
		}
	}

	function handleResume(id: number) {
		console.info('[draft-studio]', { event: 'draft_selected', id, source: 'home-resume' });
		goto(`/drafts?id=${id}`);
	}

	function relativeTime(iso: string): string {
		const diff = Date.now() - new Date(iso).getTime();
		const mins = Math.floor(diff / 60000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hrs = Math.floor(mins / 60);
		if (hrs < 24) return `${hrs}h ago`;
		const days = Math.floor(hrs / 24);
		return `${days}d ago`;
	}
</script>

<div class="quickstart">
	<div class="quickstart-header">
		<div class="header-icon">
			<PenLine size={24} />
		</div>
		<div class="header-text">
			<h2>Draft Studio</h2>
			<p>Your writing workspace — drafts, scheduling, and revision history in one place.</p>
		</div>
	</div>

	<div class="actions">
		<button class="action-primary" onclick={handleNewDraft} disabled={creating}>
			<PenLine size={16} />
			{creating ? 'Creating...' : 'New Draft'}
		</button>
		<a href="/drafts" class="action-secondary">
			Open Draft Studio
			<ArrowRight size={14} />
		</a>
	</div>

	{#if loading}
		<div class="recent-skeleton">
			{#each [1, 2, 3] as _}
				<div class="skeleton-row"></div>
			{/each}
		</div>
	{:else if recentDrafts.length > 0}
		<div class="recent-drafts">
			<h3 class="recent-heading">Recent drafts</h3>
			{#each recentDrafts as draft (draft.id)}
				<button class="draft-row" onclick={() => handleResume(draft.id)}>
					<FileText size={14} class="draft-icon" />
					<span class="draft-title">
						{draft.title || draft.content_preview || 'Untitled draft'}
					</span>
					<span class="draft-meta">
						<Clock size={11} />
						{relativeTime(draft.updated_at)}
					</span>
				</button>
			{/each}
		</div>
	{:else}
		<div class="empty-hint">
			<p>No drafts yet. Create one to get started.</p>
		</div>
	{/if}
</div>

<style>
	.quickstart {
		max-width: 560px;
		margin: 60px auto 0;
	}

	.quickstart-header {
		display: flex;
		gap: 16px;
		align-items: flex-start;
		margin-bottom: 28px;
	}

	.header-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		flex-shrink: 0;
	}

	.header-text h2 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.header-text p {
		font-size: 14px;
		color: var(--color-text-muted);
		margin: 0;
		line-height: 1.5;
	}

	.actions {
		display: flex;
		gap: 10px;
		margin-bottom: 32px;
	}

	.action-primary {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 20px;
		border: none;
		border-radius: 8px;
		background: var(--color-accent);
		color: #fff;
		font-size: 14px;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s ease, transform 0.1s ease;
	}

	.action-primary:hover:not(:disabled) {
		background: var(--color-accent-hover);
		transform: translateY(-1px);
	}

	.action-primary:disabled {
		opacity: 0.7;
		cursor: wait;
	}

	.action-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: transparent;
		color: var(--color-text);
		font-size: 14px;
		font-weight: 500;
		text-decoration: none;
		transition: border-color 0.15s, background 0.15s;
	}

	.action-secondary:hover {
		border-color: var(--color-border);
		background: var(--color-surface-hover);
	}

	.recent-drafts {
		border: 1px solid var(--color-border-subtle);
		border-radius: 10px;
		overflow: hidden;
		background: var(--color-surface);
	}

	.recent-heading {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		padding: 12px 16px 8px;
		margin: 0;
	}

	.draft-row {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 16px;
		border: none;
		border-top: 1px solid var(--color-border-subtle);
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		text-align: left;
		cursor: pointer;
		transition: background 0.12s;
	}

	.draft-row:hover {
		background: var(--color-surface-hover);
	}

	.draft-row :global(.draft-icon) {
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	.draft-title {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 500;
	}

	.draft-meta {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--color-text-subtle);
		flex-shrink: 0;
		font-family: var(--font-mono);
	}

	.empty-hint {
		text-align: center;
		padding: 24px;
		color: var(--color-text-subtle);
		font-size: 13px;
	}

	.empty-hint p {
		margin: 0;
	}

	.recent-skeleton {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.skeleton-row {
		height: 40px;
		background: var(--color-surface);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}
</style>
