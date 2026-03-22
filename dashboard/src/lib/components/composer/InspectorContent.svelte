<script lang="ts">
	import type { ScheduleConfig, ThreadBlock } from '$lib/api';
	import type { NeighborItem, DraftInsertState } from '$lib/api/types';
	import SchedulePicker from '../SchedulePicker.svelte';
	import VoiceContextPanel from './VoiceContextPanel.svelte';
	import FromNotesPanel from '../FromNotesPanel.svelte';
	import FromVaultPanel from './FromVaultPanel.svelte';

	let {
		schedule,
		selectedTime,
		scheduledDate = null,
		targetDate,
		timezone = 'UTC',
		voiceCue,
		assisting,
		hasExistingContent,
		notesPanelMode,
		showUndo,
		mode,
		selectionSessionId = null,
		threadBlocks = [],
		insertState,
		onscheduleselect,
		onunschedule,
		oncuechange,
		onaiassist,
		onopenotes,
		onopenvault,
		ongenerate,
		ongeneratefromvault,
		onclosenotes,
		onundo,
		onSelectionConsumed,
		onslotinsert,
		onundoinsert,
		voicePanelRef = $bindable()
	}: {
		schedule: ScheduleConfig | null;
		selectedTime: string | null;
		scheduledDate?: string | null;
		targetDate: Date;
		timezone?: string;
		voiceCue: string;
		assisting: boolean;
		hasExistingContent: boolean;
		notesPanelMode: 'notes' | 'vault' | null;
		showUndo: boolean;
		mode: 'tweet' | 'thread';
		selectionSessionId?: string | null;
		threadBlocks?: ThreadBlock[];
		insertState?: DraftInsertState;
		onscheduleselect: (date: string, time: string) => void;
		onunschedule: () => void;
		oncuechange: (cue: string) => void;
		onaiassist: () => void;
		onopenotes: () => void;
		onopenvault: () => void;
		ongenerate: (notes: string) => Promise<void>;
		ongeneratefromvault: (selectedNodeIds: number[], outputFormat: 'tweet' | 'thread', highlights?: string[], hookStyle?: string, neighborProvenance?: Array<{ node_id: number; edge_type?: string; edge_label?: string }>) => Promise<void>;
		onclosenotes: () => void;
		onundo: () => void;
		onSelectionConsumed?: () => void;
		onslotinsert?: (neighbor: NeighborItem, slotIndex: number, slotLabel: string) => void;
		onundoinsert?: (insertId: string) => void;
		voicePanelRef?: VoiceContextPanel;
	} = $props();
</script>

<div class="inspector-section">
	<div class="inspector-section-label">Schedule</div>
	<SchedulePicker
		{timezone}
		preferredTimes={schedule?.preferred_times ?? []}
		selectedDate={scheduledDate}
		{selectedTime}
		status="draft"
		compact={true}
		onschedule={(date, time) => onscheduleselect(date, time)}
		onunschedule={() => onunschedule()}
	/>
</div>

<div class="inspector-section">
	<div class="inspector-section-label">Voice</div>
	<VoiceContextPanel
		bind:this={voicePanelRef}
		cue={voiceCue}
		oncuechange={(c) => { oncuechange(c); }}
		inline={true}
	/>
</div>

<div class="inspector-section">
	<div class="inspector-section-label">
		<span>AI</span>
		<kbd class="inspector-kbd">{'\u2318\u21e7'}J</kbd>
	</div>
	<p class="inspector-subtitle">Improve selected text or generate new content</p>
	<div class="ai-actions-row">
		<button class="ai-action-btn" onclick={onaiassist} disabled={assisting}>
			{assisting ? 'Working...' : hasExistingContent ? 'Improve with AI' : 'AI generate'}
		</button>
		<button
			class="ai-action-btn secondary"
			class:active={notesPanelMode === 'notes'}
			onclick={onopenotes}
		>
			From notes
		</button>
		<button
			class="ai-action-btn secondary"
			class:active={notesPanelMode === 'vault'}
			onclick={onopenvault}
		>
			From vault{#if selectionSessionId}<span class="selection-dot" title="Selection pending"></span>{/if}
		</button>
	</div>
</div>

{#if notesPanelMode === 'notes'}
	<div class="inspector-section">
		<FromNotesPanel
			{mode}
			{hasExistingContent}
			compact={true}
			ongenerate={ongenerate}
			onclose={onclosenotes}
			onundo={onundo}
			{showUndo}
		/>
	</div>
{:else if notesPanelMode === 'vault'}
	<div class="inspector-section">
		<FromVaultPanel
			{mode}
			{hasExistingContent}
			{selectionSessionId}
			{threadBlocks}
			{insertState}
			ongenerate={ongeneratefromvault}
			onclose={onclosenotes}
			onundo={onundo}
			{showUndo}
			{onSelectionConsumed}
			onslotinsert={onslotinsert}
			onundoinsert={onundoinsert}
		/>
	</div>
{/if}

<style>
	.inspector-section {
		padding: 14px 0;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.inspector-section:last-child {
		border-bottom: none;
	}

	.inspector-section-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-muted);
		margin-bottom: 8px;
	}

	.inspector-kbd {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		border-radius: 3px;
		background: var(--color-surface-hover);
		color: var(--color-text-subtle);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
		line-height: 1.6;
	}

	.inspector-subtitle {
		font-size: 11px;
		color: var(--color-text-subtle);
		margin: 0 0 8px;
		line-height: 1.4;
	}

	.ai-actions-row {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.ai-action-btn {
		flex: 1;
		padding: 6px 10px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
		white-space: nowrap;
	}

	.ai-action-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
		border-color: var(--color-accent-hover);
	}

	.ai-action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.ai-action-btn.secondary {
		background: transparent;
		color: var(--color-accent);
	}

	.ai-action-btn.secondary:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.ai-action-btn.secondary.active {
		background: color-mix(in srgb, var(--color-accent) 14%, transparent);
		border-color: var(--color-accent);
	}

	.selection-dot {
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-accent);
		margin-left: 4px;
		vertical-align: middle;
		animation: pulse-dot 1.5s ease-in-out infinite;
	}

	@keyframes pulse-dot {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}

	@media (prefers-reduced-motion: reduce) {
		.ai-action-btn {
			transition: none;
		}
		.selection-dot {
			animation: none;
		}
	}

	@media (pointer: coarse) {
		.ai-action-btn {
			min-height: 44px;
		}
	}
</style>
