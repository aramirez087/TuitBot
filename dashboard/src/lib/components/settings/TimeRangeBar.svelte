<script lang="ts">
	import { Globe } from 'lucide-svelte';

	interface Props {
		start: number;
		end: number;
		activeDays: string[];
		timezone: string;
		onchange: (start: number, end: number) => void;
		onDaysChange: (days: string[]) => void;
	}

	let { start, end, activeDays, timezone, onchange, onDaysChange }: Props = $props();

	let barEl: HTMLDivElement | undefined = $state();
	let dragging: 'start' | 'end' | null = $state(null);

	const allDays = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
	const hours = Array.from({ length: 24 }, (_, i) => i);

	function isActive(hour: number): boolean {
		if (start <= end) {
			return hour >= start && hour < end;
		}
		// Wrapping range (e.g. 22-6 means active from 22 to 6)
		return hour >= start || hour < end;
	}

	function hourFromPointer(clientX: number): number {
		if (!barEl) return 0;
		const rect = barEl.getBoundingClientRect();
		const ratio = (clientX - rect.left) / rect.width;
		return Math.min(23, Math.max(0, Math.round(ratio * 24)));
	}

	function onPointerDown(handle: 'start' | 'end', e: PointerEvent) {
		e.preventDefault();
		dragging = handle;
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging) return;
		const hour = hourFromPointer(e.clientX);
		if (dragging === 'start') {
			onchange(hour, end);
		} else {
			onchange(start, hour);
		}
	}

	function onPointerUp() {
		dragging = null;
	}

	function toggleDay(day: string) {
		if (activeDays.includes(day)) {
			if (activeDays.length > 1) {
				onDaysChange(activeDays.filter((d) => d !== day));
			}
		} else {
			onDaysChange([...activeDays, day]);
		}
	}

	function handleCellClick(hour: number) {
		// Clicking a cell sets the nearest handle to that position
		const distToStart = Math.abs(hour - start);
		const distToEnd = Math.abs(hour - end);
		if (distToStart <= distToEnd) {
			onchange(hour, end);
		} else {
			onchange(start, hour);
		}
	}

	const startPercent = $derived((start / 24) * 100);
	const endPercent = $derived((end / 24) * 100);

	const activeRange = $derived(() => {
		if (start <= end) {
			return `${formatHour(start)} - ${formatHour(end)}`;
		}
		return `${formatHour(start)} - ${formatHour(end)} (overnight)`;
	});

	function formatHour(h: number): string {
		return `${h.toString().padStart(2, '0')}:00`;
	}
</script>

<div class="time-range">
	<div class="time-header">
		<div class="time-label">
			Active hours: <strong>{activeRange()}</strong>
		</div>
		<div class="timezone-badge">
			<Globe size={12} />
			{timezone}
		</div>
	</div>

	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="bar-container"
		bind:this={barEl}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
	>
		{#each hours as hour}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="hour-cell"
				class:active={isActive(hour)}
				onclick={() => handleCellClick(hour)}
			></div>
		{/each}

		<!-- Start handle -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="handle handle-start"
			style="left: {startPercent}%"
			onpointerdown={(e) => onPointerDown('start', e)}
		>
			<div class="handle-grip"></div>
		</div>

		<!-- End handle -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="handle handle-end"
			style="left: {endPercent}%"
			onpointerdown={(e) => onPointerDown('end', e)}
		>
			<div class="handle-grip"></div>
		</div>
	</div>

	<div class="hour-labels">
		<span>0</span>
		<span>6</span>
		<span>12</span>
		<span>18</span>
		<span>23</span>
	</div>

	<div class="days-row">
		{#each allDays as day}
			<button
				type="button"
				class="day-pill"
				class:active={activeDays.includes(day)}
				onclick={() => toggleDay(day)}
			>
				{day}
			</button>
		{/each}
	</div>
</div>

<style>
	.time-range {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.time-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.time-label {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.time-label strong {
		color: var(--color-text);
		font-family: var(--font-mono);
		font-size: 12px;
	}

	.timezone-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 3px 8px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		border-radius: 4px;
		font-size: 11px;
		font-weight: 500;
	}

	.bar-container {
		position: relative;
		display: flex;
		height: 32px;
		border-radius: 4px;
		overflow: visible;
		cursor: pointer;
		touch-action: none;
	}

	.hour-cell {
		flex: 1;
		background: var(--color-base);
		border-right: 1px solid var(--color-border-subtle);
		transition: background 0.1s;
	}

	.hour-cell:first-child {
		border-radius: 4px 0 0 4px;
	}

	.hour-cell:last-child {
		border-radius: 0 4px 4px 0;
		border-right: none;
	}

	.hour-cell.active {
		background: color-mix(in srgb, var(--color-accent) 25%, transparent);
	}

	.handle {
		position: absolute;
		top: -2px;
		width: 12px;
		height: 36px;
		transform: translateX(-50%);
		cursor: ew-resize;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1;
	}

	.handle-grip {
		width: 6px;
		height: 20px;
		background: var(--color-accent);
		border-radius: 3px;
		box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
	}

	.handle:hover .handle-grip {
		background: var(--color-accent-hover);
		width: 8px;
	}

	.hour-labels {
		display: flex;
		justify-content: space-between;
		padding: 0 2px;
		font-size: 10px;
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
	}

	.days-row {
		display: flex;
		gap: 6px;
		margin-top: 4px;
	}

	.day-pill {
		flex: 1;
		padding: 6px 4px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-base);
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		text-align: center;
		transition:
			background 0.15s,
			border-color 0.15s,
			color 0.15s;
	}

	.day-pill:hover {
		background: var(--color-surface-hover);
	}

	.day-pill.active {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}
</style>
