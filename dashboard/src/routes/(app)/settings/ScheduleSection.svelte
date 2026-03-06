<script lang="ts">
	import { Clock, Plus, X } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import TimeRangeBar from '$lib/components/settings/TimeRangeBar.svelte';
	import { defaults, draft, updateDraft } from '$lib/stores/settings';

	const commonTimezones = [
		'UTC',
		'America/New_York',
		'America/Chicago',
		'America/Denver',
		'America/Los_Angeles',
		'America/Anchorage',
		'Pacific/Honolulu',
		'America/Toronto',
		'America/Vancouver',
		'America/Mexico_City',
		'America/Sao_Paulo',
		'America/Argentina/Buenos_Aires',
		'Europe/London',
		'Europe/Paris',
		'Europe/Berlin',
		'Europe/Madrid',
		'Europe/Rome',
		'Europe/Amsterdam',
		'Europe/Stockholm',
		'Europe/Moscow',
		'Europe/Istanbul',
		'Africa/Cairo',
		'Africa/Johannesburg',
		'Asia/Dubai',
		'Asia/Kolkata',
		'Asia/Bangkok',
		'Asia/Singapore',
		'Asia/Shanghai',
		'Asia/Tokyo',
		'Asia/Seoul',
		'Australia/Sydney',
		'Australia/Melbourne',
		'Pacific/Auckland'
	];

	let timezoneSearch = $state('');

	const filteredTimezones = $derived(
		timezoneSearch
			? commonTimezones.filter((tz) =>
					tz.toLowerCase().includes(timezoneSearch.toLowerCase())
				)
			: commonTimezones
	);

	const allDays = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

	function addPreferredTime() {
		if (!$draft) return;
		const times = [...$draft.schedule.preferred_times, '12:00'];
		updateDraft('schedule.preferred_times', times);
	}

	function removePreferredTime(index: number) {
		if (!$draft) return;
		const times = $draft.schedule.preferred_times.filter((_: string, i: number) => i !== index);
		updateDraft('schedule.preferred_times', times);
	}

	function updatePreferredTime(index: number, value: string) {
		if (!$draft) return;
		const times = [...$draft.schedule.preferred_times];
		times[index] = value;
		updateDraft('schedule.preferred_times', times);
	}
</script>

{#if $draft}
<SettingsSection
	id="schedule"
	title="Schedule"
	description="Active hours, posting times, and timezone"
	icon={Clock}
	scope="account"
	scopeKey="schedule"
>
	<div class="field-grid">
		<div class="field full-width">
			<label class="field-label" for="timezone">Timezone</label>
			<input
				id="timezone-search"
				type="text"
				class="text-input"
				placeholder="Search timezones..."
				bind:value={timezoneSearch}
			/>
			<select
				id="timezone"
				class="select-input"
				value={$draft.schedule.timezone}
				onchange={(e) =>
					updateDraft('schedule.timezone', e.currentTarget.value)}
			>
				{#each filteredTimezones as tz}
					<option value={tz}>{tz}</option>
				{/each}
			</select>
		</div>

		<div class="field full-width">
			<TimeRangeBar
				start={$draft.schedule.active_hours_start}
				end={$draft.schedule.active_hours_end}
				activeDays={$draft.schedule.active_days}
				timezone={$draft.schedule.timezone}
				onchange={(s, e) => {
					updateDraft('schedule.active_hours_start', s);
					updateDraft('schedule.active_hours_end', e);
				}}
				onDaysChange={(days) =>
					updateDraft('schedule.active_days', days)}
			/>
		</div>

		<div class="field full-width">
			<div class="preferred-times-header">
				<span class="field-label">Preferred Posting Times</span>
				<button type="button" class="add-btn" onclick={addPreferredTime}>
					<Plus size={14} />
					Add Time
				</button>
			</div>
			{#if $draft.schedule.preferred_times.length === 0}
				<span class="field-hint">
					No preferred times set. The content loop will use interval-based posting. Add times for specific scheduling, or use "auto" for research-backed defaults (09:15, 12:30, 17:00).
				</span>
			{/if}
			<div class="time-list">
				{#each $draft.schedule.preferred_times as time, i}
					<div class="time-row">
						{#if time === 'auto'}
							<span class="auto-badge">auto</span>
							<span class="field-hint">09:15, 12:30, 17:00</span>
						{:else}
							<input
								type="time"
								class="time-input"
								value={time}
								onchange={(e) =>
									updatePreferredTime(i, e.currentTarget.value)}
							/>
						{/if}
						<button
							type="button"
							class="remove-btn"
							onclick={() => removePreferredTime(i)}
							aria-label="Remove time"
						>
							<X size={14} />
						</button>
					</div>
				{/each}
			</div>
		</div>

		<div class="field">
			<label class="field-label" for="thread_day">Thread Preferred Day</label>
			<select
				id="thread_day"
				class="select-input"
				value={$draft.schedule.thread_preferred_day ?? ''}
				onchange={(e) =>
					updateDraft(
						'schedule.thread_preferred_day',
						e.currentTarget.value || null
					)}
			>
				<option value="">None (interval mode)</option>
				{#each allDays as day}
					<option value={day}>{day}</option>
				{/each}
			</select>
		</div>

		<div class="field">
			<label class="field-label" for="thread_time">Thread Preferred Time</label>
			<input
				id="thread_time"
				type="time"
				class="time-input wide"
				value={$draft.schedule.thread_preferred_time}
				onchange={(e) =>
					updateDraft(
						'schedule.thread_preferred_time',
						e.currentTarget.value
					)}
			/>
			{#if $defaults}
				<span class="field-default">
					Default: {$defaults.schedule.thread_preferred_time}
				</span>
			{/if}
		</div>
	</div>
</SettingsSection>
{/if}

<style>
	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.field-default {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	.text-input,
	.select-input,
	.time-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus,
	.select-input:focus,
	.time-input:focus {
		border-color: var(--color-accent);
	}

	.select-input {
		cursor: pointer;
		appearance: auto;
	}

	.time-input {
		width: fit-content;
	}

	.time-input.wide {
		width: 100%;
	}

	.preferred-times-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.add-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		background: none;
		color: var(--color-accent);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 12px;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.add-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}

	.time-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.time-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.auto-badge {
		padding: 4px 10px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		border-radius: 4px;
		font-size: 12px;
		font-weight: 500;
	}

	.remove-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: none;
		color: var(--color-text-subtle);
		cursor: pointer;
		border-radius: 4px;
		transition:
			color 0.15s,
			background 0.15s;
	}

	.remove-btn:hover {
		color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}
</style>
