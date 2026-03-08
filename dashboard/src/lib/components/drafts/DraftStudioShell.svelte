<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';
	import * as studio from '$lib/stores/draftStudio.svelte';
	import DraftRail from './DraftRail.svelte';
	import DraftEmptyState from './DraftEmptyState.svelte';

	onMount(() => {
		studio.initFromUrl($page.url);
		studio.loadDrafts();

		const handler = () => {
			studio.reset();
			studio.loadDrafts();
		};
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});

	function handleCreate() {
		studio.createDraft();
	}

	function relativeTime(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}
</script>

<div class="studio-shell">
	<div class="rail-zone">
		<DraftRail
			drafts={studio.currentTabDrafts}
			selectedId={studio.getSelectedId()}
			tab={studio.getTab()}
			tabCounts={studio.tabCounts}
			loading={studio.isLoading()}
			onselect={(id) => studio.selectDraft(id)}
			ontabchange={(t) => studio.setTab(t)}
			oncreate={handleCreate}
		/>
	</div>

	<div class="composer-zone">
		{#if studio.getError()}
			<div class="error-banner">
				<span>{studio.getError()}</span>
				<button type="button" onclick={() => studio.clearError()}>Dismiss</button>
			</div>
		{/if}

		{#if studio.isLoading() && !studio.selectedDraft}
			<div class="zone-loading">
				<div class="zone-spinner"></div>
			</div>
		{:else if studio.selectedDraft}
			<div class="draft-preview">
				<div class="preview-header">
					<span class="preview-type">{studio.selectedDraft.content_type}</span>
					{#if studio.selectedDraft.status === 'scheduled'}
						<span class="preview-status scheduled">Scheduled</span>
					{/if}
				</div>
				<h2 class="preview-title">
					{studio.selectedDraft.title ?? 'Untitled draft'}
				</h2>
				<div class="preview-content">
					{#if studio.selectedDraft.content_preview?.trim()}
						<p>{studio.selectedDraft.content_preview}</p>
					{:else}
						<p class="preview-placeholder">Empty draft — start writing...</p>
					{/if}
				</div>
				<div class="preview-meta">
					<span>Created {relativeTime(studio.selectedDraft.created_at)}</span>
					<span>Updated {relativeTime(studio.selectedDraft.updated_at)}</span>
				</div>
			</div>
		{:else if studio.tabCounts.active === 0 && studio.tabCounts.scheduled === 0}
			<DraftEmptyState variant="no-drafts" oncreate={handleCreate} />
		{:else}
			<DraftEmptyState variant="no-selection" oncreate={handleCreate} />
		{/if}
	</div>
</div>

<style>
	.studio-shell {
		display: grid;
		grid-template-columns: 260px 1fr;
		height: calc(100vh - 48px);
		margin: -24px -32px;
		overflow: hidden;
	}

	.rail-zone {
		min-width: 0;
		overflow: hidden;
	}

	.composer-zone {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow-y: auto;
		background: var(--color-base);
	}

	.zone-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	.zone-spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border-subtle);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 13px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-danger) 20%, transparent);
	}

	.error-banner button {
		border: none;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		text-decoration: underline;
	}

	.draft-preview {
		max-width: 640px;
		margin: 48px auto;
		padding: 0 32px;
	}

	.preview-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 12px;
	}

	.preview-type {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		padding: 2px 8px;
		border-radius: 4px;
	}

	.preview-status.scheduled {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-warning, #d29922);
		background: color-mix(in srgb, var(--color-warning, #d29922) 10%, transparent);
		padding: 2px 8px;
		border-radius: 4px;
	}

	.preview-title {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 16px;
		line-height: 1.3;
	}

	.preview-content {
		font-size: 15px;
		color: var(--color-text);
		line-height: 1.7;
		white-space: pre-wrap;
	}

	.preview-content p {
		margin: 0;
	}

	.preview-placeholder {
		color: var(--color-text-subtle);
		font-style: italic;
	}

	.preview-meta {
		display: flex;
		gap: 16px;
		margin-top: 24px;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
		font-size: 12px;
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
