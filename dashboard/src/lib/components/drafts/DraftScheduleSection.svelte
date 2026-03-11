<script lang="ts">
	import { Copy } from 'lucide-svelte';
	import type { DraftSummary } from '$lib/api/types';
	import SchedulePicker from '../SchedulePicker.svelte';
	import { buildScheduledFor, toAccountTzParts, nowInAccountTz } from '$lib/utils/timezone';
	import { trackFunnel } from '$lib/analytics/funnel';

	let {
		draftSummary,
		prefillSchedule = null,
		timezone = 'UTC',
		preferredTimes = [],
		onschedule,
		onunschedule,
		onreschedule,
		onduplicate
	}: {
		draftSummary: DraftSummary;
		prefillSchedule?: string | null;
		timezone?: string;
		preferredTimes?: string[];
		onschedule: (scheduledFor: string) => void;
		onunschedule: () => void;
		onreschedule: (scheduledFor: string) => void;
		onduplicate: () => void;
	} = $props();

	let pickerDate = $state<string | null>(null);
	let announcement = $state('');
	let pickerTime = $state<string | null>(null);

	$effect(() => {
		if (draftSummary.status === 'scheduled' && draftSummary.scheduled_for) {
			const parts = toAccountTzParts(draftSummary.scheduled_for, timezone);
			pickerDate = parts.date;
			pickerTime = parts.time;
		} else if (prefillSchedule && draftSummary.status === 'draft') {
			const parts = toAccountTzParts(prefillSchedule, timezone);
			pickerDate = parts.date;
			pickerTime = parts.time;
		} else if (draftSummary.status === 'draft') {
			const now = nowInAccountTz(timezone);
			pickerDate = now.date;
			pickerTime = null;
		}
	});

	function handleSchedule(date: string, time: string) {
		const utcIso = buildScheduledFor(date, time, timezone);
		if (draftSummary.status === 'scheduled') {
			trackFunnel('schedule:reschedule', { source: 'draft-studio', timezone });
			onreschedule(utcIso);
			announcement = 'Schedule updated';
		} else {
			trackFunnel('schedule:created', { mode: 'draft-studio', timezone });
			onschedule(utcIso);
			announcement = 'Draft scheduled for ' + time;
		}
		pickerDate = date;
		pickerTime = time;
	}

	function handleUnschedule() {
		trackFunnel('schedule:unschedule', { source: 'draft-studio' });
		onunschedule();
		pickerTime = null;
		announcement = 'Schedule removed';
	}
</script>

<div class="schedule-section" aria-label="Schedule section">
	<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{announcement}</div>
	{#if draftSummary.status === 'posted'}
		<div class="field-label schedule-label">
			<Copy size={12} />
			Post Actions
		</div>
		<button class="action-btn" type="button" onclick={onduplicate}>
			Duplicate as draft
		</button>
	{:else}
		<SchedulePicker
			{timezone}
			{preferredTimes}
			context="draft-studio"
			selectedDate={pickerDate}
			selectedTime={pickerTime}
			scheduledFor={draftSummary.scheduled_for}
			status={draftSummary.status === 'scheduled' ? 'scheduled' : 'draft'}
			onschedule={handleSchedule}
			onunschedule={handleUnschedule}
		/>
	{/if}
</div>

<style>
	.schedule-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.field-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.schedule-label {
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.action-btn {
		flex: 1;
		padding: 5px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.action-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}
</style>
