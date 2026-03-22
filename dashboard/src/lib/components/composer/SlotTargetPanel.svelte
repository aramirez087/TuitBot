<script lang="ts">
	import type { ThreadBlock, NeighborItem, DraftInsert } from '$lib/api/types';
	import { getSlotLabel, getInsertsForBlock } from '$lib/stores/draftInsertStore';
	import type { DraftInsertState } from '$lib/api/types';
	import { Link, Undo2, ChevronDown } from 'lucide-svelte';

	let {
		threadBlocks = [],
		mode = 'tweet',
		acceptedNeighbors,
		insertState,
		oninsert,
		onundoinsert,
	}: {
		threadBlocks?: ThreadBlock[];
		mode?: 'tweet' | 'thread';
		acceptedNeighbors: Map<number, { neighbor: NeighborItem; role: string }>;
		insertState: DraftInsertState;
		oninsert?: (neighbor: NeighborItem, slotIndex: number, slotLabel: string) => void;
		onundoinsert?: (insertId: string) => void;
	} = $props();

	// Build slot options from thread blocks (or single tweet)
	const slots = $derived.by(() => {
		if (mode === 'tweet') {
			return [{ blockId: 'tweet', index: 0, label: getSlotLabel(0, 1), preview: '' }];
		}
		return threadBlocks.map((b, i) => ({
			blockId: b.id,
			index: i,
			label: getSlotLabel(i, threadBlocks.length),
			preview: b.text.slice(0, 60) + (b.text.length > 60 ? '...' : ''),
		}));
	});

	// Track which slot each neighbor is targeting via dropdown
	let slotSelections = $state<Map<number, number>>(new Map());

	function getSelectedSlot(nodeId: number): number {
		return slotSelections.get(nodeId) ?? 0;
	}

	function handleSlotChange(nodeId: number, slotIndex: number) {
		const next = new Map(slotSelections);
		next.set(nodeId, slotIndex);
		slotSelections = next;
	}

	function handleApply(neighbor: NeighborItem) {
		const slotIdx = getSelectedSlot(neighbor.node_id);
		const slot = slots[slotIdx];
		if (slot) {
			oninsert?.(neighbor, slotIdx, slot.label);
		}
	}

	// Get applied inserts for a given block
	function getAppliedForBlock(blockId: string): DraftInsert[] {
		return getInsertsForBlock(insertState, blockId);
	}

	// Check if a neighbor already has an insert applied
	function neighborHasInsert(nodeId: number): boolean {
		for (const [, inserts] of insertState.blockInserts) {
			if (inserts.some((i) => i.sourceNodeId === nodeId)) return true;
		}
		return false;
	}

	const unappliedNeighbors = $derived(
		[...acceptedNeighbors.entries()]
			.filter(([nodeId]) => !neighborHasInsert(nodeId))
			.map(([, v]) => v)
	);

	const appliedInserts = $derived(insertState.history);
</script>

<div class="slot-target-panel">
	{#if appliedInserts.length > 0}
		<div class="applied-section">
			<div class="section-label">Applied suggestions</div>
			{#each appliedInserts as insert (insert.id)}
				<div class="applied-item">
					<div class="applied-info">
						<Link size={10} />
						<span class="applied-title">{insert.sourceTitle}</span>
						<span class="applied-slot">&rarr; {insert.slotLabel}</span>
					</div>
					<button
						class="undo-btn"
						onclick={() => onundoinsert?.(insert.id)}
						aria-label="Undo insert from {insert.sourceTitle}"
						title="Undo"
					>
						<Undo2 size={10} />
					</button>
				</div>
			{/each}
		</div>
	{/if}

	{#if unappliedNeighbors.length > 0}
		<div class="unapplied-section">
			<div class="section-label">Refine specific slots</div>
			{#each unappliedNeighbors as { neighbor, role }}
				<div class="neighbor-row">
					<div class="neighbor-info">
						<span class="neighbor-title">{neighbor.node_title}</span>
						<span class="neighbor-snippet">{neighbor.snippet.slice(0, 80)}{neighbor.snippet.length > 80 ? '...' : ''}</span>
					</div>
					<div class="neighbor-actions">
						{#if slots.length > 1}
							<div class="slot-select-wrap">
								<select
									class="slot-select"
									value={getSelectedSlot(neighbor.node_id)}
									onchange={(e) => handleSlotChange(neighbor.node_id, Number((e.currentTarget as HTMLSelectElement).value))}
									aria-label="Target slot for {neighbor.node_title}"
								>
									{#each slots as slot, i}
										<option value={i}>{slot.label}</option>
									{/each}
								</select>
								<ChevronDown size={10} class="select-arrow" />
							</div>
						{/if}
						<button
							class="apply-btn"
							onclick={() => handleApply(neighbor)}
							aria-label="Apply {neighbor.node_title} to slot"
						>
							Apply
						</button>
					</div>
				</div>
			{/each}
		</div>
	{:else if appliedInserts.length === 0}
		<div class="empty-state">
			Accept related notes above to refine specific parts of your draft.
		</div>
	{/if}
</div>

<style>
	.slot-target-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 6px;
	}

	.section-label {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.03em;
		color: var(--color-text-subtle);
		margin-bottom: 4px;
	}

	.applied-section {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.applied-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
		padding: 4px 6px;
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent);
	}

	.applied-info {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--color-text-muted);
		min-width: 0;
		overflow: hidden;
	}

	.applied-title {
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 100px;
	}

	.applied-slot {
		font-size: 10px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.undo-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		flex-shrink: 0;
		transition: all 0.1s ease;
	}

	.undo-btn:hover {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	.unapplied-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.neighbor-row {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 6px;
		border-radius: 4px;
		border: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
	}

	.neighbor-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.neighbor-title {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text);
	}

	.neighbor-snippet {
		font-size: 10px;
		color: var(--color-text-subtle);
		line-height: 1.3;
	}

	.neighbor-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.slot-select-wrap {
		position: relative;
		flex: 1;
	}

	.slot-select {
		width: 100%;
		padding: 3px 20px 3px 6px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: var(--color-surface);
		color: var(--color-text);
		font-size: 10px;
		font-family: var(--font-sans);
		cursor: pointer;
		appearance: none;
	}

	.slot-select:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.slot-select-wrap :global(.select-arrow) {
		position: absolute;
		right: 4px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--color-text-subtle);
		pointer-events: none;
	}

	.apply-btn {
		padding: 3px 10px;
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		background: var(--color-accent);
		color: #fff;
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		transition: background 0.1s ease;
	}

	.apply-btn:hover {
		background: var(--color-accent-hover);
	}

	.empty-state {
		font-size: 10px;
		color: var(--color-text-subtle);
		padding: 6px 2px;
		line-height: 1.4;
	}

	@media (prefers-reduced-motion: reduce) {
		.undo-btn,
		.apply-btn {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.apply-btn {
			min-height: 32px;
			padding: 6px 12px;
		}

		.undo-btn {
			width: 32px;
			height: 32px;
		}
	}
</style>
