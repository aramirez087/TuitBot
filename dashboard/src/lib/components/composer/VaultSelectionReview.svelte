<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { VaultSelectionResponse } from '$lib/api/types';
	import { Zap, FileText } from 'lucide-svelte';
	import VaultFooter from './VaultFooter.svelte';

	let {
		sessionId,
		outputFormat = $bindable('tweet'),
		hasExistingContent = false,
		showUndo = false,
		onundo,
		ongenerate,
		onSelectionConsumed,
		onexpired,
		onformatchange,
	}: {
		sessionId: string;
		outputFormat?: 'tweet' | 'thread';
		hasExistingContent?: boolean;
		showUndo?: boolean;
		onundo?: () => void;
		ongenerate: (nodeIds: number[], format: 'tweet' | 'thread', highlights?: string[]) => Promise<void>;
		onSelectionConsumed?: () => void;
		onexpired?: () => void;
		onformatchange?: (format: 'tweet' | 'thread') => void;
	} = $props();

	let selection = $state<VaultSelectionResponse | null>(null);
	let loading = $state(true);
	let expired = $state(false);
	let generating = $state(false);
	let confirmReplace = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			selection = await api.vault.getSelection(sessionId);
		} catch {
			expired = true;
		} finally {
			loading = false;
			onSelectionConsumed?.();
		}
	});

	async function handleGenerate() {
		if (!selection || generating) return;
		if (hasExistingContent && !confirmReplace) {
			confirmReplace = true;
			return;
		}
		generating = true;
		error = null;
		confirmReplace = false;
		try {
			const nodeIds = selection.resolved_node_id ? [selection.resolved_node_id] : [];
			const input = selection.selected_text ? [selection.selected_text] : undefined;
			await ongenerate(nodeIds, outputFormat, input);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Generation failed';
		} finally {
			generating = false;
		}
	}
</script>

{#if loading}
	<div class="vault-empty-state">
		<div class="vault-loading-shimmer"></div>
		<p>Loading selection...</p>
	</div>
{:else if expired}
	<div class="vault-empty-state">
		<FileText size={20} />
		<p>Selection expired.</p>
		<p class="vault-empty-hint">
			Select blocks manually or send a new selection from Obsidian.
		</p>
		<button class="vault-expired-dismiss" onclick={() => onexpired?.()}>Browse vault</button>
	</div>
{:else if selection}
	<div class="vault-selection-review">
		<div class="selection-source-meta">
			<Zap size={12} />
			<span class="selection-source-path">{selection.note_title || selection.file_path}</span>
		</div>
		{#if selection.heading_context}
			<div class="selection-heading">{selection.heading_context}</div>
		{/if}
		{#if selection.selected_text}
			<div class="selection-text-preview">{selection.selected_text}</div>
		{:else}
			<div class="selection-text-cloud-note">Text not shown in cloud mode for privacy.</div>
		{/if}
		{#if selection.frontmatter_tags && selection.frontmatter_tags.length > 0}
			<div class="selection-tags">
				{#each selection.frontmatter_tags as tag}
					<span class="selection-tag">#{tag}</span>
				{/each}
			</div>
		{/if}
	</div>
	{#if error}
		<div class="vault-error" role="alert">{error}</div>
	{/if}
	<VaultFooter
		selectionCount={1}
		maxSelections={1}
		{outputFormat}
		{generating}
		{confirmReplace}
		{showUndo}
		{onundo}
		onGenerate={handleGenerate}
		onCancelReplace={() => { confirmReplace = false; }}
		onformatchange={(f) => { outputFormat = f; onformatchange?.(f); }}
		selectionMode={true}
	/>
{/if}

<style>
	.vault-empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 20px 12px;
		color: var(--color-text-subtle);
		text-align: center;
	}

	.vault-empty-state p {
		margin: 0;
		font-size: 12px;
	}

	.vault-empty-hint {
		font-size: 11px !important;
		color: var(--color-text-subtle);
	}

	.vault-expired-dismiss {
		padding: 4px 12px;
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		background: transparent;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		margin-top: 4px;
	}

	.vault-expired-dismiss:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.vault-loading-shimmer {
		width: 100%;
		height: 40px;
		border-radius: 6px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-accent) 8%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	@keyframes shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	.vault-selection-review {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 10px;
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-accent) 4%, transparent);
	}

	.selection-source-meta {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
	}

	.selection-source-path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.selection-heading {
		font-size: 11px;
		color: var(--color-text-muted);
		font-weight: 500;
	}

	.selection-text-preview {
		font-size: 12px;
		color: var(--color-text);
		line-height: 1.5;
		max-height: 100px;
		overflow-y: auto;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.selection-text-cloud-note {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	.selection-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.selection-tag {
		font-size: 10px;
		padding: 1px 6px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-text-subtle) 10%, transparent);
		color: var(--color-text-subtle);
	}

	.vault-error {
		font-size: 12px;
		color: var(--color-danger);
		margin-top: 4px;
	}
</style>
