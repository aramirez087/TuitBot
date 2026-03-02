<script lang="ts">
	import type { ScheduleConfig } from '$lib/api';
	import TimePicker from '../TimePicker.svelte';
	import VoiceContextPanel from './VoiceContextPanel.svelte';
	import FromNotesPanel from '../FromNotesPanel.svelte';

	let {
		schedule,
		selectedTime,
		targetDate,
		voiceCue,
		assisting,
		hasExistingContent,
		showFromNotes,
		showUndo,
		mode,
		onselect,
		oncuechange,
		onaiassist,
		onopenotes,
		ongenerate,
		onclosenotes,
		onundo,
		voicePanelRef = $bindable()
	}: {
		schedule: ScheduleConfig | null;
		selectedTime: string | null;
		targetDate: Date;
		voiceCue: string;
		assisting: boolean;
		hasExistingContent: boolean;
		showFromNotes: boolean;
		showUndo: boolean;
		mode: 'tweet' | 'thread';
		onselect: (time: string | null) => void;
		oncuechange: (cue: string) => void;
		onaiassist: () => void;
		onopenotes: () => void;
		ongenerate: (notes: string) => Promise<void>;
		onclosenotes: () => void;
		onundo: () => void;
		voicePanelRef?: VoiceContextPanel;
	} = $props();
</script>

<div class="inspector-section">
	<div class="inspector-section-label">Schedule</div>
	<TimePicker
		{schedule} {selectedTime} {targetDate}
		onselect={(time) => onselect(time || null)}
	/>
	{#if !selectedTime}
		<p class="inspector-hint">Posts immediately unless scheduled</p>
	{/if}
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
		<kbd class="inspector-kbd">{'\u2318'}J</kbd>
	</div>
	<p class="inspector-subtitle">Improve selected text or generate new content</p>
	<div class="ai-actions-row">
		<button class="ai-action-btn" onclick={onaiassist} disabled={assisting}>
			{assisting ? 'Working...' : hasExistingContent ? 'Improve with AI' : 'AI generate'}
		</button>
		<button class="ai-action-btn secondary" onclick={onopenotes}>
			From notes
		</button>
	</div>
</div>

{#if showFromNotes}
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

	.inspector-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
		margin: 6px 0 0;
	}

	.ai-actions-row {
		display: flex;
		gap: 6px;
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

	@media (prefers-reduced-motion: reduce) {
		.ai-action-btn {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.ai-action-btn {
			min-height: 44px;
		}
	}
</style>
