<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { ChevronLeft, ChevronRight, Plus, Calendar, LayoutGrid, Loader2 } from 'lucide-svelte';
	import CalendarWeekView from '$lib/components/CalendarWeekView.svelte';
	import CalendarMonthView from '$lib/components/CalendarMonthView.svelte';
	import ComposeModal from '$lib/components/ComposeModal.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import type { ComposeRequest } from '$lib/api';
	import {
		calendarItems,
		schedule,
		loading,
		error,
		currentDate,
		viewMode,
		weekDays,
		monthDays,
		loadCalendar,
		loadSchedule,
		composeContent,
		cancelScheduledItem,
		goNext,
		goPrev,
		goToday,
		setViewMode,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/calendar';

	let composeOpen = $state(false);
	let composePrefillTime = $state<string | null>(null);
	let composePrefillDate = $state<Date | null>(null);

	const headerLabel = $derived(() => {
		const d = $currentDate;
		if ($viewMode === 'week') {
			const start = $weekDays[0];
			const end = $weekDays[6];
			const sameMonth = start.getMonth() === end.getMonth();
			if (sameMonth) {
				return `${start.toLocaleDateString('en-US', { month: 'long' })} ${start.getDate()}–${end.getDate()}, ${start.getFullYear()}`;
			}
			return `${start.toLocaleDateString('en-US', { month: 'short' })} ${start.getDate()} – ${end.toLocaleDateString('en-US', { month: 'short' })} ${end.getDate()}, ${end.getFullYear()}`;
		} else {
			return d.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
		}
	});

	function openCompose(date: Date | null = null, prefillTime: string | null = null) {
		composePrefillDate = date;
		composePrefillTime = prefillTime;
		composeOpen = true;
	}

	function handleSlotClick(date: Date, time: string) {
		openCompose(date, time);
	}

	function handleDayClick(date: Date) {
		openCompose(date);
	}

	async function handleCompose(data: ComposeRequest) {
		await composeContent(data);
		composeOpen = false;
	}

	function handleCancel(id: number) {
		cancelScheduledItem(id);
	}

	onMount(() => {
		loadSchedule();
		loadCalendar();
		startAutoRefresh();

		// Auto-open composer if redirected from onboarding.
		if ($page.url.searchParams.get('compose') === 'true') {
			openCompose(new Date());
		}
	});

	onDestroy(() => {
		stopAutoRefresh();
	});
</script>

<svelte:head>
	<title>Content Calendar — Tuitbot</title>
</svelte:head>

<div class="page-header">
	<div class="header-left">
		<h1>Content Calendar</h1>
		{#if $schedule}
			<span class="timezone-badge">{$schedule.timezone}</span>
		{/if}
	</div>
	<button class="compose-btn" onclick={() => openCompose(new Date())}>
		<Plus size={14} />
		Compose
	</button>
</div>

<div class="calendar-controls">
	<div class="nav-group">
		<button class="nav-btn" onclick={goPrev}>
			<ChevronLeft size={16} />
		</button>
		<button class="today-btn" onclick={goToday}>Today</button>
		<button class="nav-btn" onclick={goNext}>
			<ChevronRight size={16} />
		</button>
		<span class="period-label">{headerLabel()}</span>
	</div>

	<div class="view-toggle">
		<button class="view-btn" class:active={$viewMode === 'week'} onclick={() => setViewMode('week')}>
			<Calendar size={14} />
			Week
		</button>
		<button class="view-btn" class:active={$viewMode === 'month'} onclick={() => setViewMode('month')}>
			<LayoutGrid size={14} />
			Month
		</button>
	</div>
</div>

{#if $loading && $calendarItems.length === 0}
	<div class="loading-state">
		<Loader2 size={20} class="spinner" />
		<span>Loading calendar...</span>
	</div>
{:else if $error && $calendarItems.length === 0}
	<ErrorState message={$error} onretry={() => loadCalendar()} />
{:else}
	<div class="calendar-body">
		{#if $viewMode === 'week'}
			<CalendarWeekView
				items={$calendarItems}
				schedule={$schedule}
				days={$weekDays}
				onslotclick={handleSlotClick}
				ondayclick={handleDayClick}
				oncancel={handleCancel}
			/>
		{:else}
			<CalendarMonthView
				items={$calendarItems}
				days={$monthDays}
				currentDate={$currentDate}
				ondayclick={handleDayClick}
			/>
		{/if}
	</div>

	{#if $calendarItems.length === 0 && !$loading}
		<EmptyState
			title="No content scheduled"
			description="Click a time slot or use the Compose button to schedule your first post."
			actionLabel="Compose"
			onaction={() => openCompose(new Date())}
		/>
	{/if}

	<div class="legend">
		<span class="legend-item">
			<span class="legend-dot posted"></span>
			Posted
		</span>
		<span class="legend-item">
			<span class="legend-dot scheduled"></span>
			Scheduled
		</span>
		<span class="legend-item">
			<span class="legend-dot pending"></span>
			Pending approval
		</span>
		<span class="legend-item">
			<span class="legend-dot available"></span>
			Available slot
		</span>
	</div>
{/if}

<ComposeModal
	open={composeOpen}
	prefillTime={composePrefillTime}
	prefillDate={composePrefillDate}
	schedule={$schedule}
	onclose={() => (composeOpen = false)}
	onsubmit={handleCompose}
/>

<style>
	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 20px;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.timezone-badge {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		background: var(--color-surface);
		padding: 2px 8px;
		border-radius: 10px;
		border: 1px solid var(--color-border-subtle);
	}

	.compose-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.compose-btn:hover {
		background: var(--color-accent-hover);
	}

	.calendar-controls {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
		flex-wrap: wrap;
		gap: 12px;
	}

	.nav-group {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.nav-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.today-btn {
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.today-btn:hover {
		background: var(--color-surface-hover);
	}

	.period-label {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin-left: 8px;
	}

	.view-toggle {
		display: flex;
		gap: 0;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		overflow: hidden;
	}

	.view-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 12px;
		border: none;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.view-btn:first-child {
		border-right: 1px solid var(--color-border);
	}

	.view-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.view-btn.active {
		background: var(--color-surface);
		color: var(--color-accent);
	}

	.calendar-body {
		margin-bottom: 16px;
	}

	.loading-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 60px 20px;
		color: var(--color-text-muted);
		font-size: 13px;
	}

	.loading-state :global(.spinner) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}


	.legend {
		display: flex;
		flex-wrap: wrap;
		gap: 16px;
		padding: 12px 0;
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.legend-dot {
		width: 8px;
		height: 8px;
		border-radius: 2px;
	}

	.legend-dot.posted {
		background: var(--color-accent);
	}

	.legend-dot.scheduled {
		border: 1px dashed var(--color-accent);
		background: transparent;
	}

	.legend-dot.pending {
		background: var(--color-warning);
	}

	.legend-dot.available {
		background: var(--color-border);
		border-radius: 50%;
		opacity: 0.5;
	}
</style>
