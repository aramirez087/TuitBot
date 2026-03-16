<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api';
	import type { VaultNoteItem, VaultNoteDetail } from '$lib/api/types';
	import { X, Search, FileText } from 'lucide-svelte';
	import VaultNoteList from './VaultNoteList.svelte';
	import VaultFooter from './VaultFooter.svelte';

	let {
		mode,
		hasExistingContent = false,
		ongenerate,
		onclose,
		onundo,
		showUndo = false,
	}: {
		mode: 'tweet' | 'thread';
		hasExistingContent?: boolean;
		ongenerate: (selectedNodeIds: number[]) => Promise<void>;
		onclose: () => void;
		onundo?: () => void;
		showUndo?: boolean;
	} = $props();

	const MAX_SELECTIONS = 3;

	let searchQuery = $state('');
	let notes = $state<VaultNoteItem[]>([]);
	let expandedNote = $state<VaultNoteDetail | null>(null);
	let expandedNodeId = $state<number | null>(null);
	let selectedChunks = $state<Map<number, { nodeId: number; heading: string }>>(new Map());
	let loading = $state(false);
	let expanding = $state(false);
	let generating = $state(false);
	let error = $state<string | null>(null);
	let confirmReplace = $state(false);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let searchRef: HTMLInputElement | undefined = $state();
	let noSources = $state(false);

	const selectionCount = $derived(selectedChunks.size);
	const atLimit = $derived(selectionCount >= MAX_SELECTIONS);

	async function searchNotes(q: string) {
		loading = true;
		error = null;
		try {
			const result = await api.vault.searchNotes({ q: q || undefined, limit: 20 });
			notes = result.notes;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Search failed';
		} finally {
			loading = false;
		}
	}

	function handleSearchInput() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => searchNotes(searchQuery), 300);
	}

	async function toggleNote(nodeId: number) {
		if (expandedNodeId === nodeId) {
			expandedNodeId = null;
			expandedNote = null;
			return;
		}
		expanding = true;
		try {
			const detail = await api.vault.noteDetail(nodeId);
			expandedNote = detail;
			expandedNodeId = nodeId;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load note';
		} finally {
			expanding = false;
		}
	}

	function toggleChunk(chunkId: number, nodeId: number, heading: string) {
		const next = new Map(selectedChunks);
		if (next.has(chunkId)) {
			next.delete(chunkId);
		} else if (!atLimit) {
			next.set(chunkId, { nodeId, heading });
		}
		selectedChunks = next;
	}

	async function handleGenerate() {
		if (selectionCount === 0 || generating) return;
		if (hasExistingContent && !confirmReplace) {
			confirmReplace = true;
			return;
		}
		generating = true;
		error = null;
		confirmReplace = false;
		try {
			const nodeIds = [...new Set([...selectedChunks.values()].map((v) => v.nodeId))];
			await ongenerate(nodeIds);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Generation failed';
		} finally {
			generating = false;
		}
	}

	function cancelReplace() {
		confirmReplace = false;
	}

	onMount(async () => {
		searchRef?.focus();
		try {
			const sourcesResult = await api.vault.sources();
			if (!sourcesResult.sources || sourcesResult.sources.length === 0) {
				noSources = true;
				return;
			}
		} catch {
			// sources endpoint failed — still try loading notes
		}
		await searchNotes('');
	});
</script>

<div class="vault-panel">
	<div class="vault-header">
		<span class="vault-label">From Vault</span>
		<button class="vault-close" onclick={onclose} aria-label="Close vault panel">
			<X size={12} />
		</button>
	</div>

	{#if noSources}
		<div class="vault-empty-state">
			<FileText size={20} />
			<p>No vault sources configured.</p>
			<p class="vault-empty-hint">
				Add content sources in Settings to use vault search.
			</p>
		</div>
	{:else}
		<div class="vault-search-wrap">
			<Search size={13} class="vault-search-icon" />
			<input
				bind:this={searchRef}
				bind:value={searchQuery}
				oninput={handleSearchInput}
				class="vault-search-input"
				type="text"
				placeholder="Search notes by title..."
				aria-label="Search vault notes"
			/>
		</div>

		{#if error}
			<div class="vault-error" role="alert">{error}</div>
		{/if}

		<VaultNoteList
			{notes}
			{loading}
			{expandedNodeId}
			{expandedNote}
			{expanding}
			{selectedChunks}
			{atLimit}
			{searchQuery}
			onToggleNote={toggleNote}
			onToggleChunk={toggleChunk}
		/>

		<VaultFooter
			{selectionCount}
			maxSelections={MAX_SELECTIONS}
			{mode}
			{generating}
			{confirmReplace}
			{showUndo}
			{onundo}
			onGenerate={handleGenerate}
			onCancelReplace={cancelReplace}
		/>
	{/if}
</div>

<style>
	.vault-panel {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.vault-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.vault-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.vault-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.vault-close:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.vault-search-wrap {
		position: relative;
		margin-bottom: 6px;
	}

	.vault-search-wrap :global(.vault-search-icon) {
		position: absolute;
		left: 8px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--color-text-subtle);
		pointer-events: none;
	}

	.vault-search-input {
		width: 100%;
		padding: 6px 8px 6px 28px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.vault-search-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.vault-search-input::placeholder {
		color: var(--color-text-subtle);
	}

	.vault-error {
		font-size: 12px;
		color: var(--color-danger);
		margin-bottom: 4px;
	}

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

	@media (max-width: 640px) {
		.vault-search-input {
			font-size: 16px;
		}
	}

	@media (pointer: coarse) {
		.vault-close {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.vault-close,
		.vault-search-input {
			transition: none;
		}
	}
</style>
