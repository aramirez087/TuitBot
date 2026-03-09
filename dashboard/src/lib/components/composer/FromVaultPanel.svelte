<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "$lib/api";
	import type { VaultNoteItem, VaultNoteDetail } from "$lib/api/types";
	import {
		X,
		Search,
		ChevronRight,
		ChevronDown,
		FileText,
	} from "lucide-svelte";

	let {
		mode,
		hasExistingContent = false,
		ongenerate,
		onclose,
		onundo,
		showUndo = false,
	}: {
		mode: "tweet" | "thread";
		hasExistingContent?: boolean;
		ongenerate: (selectedNodeIds: number[]) => Promise<void>;
		onclose: () => void;
		onundo?: () => void;
		showUndo?: boolean;
	} = $props();

	const MAX_SELECTIONS = 3;

	let searchQuery = $state("");
	let notes = $state<VaultNoteItem[]>([]);
	let expandedNote = $state<VaultNoteDetail | null>(null);
	let expandedNodeId = $state<number | null>(null);
	let selectedChunks = $state<
		Map<number, { nodeId: number; heading: string }>
	>(new Map());
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
			const result = await api.vault.searchNotes({
				q: q || undefined,
				limit: 20,
			});
			notes = result.notes;
		} catch (e) {
			error = e instanceof Error ? e.message : "Search failed";
		} finally {
			loading = false;
		}
	}

	function handleSearchInput() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			searchNotes(searchQuery);
		}, 300);
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
			error = e instanceof Error ? e.message : "Failed to load note";
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
			const nodeIds = [
				...new Set([...selectedChunks.values()].map((v) => v.nodeId)),
			];
			await ongenerate(nodeIds);
		} catch (e) {
			error = e instanceof Error ? e.message : "Generation failed";
		} finally {
			generating = false;
		}
	}

	function cancelReplace() {
		confirmReplace = false;
	}

	function noteTitle(note: VaultNoteItem): string {
		return (
			note.title ||
			note.relative_path.split("/").pop()?.replace(/\.md$/, "") ||
			"Untitled"
		);
	}

	function handleNoteKeydown(e: KeyboardEvent, nodeId: number) {
		if (e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			toggleNote(nodeId);
		}
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
		await searchNotes("");
	});
</script>

<div class="vault-panel">
	<div class="vault-header">
		<span class="vault-label">From Vault</span>
		<button
			class="vault-close"
			onclick={onclose}
			aria-label="Close vault panel"
		>
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

		<div class="vault-note-list" role="list" aria-label="Vault notes">
			{#if loading}
				<div class="vault-loading">
					<div class="vault-loading-shimmer"></div>
				</div>
			{:else if notes.length === 0}
				<div class="vault-no-results">
					{searchQuery
						? "No notes match your search."
						: "No notes in vault yet."}
				</div>
			{:else}
				{#each notes as note (note.node_id)}
					<div class="vault-note-item" role="listitem">
						<button
							class="vault-note-row"
							class:expanded={expandedNodeId === note.node_id}
							onclick={() => toggleNote(note.node_id)}
							onkeydown={(e) =>
								handleNoteKeydown(e, note.node_id)}
							aria-expanded={expandedNodeId === note.node_id}
						>
							{#if expandedNodeId === note.node_id}
								<ChevronDown size={12} />
							{:else}
								<ChevronRight size={12} />
							{/if}
							<span class="vault-note-title"
								>{noteTitle(note)}</span
							>
							{#if note.chunk_count > 0}
								<span class="vault-chunk-badge"
									>{note.chunk_count}</span
								>
							{/if}
						</button>

						{#if expandedNodeId === note.node_id && expandedNote}
							<div
								class="vault-chunks"
								role="group"
								aria-label="Note sections"
							>
								{#if expanding}
									<div class="vault-chunk-loading">
										Loading...
									</div>
								{:else if expandedNote.chunks.length === 0}
									<div class="vault-chunk-empty">
										No indexed sections.
									</div>
								{:else}
									{#each expandedNote.chunks as chunk (chunk.chunk_id)}
										{@const isSelected = selectedChunks.has(
											chunk.chunk_id,
										)}
										{@const isDisabled =
											atLimit && !isSelected}
										<label
											class="vault-chunk-row"
											class:selected={isSelected}
											class:disabled={isDisabled}
										>
											<input
												type="checkbox"
												checked={isSelected}
												disabled={isDisabled}
												onchange={() =>
													toggleChunk(
														chunk.chunk_id,
														note.node_id,
														chunk.heading_path,
													)}
												class="vault-chunk-cb"
											/>
											<div class="vault-chunk-info">
												<span
													class="vault-chunk-heading"
													>{chunk.heading_path}</span
												>
												<span
													class="vault-chunk-snippet"
													>{chunk.snippet}</span
												>
											</div>
										</label>
									{/each}
								{/if}
							</div>
						{/if}
					</div>
				{/each}
			{/if}
		</div>

		<div class="vault-footer">
			<span class="vault-selection-count">
				{selectionCount} of {MAX_SELECTIONS} selected
			</span>

			{#if confirmReplace}
				<div class="vault-replace-banner" role="alert">
					<span>This will replace your current content.</span>
					<div class="vault-replace-actions">
						<button
							class="vault-replace-confirm"
							onclick={handleGenerate}>Replace</button
						>
						<button
							class="vault-replace-cancel"
							onclick={cancelReplace}>Cancel</button
						>
					</div>
				</div>
			{:else}
				<button
					class="vault-generate-btn"
					onclick={handleGenerate}
					disabled={selectionCount === 0 || generating}
				>
					{generating
						? "Generating..."
						: mode === "thread"
							? "Generate thread from vault"
							: "Generate tweet from vault"}
				</button>
			{/if}

			{#if showUndo && onundo}
				<button class="vault-undo-btn" onclick={onundo}
					>Undo replacement</button
				>
			{/if}
		</div>
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

	.vault-note-list {
		max-height: 200px;
		overflow-y: auto;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-base);
	}

	.vault-loading {
		padding: 12px;
		position: relative;
		overflow: hidden;
		min-height: 40px;
	}

	.vault-loading-shimmer {
		position: absolute;
		inset: 0;
		background: linear-gradient(
			90deg,
			transparent 25%,
			color-mix(in srgb, var(--color-accent) 8%, transparent) 50%,
			transparent 75%
		);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	@keyframes shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}

	.vault-no-results {
		padding: 14px 12px;
		text-align: center;
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.vault-note-item {
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.vault-note-item:last-child {
		border-bottom: none;
	}

	.vault-note-row {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 8px 10px;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-sans);
		cursor: pointer;
		text-align: left;
		transition: background 0.1s ease;
	}

	.vault-note-row:hover {
		background: var(--color-surface-hover);
	}

	.vault-note-row.expanded {
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
	}

	.vault-note-title {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 500;
	}

	.vault-chunk-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 18px;
		height: 18px;
		padding: 0 4px;
		border-radius: 9px;
		background: color-mix(
			in srgb,
			var(--color-text-subtle) 15%,
			transparent
		);
		color: var(--color-text-subtle);
		font-size: 10px;
		font-weight: 600;
		flex-shrink: 0;
	}

	.vault-chunks {
		padding: 0 10px 8px 26px;
	}

	.vault-chunk-loading,
	.vault-chunk-empty {
		font-size: 11px;
		color: var(--color-text-subtle);
		padding: 4px 0;
	}

	.vault-chunk-row {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		padding: 4px 6px;
		border-radius: 4px;
		cursor: pointer;
		transition: background 0.1s ease;
	}

	.vault-chunk-row:hover:not(.disabled) {
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
	}

	.vault-chunk-row.selected {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.vault-chunk-row.disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.vault-chunk-cb {
		margin-top: 2px;
		accent-color: var(--color-accent);
		flex-shrink: 0;
	}

	.vault-chunk-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}

	.vault-chunk-heading {
		font-size: 11px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.vault-chunk-snippet {
		font-size: 10px;
		color: var(--color-text-subtle);
		font-style: italic;
		overflow: hidden;
		display: -webkit-box;
		display: box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		box-orient: vertical;
		line-height: 1.4;
	}

	.vault-footer {
		margin-top: 6px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.vault-selection-count {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.vault-generate-btn {
		padding: 6px 14px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.vault-generate-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.vault-generate-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.vault-replace-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		font-size: 12px;
		color: var(--color-warning);
	}

	.vault-replace-actions {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
	}

	.vault-replace-confirm {
		padding: 4px 10px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: var(--color-warning);
		color: #000;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
	}

	.vault-replace-cancel {
		padding: 4px 10px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: transparent;
		color: var(--color-warning);
		font-size: 11px;
		cursor: pointer;
	}

	.vault-undo-btn {
		padding: 4px 10px;
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		background: transparent;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.vault-undo-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
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

		.vault-note-row {
			min-height: 44px;
			padding: 10px;
		}

		.vault-chunk-row {
			padding: 8px 6px;
		}

		.vault-generate-btn {
			min-height: 44px;
			padding: 10px 14px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.vault-close,
		.vault-note-row,
		.vault-chunk-row,
		.vault-generate-btn,
		.vault-undo-btn,
		.vault-search-input {
			transition: none;
		}
	}
</style>
