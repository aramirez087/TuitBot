<script lang="ts">
	import type { EvidenceResult } from '$lib/api/types';
	import { Pin, PinOff, X, ArrowDownToLine, ChevronDown } from 'lucide-svelte';

	let {
		result,
		pinned = false,
		hasExistingContent = false,
		suggested = false,
		slotOptions,
		onpin,
		onunpin,
		ondismiss,
		onapply,
		onapplyToSlot,
	}: {
		result: EvidenceResult;
		pinned?: boolean;
		hasExistingContent?: boolean;
		suggested?: boolean;
		slotOptions?: Array<{ index: number; label: string }>;
		onpin?: () => void;
		onunpin?: () => void;
		ondismiss?: () => void;
		onapply?: () => void;
		onapplyToSlot?: (slotIndex: number, slotLabel: string) => void;
	} = $props();

	let showSlotPicker = $state(false);

	const reasonBadge: Record<string, { label: string; className: string }> = {
		semantic: { label: 'Semantic', className: 'badge-semantic' },
		keyword: { label: 'Keyword', className: 'badge-keyword' },
		graph: { label: 'Graph', className: 'badge-graph' },
		hybrid: { label: 'Hybrid', className: 'badge-hybrid' },
	};

	const badge = $derived(reasonBadge[result.match_reason] ?? reasonBadge.semantic);
	const hasMultipleSlots = $derived(slotOptions != null && slotOptions.length > 1);

	function truncate(text: string, max: number = 120): string {
		if (text.length <= max) return text;
		return text.slice(0, max).trimEnd() + '\u2026';
	}

	function handleApplyClick() {
		if (hasMultipleSlots) {
			showSlotPicker = !showSlotPicker;
		} else {
			onapply?.();
		}
	}

	function handleSlotSelect(slotIndex: number, slotLabel: string) {
		showSlotPicker = false;
		onapplyToSlot?.(slotIndex, slotLabel);
	}

	function handleSlotPickerKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			showSlotPicker = false;
		}
	}
</script>

<svelte:window onclick={() => { if (showSlotPicker) showSlotPicker = false; }} />

<div class="evidence-card" class:pinned role="article" aria-label="Evidence: {result.heading_path}">
	<div class="card-header">
		<span class="match-badge {badge.className}">{badge.label}</span>
		{#if suggested}
			<span class="suggested-badge">Suggested</span>
		{/if}
		<span class="card-score" title="Relevance score">{(result.score * 100).toFixed(0)}%</span>
	</div>

	{#if result.heading_path}
		<div class="card-heading" title={result.heading_path}>{result.heading_path}</div>
	{/if}

	<div class="card-snippet">{truncate(result.snippet)}</div>

	{#if result.node_title}
		<div class="card-source" title={result.relative_path ?? result.node_title}>
			{result.node_title}
		</div>
	{/if}

	<div class="card-actions">
		{#if pinned}
			<button class="card-action-btn" onclick={onunpin} title="Unpin evidence" aria-label="Unpin">
				<PinOff size={13} />
			</button>
		{:else}
			<button class="card-action-btn" onclick={onpin} title="Pin evidence" aria-label="Pin">
				<Pin size={13} />
			</button>
		{/if}
		{#if hasExistingContent && (onapply || onapplyToSlot)}
			<div class="apply-wrapper">
				<button
					class="card-action-btn"
					onclick={(e) => { e.stopPropagation(); handleApplyClick(); }}
					title={hasMultipleSlots ? 'Apply to slot' : 'Apply to draft'}
					aria-label={hasMultipleSlots ? 'Apply to slot' : 'Apply to slot'}
					aria-haspopup={hasMultipleSlots ? 'true' : undefined}
					aria-expanded={hasMultipleSlots ? showSlotPicker : undefined}
				>
					<ArrowDownToLine size={13} />
					{#if hasMultipleSlots}
						<ChevronDown size={10} />
					{/if}
				</button>
				{#if showSlotPicker && slotOptions}
					<!-- svelte-ignore a11y_interactive_supports_focus -->
					<div
						class="slot-picker"
						role="menu"
						aria-label="Choose target slot"
						onkeydown={handleSlotPickerKeydown}
					>
						{#each slotOptions as slot (slot.index)}
							<button
								class="slot-option"
								role="menuitem"
								onclick={(e) => { e.stopPropagation(); handleSlotSelect(slot.index, slot.label); }}
							>
								{slot.label}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
		{#if !pinned && ondismiss}
			<button class="card-action-btn dismiss" onclick={ondismiss} title="Dismiss" aria-label="Dismiss">
				<X size={13} />
			</button>
		{/if}
	</div>
</div>

<style>
	.evidence-card {
		padding: 8px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-surface);
		transition: border-color 0.15s ease;
	}

	.evidence-card:hover {
		border-color: var(--color-border);
	}

	.evidence-card.pinned {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 4%, var(--color-surface));
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 4px;
	}

	.match-badge {
		font-size: 9px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding: 1px 5px;
		border-radius: 3px;
		line-height: 1.5;
	}

	.badge-semantic { background: color-mix(in srgb, #a855f7 16%, transparent); color: #a855f7; }
	.badge-keyword { background: var(--color-surface-hover); color: var(--color-text-muted); }
	.badge-graph { background: color-mix(in srgb, #3b82f6 16%, transparent); color: #3b82f6; }
	.badge-hybrid { background: linear-gradient(135deg, color-mix(in srgb, #a855f7 12%, transparent), color-mix(in srgb, #3b82f6 12%, transparent)); color: #8b5cf6; }

	.suggested-badge {
		font-size: 9px;
		font-weight: 500;
		color: var(--color-text-subtle);
		padding: 1px 4px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 3px;
		line-height: 1.5;
	}

	.card-score {
		margin-left: auto;
		font-size: 10px;
		color: var(--color-text-subtle);
		font-variant-numeric: tabular-nums;
	}

	.card-heading {
		font-size: 11px;
		color: var(--color-accent);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-bottom: 2px;
	}

	.card-snippet {
		font-size: 12px;
		color: var(--color-text);
		line-height: 1.4;
		margin-bottom: 4px;
	}

	.card-source {
		font-size: 10px;
		color: var(--color-text-subtle);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-bottom: 4px;
	}

	.card-actions {
		display: flex;
		gap: 4px;
	}

	.apply-wrapper {
		position: relative;
	}

	.card-action-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 2px;
		padding: 3px 6px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.1s ease;
	}

	.card-action-btn:hover {
		border-color: var(--color-border);
		color: var(--color-text);
		background: var(--color-surface-hover);
	}

	.card-action-btn.dismiss:hover {
		border-color: #ef4444;
		color: #ef4444;
	}

	.card-action-btn:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 1px;
	}

	.slot-picker {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		z-index: 20;
		min-width: 140px;
		padding: 4px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-surface);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.slot-option {
		display: block;
		width: 100%;
		padding: 5px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 11px;
		text-align: left;
		cursor: pointer;
		transition: background 0.1s ease;
	}

	.slot-option:hover {
		background: var(--color-surface-hover);
	}

	.slot-option:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: -1px;
	}

	@media (prefers-reduced-motion: reduce) {
		.evidence-card { transition: none; }
		.card-action-btn { transition: none; }
		.slot-option { transition: none; }
	}
</style>
