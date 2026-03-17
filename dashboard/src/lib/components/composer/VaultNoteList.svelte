<script lang="ts">
	import { ChevronRight, ChevronDown } from 'lucide-svelte';
	import type { VaultNoteItem, VaultNoteDetail } from '$lib/api/types';

	interface Props {
		notes: VaultNoteItem[];
		loading: boolean;
		expandedNodeId: number | null;
		expandedNote: VaultNoteDetail | null;
		expanding: boolean;
		selectedChunks: Map<number, { nodeId: number; heading: string }>;
		atLimit: boolean;
		searchQuery: string;
		onToggleNote: (nodeId: number) => void;
		onToggleChunk: (chunkId: number, nodeId: number, heading: string) => void;
	}

	const {
		notes,
		loading,
		expandedNodeId,
		expandedNote,
		expanding,
		selectedChunks,
		atLimit,
		searchQuery,
		onToggleNote,
		onToggleChunk,
	}: Props = $props();

	function noteTitle(note: VaultNoteItem): string {
		return (
			note.title ||
			note.relative_path.split('/').pop()?.replace(/\.md$/, '') ||
			'Untitled'
		);
	}

	function handleNoteKeydown(e: KeyboardEvent, nodeId: number) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onToggleNote(nodeId);
		}
	}
</script>

<div class="vault-note-list" role="list" aria-label="Vault notes">
	{#if loading}
		<div class="vault-loading">
			<div class="vault-loading-shimmer"></div>
		</div>
	{:else if notes.length === 0}
		<div class="vault-no-results">
			{searchQuery ? 'No notes match your search.' : 'No notes in vault yet.'}
		</div>
	{:else}
		{#each notes as note (note.node_id)}
			<div class="vault-note-item" role="listitem">
				<button
					class="vault-note-row"
					class:expanded={expandedNodeId === note.node_id}
					onclick={() => onToggleNote(note.node_id)}
					onkeydown={(e) => handleNoteKeydown(e, note.node_id)}
					aria-expanded={expandedNodeId === note.node_id}
				>
					{#if expandedNodeId === note.node_id}
						<ChevronDown size={12} />
					{:else}
						<ChevronRight size={12} />
					{/if}
					<span class="vault-note-title">{noteTitle(note)}</span>
					{#if note.chunk_count > 0}
						<span class="vault-chunk-badge">{note.chunk_count}</span>
					{/if}
				</button>

				{#if expandedNodeId === note.node_id && expandedNote}
					<div class="vault-chunks" role="group" aria-label="Note sections">
						{#if expanding}
							<div class="vault-chunk-loading">Loading...</div>
						{:else if expandedNote.chunks.length === 0}
							<div class="vault-chunk-empty">No indexed sections.</div>
						{:else}
							{#each expandedNote.chunks as chunk (chunk.chunk_id)}
								{@const isSelected = selectedChunks.has(chunk.chunk_id)}
								{@const isDisabled = atLimit && !isSelected}
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
											onToggleChunk(chunk.chunk_id, note.node_id, chunk.heading_path)}
										class="vault-chunk-cb"
										aria-label={chunk.heading_path}
									/>
									<div class="vault-chunk-info">
										<span class="vault-chunk-heading">{chunk.heading_path}</span>
										<span class="vault-chunk-snippet">{chunk.snippet}</span>
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

<style>
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
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
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
		background: color-mix(in srgb, var(--color-text-subtle) 15%, transparent);
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

	@media (pointer: coarse) {
		.vault-note-row {
			min-height: 44px;
			padding: 10px;
		}

		.vault-chunk-row {
			padding: 8px 6px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.vault-note-row,
		.vault-chunk-row {
			transition: none;
		}
	}
</style>
