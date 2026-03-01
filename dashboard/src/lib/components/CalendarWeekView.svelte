<script lang="ts">
	import type { CalendarItem, ScheduleConfig } from '$lib/api';
	import ContentItem from './ContentItem.svelte';

	let {
		items,
		schedule,
		days,
		onslotclick,
		ondayclick,
		oncancel,
		onedit
	}: {
		items: CalendarItem[];
		schedule: ScheduleConfig | null;
		days: Date[];
		onslotclick?: (date: Date, time: string) => void;
		ondayclick?: (date: Date) => void;
		oncancel?: (id: number) => void;
		onedit?: (id: number) => void;
	} = $props();

	const dayLabels = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

	const preferredTimes = $derived(schedule?.preferred_times ?? []);

	const today = $derived(() => {
		const now = new Date();
		return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
	});

	function dateKey(d: Date): string {
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	function isToday(d: Date): boolean {
		return dateKey(d) === today();
	}

	function formatDay(d: Date): string {
		return d.getDate().toString();
	}

	function formatMonth(d: Date): string {
		return d.toLocaleDateString('en-US', { month: 'short' });
	}

	/** Group items by date key. */
	const itemsByDate = $derived(() => {
		const map = new Map<string, CalendarItem[]>();
		for (const item of items) {
			const d = new Date(item.timestamp);
			const key = dateKey(d);
			if (!map.has(key)) map.set(key, []);
			map.get(key)!.push(item);
		}
		return map;
	});

	/** Group items by date+time slot. */
	const itemsBySlot = $derived(() => {
		const map = new Map<string, CalendarItem[]>();
		for (const item of items) {
			const d = new Date(item.timestamp);
			const key = dateKey(d);
			const time = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
			const slotKey = `${key}|${time}`;
			if (!map.has(slotKey)) map.set(slotKey, []);
			map.get(slotKey)!.push(item);
		}
		return map;
	});

	/** Check if time matches a preferred slot (±30min). */
	function matchesSlot(time: string, itemTime: string): boolean {
		const [h1, m1] = time.split(':').map(Number);
		const [h2, m2] = itemTime.split(':').map(Number);
		const t1 = h1 * 60 + m1;
		const t2 = h2 * 60 + m2;
		return Math.abs(t1 - t2) <= 30;
	}

	function getSlotItems(day: Date, time: string): CalendarItem[] {
		const dk = dateKey(day);
		const result: CalendarItem[] = [];
		const byDate = itemsByDate();
		const dayItems = byDate.get(dk) ?? [];
		for (const item of dayItems) {
			const d = new Date(item.timestamp);
			const itemTime = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
			if (matchesSlot(time, itemTime)) {
				result.push(item);
			}
		}
		return result;
	}

	/** Get items not matching any preferred time slot (overflow). */
	function getUnslottedItems(day: Date): CalendarItem[] {
		const dk = dateKey(day);
		const byDate = itemsByDate();
		const dayItems = byDate.get(dk) ?? [];
		if (preferredTimes.length === 0) return dayItems;
		return dayItems.filter((item) => {
			const d = new Date(item.timestamp);
			const itemTime = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
			return !preferredTimes.some((pt) => matchesSlot(pt, itemTime));
		});
	}

	function handleSlotClick(day: Date, time: string) {
		if (onslotclick) {
			onslotclick(day, time);
		}
	}

	function handleCellKeydown(event: KeyboardEvent, activate: () => void) {
		if (event.key !== 'Enter' && event.key !== ' ') return;
		event.preventDefault();
		activate();
	}
</script>

<div class="week-view">
	<!-- Header row: day labels -->
	<div class="week-header">
		<div class="time-gutter"></div>
		{#each days as day, i}
			<button class="day-header" class:today={isToday(day)} onclick={() => ondayclick?.(day)}>
				<span class="day-label">{dayLabels[i]}</span>
				<span class="day-date" class:today={isToday(day)}>
					{#if day.getDate() === 1 || i === 0}
						<span class="day-month">{formatMonth(day)}</span>
					{/if}
					{formatDay(day)}
				</span>
			</button>
		{/each}
	</div>

	<!-- Slot rows -->
	{#if preferredTimes.length > 0}
		{#each preferredTimes as time}
			<div class="slot-row">
				<div class="time-gutter">
					<span class="time-label">{time}</span>
				</div>
				{#each days as day}
					{@const slotItems = getSlotItems(day, time)}
					<div
						class="slot-cell"
						class:has-items={slotItems.length > 0}
						class:today={isToday(day)}
						role="button"
						tabindex="0"
						onclick={() => handleSlotClick(day, time)}
						onkeydown={(event) => handleCellKeydown(event, () => handleSlotClick(day, time))}
					>
						{#if slotItems.length > 0}
							{#each slotItems as item}
								<ContentItem {item} {oncancel} {onedit} />
							{/each}
						{:else}
							<span class="empty-plus">+</span>
						{/if}
					</div>
				{/each}
			</div>
		{/each}
	{/if}

	<!-- Unslotted / overflow items -->
	{#if days.some((d) => getUnslottedItems(d).length > 0) || preferredTimes.length === 0}
		<div class="slot-row overflow-row">
			<div class="time-gutter">
				<span class="time-label">{preferredTimes.length > 0 ? 'Other' : 'All'}</span>
			</div>
			{#each days as day}
				{@const unslotted = preferredTimes.length > 0 ? getUnslottedItems(day) : (itemsByDate().get(dateKey(day)) ?? [])}
				<div
					class="slot-cell overflow-cell"
					class:today={isToday(day)}
					role="button"
					tabindex="0"
					onclick={() => ondayclick?.(day)}
					onkeydown={(event) => handleCellKeydown(event, () => ondayclick?.(day))}
				>
					{#if unslotted.length > 0}
						{#each unslotted as item}
							<ContentItem {item} {oncancel} {onedit} />
						{/each}
					{:else}
						<span class="empty-plus">+</span>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.week-view {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--color-border);
		border-radius: 8px;
		overflow: hidden;
	}

	.week-header {
		display: grid;
		grid-template-columns: 60px repeat(7, 1fr);
		border-bottom: 1px solid var(--color-border);
		background: var(--color-surface);
	}

	.time-gutter {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border-right: 1px solid var(--color-border-subtle);
		min-width: 60px;
	}

	.time-label {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
	}

	.day-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 8px 4px;
		border-right: 1px solid var(--color-border-subtle);
		border-top: none;
		border-bottom: none;
		border-left: none;
		background: transparent;
		cursor: pointer;
		gap: 2px;
		transition: background 0.15s ease;
	}

	.day-header:hover {
		background: var(--color-surface-hover);
	}

	.day-header:last-child {
		border-right: none;
	}

	.day-label {
		font-size: 10px;
		font-weight: 500;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.day-date {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.day-date.today {
		color: var(--color-accent);
	}

	.day-month {
		font-size: 10px;
		font-weight: 400;
		color: var(--color-text-muted);
	}

	.slot-row {
		display: grid;
		grid-template-columns: 60px repeat(7, 1fr);
		border-bottom: 1px solid var(--color-border-subtle);
		min-height: 48px;
	}

	.slot-row:last-child {
		border-bottom: none;
	}

	.slot-cell {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 4px;
		border-right: 1px solid var(--color-border-subtle);
		background: transparent;
		border-top: none;
		border-bottom: none;
		border-left: none;
		cursor: pointer;
		transition: background 0.15s ease;
		text-align: left;
		min-height: 40px;
	}

	.slot-cell:last-child {
		border-right: none;
	}

	.slot-cell:hover {
		background: var(--color-surface-hover);
	}

	.slot-cell.today {
		background: color-mix(in srgb, var(--color-accent) 3%, transparent);
	}

	.slot-cell.today:hover {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.empty-plus {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		min-height: 32px;
		font-size: 16px;
		font-weight: 300;
		color: var(--color-border);
		opacity: 0;
		transition: opacity 0.15s ease, color 0.15s ease;
	}

	.slot-cell:hover .empty-plus {
		opacity: 1;
		color: var(--color-accent);
	}

	.overflow-row {
		background: color-mix(in srgb, var(--color-surface) 50%, transparent);
	}
</style>
