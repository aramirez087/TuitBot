<script lang="ts">
	import { Clock, X, Loader2 } from 'lucide-svelte';
	import { formatInAccountTz, toAccountTzParts, nowInAccountTz } from '$lib/utils/timezone';
	import { trackFunnel } from '$lib/analytics/funnel';
	import SchedulePickerSlots from './SchedulePickerSlots.svelte';

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
	// Brief loading state for "next free slot" — gives visual feedback even though the calc is synchronous.
	let loadingNextSlot = $state(false);

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

	$effect(() => {
		if (selectedDate && !customDate) customDate = selectedDate;
		if (selectedTime && !customTime) customTime = selectedTime;
	});

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
		loadingNextSlot = true;
		// Calc is synchronous; setTimeout lets Svelte flush the loading state to DOM before we resolve.
		setTimeout(() => {
			_resolveNextFreeSlot();
			loadingNextSlot = false;
		}, 0);
	}

	function _resolveNextFreeSlot() {
		const now = nowInAccountTz(timezone);
		const [nowH, nowM] = now.time.split(':').map(Number);
		const nowMinutes = nowH * 60 + nowM;

		for (const time of preferredTimes) {
			const [h, m] = time.split(':').map(Number);
			const mins = h * 60 + m;
			if (mins > nowMinutes + 15) {
				onschedule?.(now.date, time);
				return;
			}
		}

		if (preferredTimes.length > 0) {
			const tomorrow = new Date();
			tomorrow.setDate(tomorrow.getDate() + 1);
			const y = tomorrow.getFullYear();
			const mo = String(tomorrow.getMonth() + 1).padStart(2, '0');
			const d = String(tomorrow.getDate()).padStart(2, '0');
			onschedule?.(`${y}-${mo}-${d}`, preferredTimes[0]);
			return;
		}

		const tomorrow = new Date();
		tomorrow.setDate(tomorrow.getDate() + 1);
		const y = tomorrow.getFullYear();
		const mo = String(tomorrow.getMonth() + 1).padStart(2, '0');
		const d = String(tomorrow.getDate()).padStart(2, '0');
		const [nowH2] = now.time.split(':').map(Number);
		const nextHour = String(Math.min(nowH2 + 1, 23)).padStart(2, '0');
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

	<SchedulePickerSlots
		{preferredTimes}
		{selectedTime}
		{hasSelection}
		{status}
		{compact}
		{loadingNextSlot}
		onselecttime={selectPreferredTime}
		onquickslot={handleNextFreeSlot}
		{onunschedule}
	/>
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

	@media (prefers-reduced-motion: reduce) {
		.set-btn,
		.picker-action-btn {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.set-btn,
		.picker-action-btn {
			min-height: 44px;
		}

		.picker-input {
			min-height: 44px;
		}
	}
</style>
