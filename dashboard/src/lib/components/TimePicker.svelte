<script lang="ts">
	import type { ScheduleConfig } from '$lib/api';
	import { Clock } from 'lucide-svelte';

	let {
		schedule,
		selectedTime = null,
		targetDate = null,
		onselect
	}: {
		schedule: ScheduleConfig | null;
		selectedTime: string | null;
		targetDate?: Date | null;
		onselect: (time: string) => void;
	} = $props();

	let customTime = $state('');

	const preferredTimes = $derived(schedule?.preferred_times ?? []);

	const dateDisplay = $derived(() => {
		if (!targetDate) return 'today';
		const now = new Date();
		const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
		const target = new Date(targetDate.getFullYear(), targetDate.getMonth(), targetDate.getDate());
		const diffDays = Math.round((target.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
		if (diffDays === 0) return 'today';
		if (diffDays === 1) return 'tomorrow';
		return targetDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	});

	function selectCustom() {
		if (customTime && /^\d{2}:\d{2}$/.test(customTime)) {
			onselect(customTime);
		}
	}
</script>

<div class="time-picker">
	<div class="picker-label">
		<Clock size={12} />
		<span>Schedule for {dateDisplay()}</span>
	</div>

	{#if preferredTimes.length > 0}
		<div class="preferred-slots">
			{#each preferredTimes as time}
				<button
					class="slot-btn"
					class:selected={selectedTime === time}
					onclick={() => onselect(time)}
				>
					{time}
				</button>
			{/each}
		</div>
	{/if}

	<div class="custom-time">
		<input
			type="time"
			class="time-input"
			bind:value={customTime}
			placeholder="HH:MM"
		/>
		<button class="custom-btn" onclick={selectCustom} disabled={!customTime}>
			Set
		</button>
	</div>

	{#if selectedTime}
		<button class="clear-btn" onclick={() => onselect('')}>
			Clear (next available slot)
		</button>
	{/if}
</div>

<style>
	.time-picker {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.picker-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.preferred-slots {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.slot-btn {
		padding: 4px 12px;
		border-radius: 14px;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-mono);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.slot-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}

	.slot-btn.selected {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.custom-time {
		display: flex;
		gap: 6px;
		align-items: center;
	}

	.time-input {
		padding: 4px 8px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: var(--color-base);
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-mono);
		width: 100px;
	}

	.time-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.custom-btn {
		padding: 4px 12px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.custom-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
	}

	.custom-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.clear-btn {
		padding: 2px 0;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		text-align: left;
	}

	.clear-btn:hover {
		color: var(--color-text-muted);
	}
</style>
