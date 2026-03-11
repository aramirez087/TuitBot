<script lang="ts">
	import { Clock, X, Zap } from 'lucide-svelte';
	import { formatInAccountTz, toAccountTzParts, nowInAccountTz } from '$lib/utils/timezone';
	import { trackFunnel } from '$lib/analytics/funnel';

	let {
		timezone = 'UTC',
		preferredTimes = [],
		selectedDate = null,
		selectedTime = null,
		scheduledFor = null,
		status = 'draft',
		compact = false,
		context = 'composer',
		onschedule,
		onunschedule,
	}: {
		timezone?: string;
		preferredTimes?: string[];
		selectedDate?: string | null;
		selectedTime?: string | null;
		scheduledFor?: string | null;
		status?: 'draft' | 'scheduled' | 'posted';
		compact?: boolean;
		context?: 'composer' | 'draft-studio' | 'calendar';
		onschedule?: (date: string, time: string) => void;
		onunschedule?: () => void;
	} = $props();

	let customTime = $state('');
	let customDate = $state('');

	const tzLabel = $derived(() => {
		try {
			const fmt = new Intl.DateTimeFormat('en-US', {
				timeZone: timezone,
				timeZoneName: 'short',
			});
			const parts = fmt.formatToParts(new Date());
			const tzPart = parts.find((p) => p.type === 'timeZoneName');
			return tzPart?.value ?? timezone;
		} catch {
			return timezone;
		}
	});

	const tzFull = $derived(() => {
		return timezone.replace(/_/g, ' ').split('/').pop() ?? timezone;
	});

	const scheduledDisplay = $derived(() => {
		if (!scheduledFor) return null;
		return formatInAccountTz(scheduledFor, timezone, {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit',
			timeZoneName: 'short',
		});
	});

	const hasSelection = $derived(!!selectedDate && !!selectedTime);

	// Pre-populate custom inputs when we have a selection
	$effect(() => {
		if (selectedDate && !customDate) customDate = selectedDate;
		if (selectedTime && !customTime) customTime = selectedTime;
	});

	// Pre-populate from scheduledFor when in scheduled state
	$effect(() => {
		if (status === 'scheduled' && scheduledFor) {
			const parts = toAccountTzParts(scheduledFor, timezone);
			customDate = parts.date;
			customTime = parts.time;
		}
	});

	function selectPreferredTime(time: string) {
		const now = nowInAccountTz(timezone);
		const dateToUse = customDate || selectedDate || now.date;
		trackFunnel('schedule:time-selected', { context, source: 'preferred-time', timezone });
		onschedule?.(dateToUse, time);
	}

	function selectCustom() {
		if (customTime && customDate && /^\d{2}:\d{2}$/.test(customTime)) {
			trackFunnel('schedule:time-selected', { context, source: 'custom', timezone });
			onschedule?.(customDate, customTime);
		}
	}

	function handleNextFreeSlot() {
		const now = nowInAccountTz(timezone);
		const [nowH, nowM] = now.time.split(':').map(Number);
		const nowMinutes = nowH * 60 + nowM;

		// Find the next preferred time after now (today or tomorrow)
		for (const time of preferredTimes) {
			const [h, m] = time.split(':').map(Number);
			const mins = h * 60 + m;
			if (mins > nowMinutes + 15) {
				onschedule?.(now.date, time);
				return;
			}
		}

		// All today's slots passed — use first slot tomorrow
		if (preferredTimes.length > 0) {
			const tomorrow = new Date();
			tomorrow.setDate(tomorrow.getDate() + 1);
			const y = tomorrow.getFullYear();
			const mo = String(tomorrow.getMonth() + 1).padStart(2, '0');
			const d = String(tomorrow.getDate()).padStart(2, '0');
			onschedule?.(`${y}-${mo}-${d}`, preferredTimes[0]);
			return;
		}

		// No preferred times — schedule for tomorrow same time + 1 hour
		const tomorrow = new Date();
		tomorrow.setDate(tomorrow.getDate() + 1);
		const y = tomorrow.getFullYear();
		const mo = String(tomorrow.getMonth() + 1).padStart(2, '0');
		const d = String(tomorrow.getDate()).padStart(2, '0');
		const nextHour = String(Math.min(nowH + 1, 23)).padStart(2, '0');
		onschedule?.(`${y}-${mo}-${d}`, `${nextHour}:00`);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && customTime && customDate) {
			e.preventDefault();
			selectCustom();
		}
	}
</script>

<div class="schedule-picker" class:compact aria-label="Schedule picker">
	<div class="tz-badge" id="tz-info" title={timezone}>
		<Clock size={11} />
		<span>{tzFull()} ({tzLabel()})</span>
	</div>

	{#if status === 'scheduled' && scheduledFor}
		<div class="scheduled-state">
			<div class="scheduled-display">
				{scheduledDisplay()}
			</div>
			<div class="scheduled-actions">
				<button
					class="picker-action-btn"
					type="button"
					onclick={onunschedule}
					aria-label="Remove schedule"
				>
					<X size={12} />
					Unschedule
				</button>
			</div>
		</div>
	{/if}

	<div class="picker-row">
		<input
			type="date"
			class="picker-input date-input"
			bind:value={customDate}
			aria-label="Schedule date"
			aria-describedby="tz-info"
		/>
		<input
			type="time"
			class="picker-input time-input"
			bind:value={customTime}
			onkeydown={handleKeydown}
			aria-label="Schedule time"
			aria-describedby="tz-info"
		/>
		<button
			class="set-btn"
			type="button"
			onclick={selectCustom}
			disabled={!customTime || !customDate}
			aria-label="Set custom schedule"
		>
			Set
		</button>
	</div>

	{#if preferredTimes.length > 0}
		<div class="preferred-slots">
			{#each preferredTimes as time}
				<button
					class="slot-pill"
					class:active={selectedTime === time}
					type="button"
					onclick={() => selectPreferredTime(time)}
					aria-label="Schedule at {time}"
				>
					{time}
				</button>
			{/each}
		</div>
	{/if}

	<div class="quick-actions">
		<button
			class="quick-btn"
			type="button"
			onclick={handleNextFreeSlot}
			aria-label="Schedule for next free slot"
		>
			<Zap size={12} />
			Next free slot
		</button>
		{#if hasSelection && status !== 'scheduled'}
			<button
				class="quick-btn clear"
				type="button"
				onclick={onunschedule}
				aria-label="Clear schedule"
			>
				<X size={12} />
				Clear
			</button>
		{/if}
	</div>
</div>

<style>
	.schedule-picker {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.tz-badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		color: var(--color-text-subtle);
		padding: 3px 8px;
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-surface-hover) 60%, transparent);
		width: fit-content;
	}

	.scheduled-state {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.scheduled-display {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-accent);
		padding: 6px 0;
	}

	.scheduled-actions {
		display: flex;
		gap: 6px;
	}

	.picker-row {
		display: flex;
		gap: 6px;
		align-items: center;
	}

	.picker-input {
		padding: 5px 8px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: var(--color-base);
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-mono);
		outline: none;
		min-width: 0;
	}

	.date-input {
		flex: 1.2;
	}

	.time-input {
		flex: 0.8;
	}

	.picker-input:focus {
		border-color: var(--color-accent);
	}

	.set-btn {
		padding: 5px 12px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
		flex-shrink: 0;
	}

	.set-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.set-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.preferred-slots {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
	}

	.slot-pill {
		padding: 4px 12px;
		border-radius: 14px;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-mono);
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.slot-pill:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}

	.slot-pill.active {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.quick-actions {
		display: flex;
		gap: 6px;
	}

	.quick-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.quick-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.quick-btn.clear:hover {
		border-color: var(--color-danger);
		color: var(--color-danger);
	}

	.picker-action-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.picker-action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.compact .tz-badge {
		font-size: 10px;
		padding: 2px 6px;
	}

	.compact .picker-input {
		padding: 4px 6px;
		font-size: 11px;
	}

	.compact .slot-pill {
		padding: 3px 10px;
		font-size: 11px;
	}

	@media (prefers-reduced-motion: reduce) {
		.slot-pill,
		.quick-btn,
		.set-btn,
		.picker-action-btn {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.slot-pill,
		.quick-btn,
		.set-btn,
		.picker-action-btn {
			min-height: 44px;
		}

		.picker-input {
			min-height: 44px;
		}
	}
</style>
