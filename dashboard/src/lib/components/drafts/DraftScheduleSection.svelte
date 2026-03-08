<script lang="ts">
	import { CalendarClock, Copy } from 'lucide-svelte';
	import type { DraftSummary } from '$lib/api/types';

	let {
		draftSummary,
		prefillSchedule = null,
		onschedule,
		onunschedule,
		onreschedule,
		onduplicate
	}: {
		draftSummary: DraftSummary;
		prefillSchedule?: string | null;
		onschedule: (scheduledFor: string) => void;
		onunschedule: () => void;
		onreschedule: (scheduledFor: string) => void;
		onduplicate: () => void;
	} = $props();

	let scheduleDate = $state('');
	let scheduleTime = $state('');
	let showReschedule = $state(false);
	let scheduling = $state(false);

	$effect(() => {
		if (draftSummary.status === 'scheduled' && draftSummary.scheduled_for) {
			const d = new Date(draftSummary.scheduled_for);
			scheduleDate = d.toISOString().slice(0, 10);
			scheduleTime = d.toTimeString().slice(0, 5);
			showReschedule = false;
		} else if (prefillSchedule && draftSummary.status === 'draft') {
			const d = new Date(prefillSchedule);
			if (!isNaN(d.getTime())) {
				scheduleDate = d.toISOString().slice(0, 10);
				scheduleTime = d.toTimeString().slice(0, 5);
			}
		} else {
			const tomorrow = new Date();
			tomorrow.setDate(tomorrow.getDate() + 1);
			tomorrow.setHours(tomorrow.getHours() + 1, 0, 0, 0);
			scheduleDate = tomorrow.toISOString().slice(0, 10);
			scheduleTime = tomorrow.toTimeString().slice(0, 5);
			showReschedule = false;
		}
	});

	function buildIso(): string {
		return new Date(`${scheduleDate}T${scheduleTime}`).toISOString();
	}

	async function handleScheduleSubmit() {
		if (!scheduleDate || !scheduleTime) return;
		scheduling = true;
		try {
			onschedule(buildIso());
		} finally {
			scheduling = false;
		}
	}

	async function handleRescheduleSubmit() {
		if (!scheduleDate || !scheduleTime) return;
		scheduling = true;
		try {
			onreschedule(buildIso());
			showReschedule = false;
		} finally {
			scheduling = false;
		}
	}

	function formatScheduledTime(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' });
	}

	function formatDate(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}
</script>

<div class="schedule-section">
	{#if draftSummary.status === 'draft'}
		<div class="field-label schedule-label">
			<CalendarClock size={12} />
			Schedule
		</div>
		<div class="schedule-inputs">
			<input type="date" bind:value={scheduleDate} class="schedule-input" />
			<input type="time" bind:value={scheduleTime} class="schedule-input" />
		</div>
		<button
			class="schedule-btn"
			type="button"
			onclick={handleScheduleSubmit}
			disabled={!scheduleDate || !scheduleTime || scheduling}
		>
			{scheduling ? 'Scheduling...' : 'Schedule'}
		</button>
	{:else if draftSummary.status === 'scheduled' && draftSummary.scheduled_for}
		<div class="field-label schedule-label">
			<CalendarClock size={12} />
			Scheduled
		</div>
		<div class="scheduled-info">
			{formatDate(draftSummary.scheduled_for)} at {formatScheduledTime(draftSummary.scheduled_for)}
		</div>
		{#if showReschedule}
			<div class="schedule-inputs">
				<input type="date" bind:value={scheduleDate} class="schedule-input" />
				<input type="time" bind:value={scheduleTime} class="schedule-input" />
			</div>
			<div class="schedule-actions-row">
				<button class="schedule-btn" type="button" onclick={handleRescheduleSubmit} disabled={scheduling}>
					{scheduling ? 'Saving...' : 'Save'}
				</button>
				<button class="action-btn-secondary" type="button" onclick={() => { showReschedule = false; }}>
					Cancel
				</button>
			</div>
		{:else}
			<div class="schedule-actions-row">
				<button class="action-btn" type="button" onclick={onunschedule}>Unschedule</button>
				<button class="action-btn-secondary" type="button" onclick={() => { showReschedule = true; }}>
					Reschedule
				</button>
			</div>
		{/if}
	{:else if draftSummary.status === 'posted'}
		<div class="field-label schedule-label">
			<Copy size={12} />
			Actions
		</div>
		<button class="action-btn" type="button" onclick={onduplicate}>
			Duplicate as draft
		</button>
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

	.schedule-inputs {
		display: flex;
		gap: 6px;
	}

	.schedule-input {
		flex: 1;
		padding: 5px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: var(--color-base);
		color: var(--color-text);
		font-size: 12px;
		outline: none;
		min-width: 0;
	}

	.schedule-input:focus {
		border-color: var(--color-accent);
	}

	.schedule-btn {
		width: 100%;
		padding: 6px 12px;
		border: none;
		border-radius: 5px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.12s ease;
	}

	.schedule-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.schedule-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.scheduled-info {
		font-size: 12px;
		color: var(--color-text);
		padding: 4px 0;
	}

	.schedule-actions-row {
		display: flex;
		gap: 6px;
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

	.action-btn-secondary {
		flex: 1;
		padding: 5px 10px;
		border: 1px solid transparent;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.action-btn-secondary:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}
</style>
