<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { VaultSelectionResponse, HookOption, NeighborItem, GraphState, ThreadBlock, DraftInsertState } from '$lib/api/types';
	import { Zap, FileText } from 'lucide-svelte';
	import VaultFooter from './VaultFooter.svelte';
	import HookPicker from './HookPicker.svelte';
	import GraphSuggestionCards from './GraphSuggestionCards.svelte';
	import SlotTargetPanel from './SlotTargetPanel.svelte';
	import { createInsertState } from '$lib/stores/draftInsertStore';

	let {
		sessionId,
		outputFormat = $bindable('tweet'),
		hasExistingContent = false,
		showUndo = false,
		threadBlocks = [],
		mode = 'tweet',
		insertState,
		onundo,
		ongenerate,
		onSelectionConsumed,
		onexpired,
		onformatchange,
		oninsert,
		onundoinsert,
	}: {
		sessionId: string;
		outputFormat?: 'tweet' | 'thread';
		hasExistingContent?: boolean;
		showUndo?: boolean;
		threadBlocks?: ThreadBlock[];
		mode?: 'tweet' | 'thread';
		insertState?: DraftInsertState;
		onundo?: () => void;
		ongenerate: (nodeIds: number[], format: 'tweet' | 'thread', highlights?: string[], hookStyle?: string, neighborProvenance?: Array<{ node_id: number; edge_type?: string; edge_label?: string }>) => Promise<void>;
		onSelectionConsumed?: () => void;
		onexpired?: () => void;
		onformatchange?: (format: 'tweet' | 'thread') => void;
		oninsert?: (neighbor: NeighborItem, slotIndex: number, slotLabel: string) => void;
		onundoinsert?: (insertId: string) => void;
	} = $props();

	const effectiveInsertState = $derived(insertState ?? createInsertState());

	let selection = $state<VaultSelectionResponse | null>(null);
	let loading = $state(true);
	let expired = $state(false);
	let generating = $state(false);
	let confirmReplace = $state(false);
	let error = $state<string | null>(null);
	let hookOptions = $state<HookOption[] | null>(null);
	let hookLoading = $state(false);
	let hookError = $state<string | null>(null);

	// Graph suggestion state (session-scoped)
	let acceptedNeighbors = $state<Map<number, { neighbor: NeighborItem; role: string }>>(new Map());
	let dismissedNodeIds = $state<Set<number>>(new Set());
	let synthesisEnabled = $state(true);

	const graphNeighbors = $derived(selection?.graph_neighbors ?? []);
	const graphState = $derived<GraphState>(selection?.graph_state ?? 'fallback_active');
	const graphLoading = $derived(loading);
	const visibleNeighbors = $derived(
		graphNeighbors.filter((n) => !dismissedNodeIds.has(n.node_id))
	);

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

	function handleAcceptNeighbor(neighbor: NeighborItem, role: string) {
		const next = new Map(acceptedNeighbors);
		next.set(neighbor.node_id, { neighbor, role });
		acceptedNeighbors = next;
	}

	function handleDismissNeighbor(nodeId: number) {
		const nextDismissed = new Set(dismissedNodeIds);
		nextDismissed.add(nodeId);
		dismissedNodeIds = nextDismissed;
		// Also remove from accepted if it was accepted
		if (acceptedNeighbors.has(nodeId)) {
			const next = new Map(acceptedNeighbors);
			next.delete(nodeId);
			acceptedNeighbors = next;
		}
	}

	function toggleSynthesis() {
		synthesisEnabled = !synthesisEnabled;
	}

	async function handleGenerate() {
		if (!selection || hookLoading) return;
		hookLoading = true;
		hookError = null;
		error = null;
		try {
			const topic = selection.selected_text || selection.note_title || selection.heading_context || 'general topic';
			const result = await api.assist.hooks(topic, { sessionId: sessionId });
			hookOptions = result.hooks;
		} catch (e) {
			hookError = e instanceof Error ? e.message : 'Failed to generate hooks';
		} finally {
			hookLoading = false;
		}
	}

	async function handleHookSelected(hook: HookOption, format: 'tweet' | 'thread') {
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
			// Build neighbor provenance for accepted neighbors
			const neighborProv: Array<{ node_id: number; edge_type?: string; edge_label?: string }> = [];
			if (synthesisEnabled && acceptedNeighbors.size > 0) {
				for (const [nid, { neighbor }] of acceptedNeighbors) {
					if (!nodeIds.includes(nid)) nodeIds.push(nid);
					neighborProv.push({
						node_id: nid,
						edge_type: neighbor.reason,
						edge_label: neighbor.reason_label,
					});
				}
			}
			await ongenerate(nodeIds, format, [hook.text], hook.style, neighborProv.length > 0 ? neighborProv : undefined);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Generation failed';
		} finally {
			generating = false;
		}
	}

	async function handleRegenerateHooks() {
		if (!selection) return;
		hookLoading = true;
		hookError = null;
		try {
			const topic = selection.selected_text || selection.note_title || selection.heading_context || 'general topic';
			const result = await api.assist.hooks(topic, { sessionId: sessionId });
			hookOptions = result.hooks;
		} catch (e) {
			hookError = e instanceof Error ? e.message : 'Failed to generate hooks';
		} finally {
			hookLoading = false;
		}
	}

	function handleBackFromHooks() {
		hookOptions = null;
		hookError = null;
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
{:else if selection && (hookOptions || hookLoading)}
	{#if error}
		<div class="vault-error" role="alert">{error}</div>
	{/if}
	<HookPicker
		hooks={hookOptions ?? []}
		{outputFormat}
		loading={hookLoading}
		error={hookError}
		onselect={handleHookSelected}
		onregenerate={handleRegenerateHooks}
		onback={handleBackFromHooks}
		onformatchange={(f) => { outputFormat = f; onformatchange?.(f); }}
	/>
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

	{#if graphState !== 'fallback_active'}
		<div class="graph-toggle-row">
			<button
				class="synthesis-toggle"
				class:active={synthesisEnabled}
				onclick={toggleSynthesis}
				aria-pressed={synthesisEnabled}
				aria-label="Toggle related notes"
			>
				Related notes {synthesisEnabled ? 'ON' : 'OFF'}
			</button>
		</div>
	{/if}

	{#if synthesisEnabled}
		<GraphSuggestionCards
			neighbors={visibleNeighbors}
			{graphState}
			loading={graphLoading}
			onaccept={handleAcceptNeighbor}
			ondismiss={handleDismissNeighbor}
		/>
	{/if}

	{#if acceptedNeighbors.size > 0 && synthesisEnabled}
		<div class="accepted-summary">
			{acceptedNeighbors.size} related {acceptedNeighbors.size === 1 ? 'note' : 'notes'} will be included
		</div>
	{/if}

	{#if hasExistingContent && acceptedNeighbors.size > 0 && synthesisEnabled}
		<SlotTargetPanel
			{threadBlocks}
			{mode}
			{acceptedNeighbors}
			insertState={effectiveInsertState}
			{oninsert}
			{onundoinsert}
		/>
	{/if}

	{#if error}
		<div class="vault-error" role="alert">{error}</div>
	{/if}
	<VaultFooter
		selectionCount={1}
		maxSelections={1}
		{outputFormat}
		generating={hookLoading}
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

	.graph-toggle-row {
		display: flex;
		align-items: center;
		margin-top: 6px;
	}

	.synthesis-toggle {
		padding: 2px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 10px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.12s ease;
		text-transform: uppercase;
		letter-spacing: 0.02em;
	}

	.synthesis-toggle.active {
		border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
	}

	.synthesis-toggle:hover {
		border-color: var(--color-accent);
	}

	.accepted-summary {
		font-size: 10px;
		color: var(--color-accent);
		font-weight: 500;
		margin-top: 4px;
		padding: 0 2px;
	}

	.vault-error {
		font-size: 12px;
		color: var(--color-danger);
		margin-top: 4px;
	}

	@media (prefers-reduced-motion: reduce) {
		.synthesis-toggle { transition: none; }
	}
</style>
