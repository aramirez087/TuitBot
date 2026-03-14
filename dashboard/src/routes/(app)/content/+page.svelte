<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { Plus, Loader2 } from 'lucide-svelte';
	import CalendarWeekView from '$lib/components/CalendarWeekView.svelte';
	import CalendarMonthView from '$lib/components/CalendarMonthView.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import EmptyState from '$lib/components/EmptyState.svelte';
	import ContentCalendarControls from './ContentCalendarControls.svelte';
	import { api } from '$lib/api';
	import { trackFunnel } from '$lib/analytics/funnel';
	import {
		calendarItems,
		schedule,
		loading,
		error,
		currentDate,
		viewMode,
		weekDays,
		monthDays,
		accountTimezone,
		loadCalendar,
		loadSchedule,
		cancelScheduledItem,
		goNext,
		goPrev,
		goToday,
		setViewMode,
		startAutoRefresh,
		stopAutoRefresh,
	} from '$lib/stores/calendar';
	import { ACCOUNT_SWITCHED_EVENT } from '$lib/stores/accounts';

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

	function buildScheduledFor(date: Date, time: string | null): string | undefined {
		if (!time) return undefined;
		const y = date.getFullYear();
		const m = String(date.getMonth() + 1).padStart(2, '0');
		const d = String(date.getDate()).padStart(2, '0');
		return `${y}-${m}-${d}T${time}:00`;
	}

	async function createDraftAndRedirect(date: Date | null, time: string | null, source: string) {
		try {
			const scheduledFor = date && time ? buildScheduledFor(date, time) : undefined;
			const isFirst = $calendarItems.length === 0;
			const result = await api.draftStudio.create({ content_type: 'tweet' });
			console.info('[draft-studio]', { event: 'draft_created', id: result.id, source });
			if (isFirst) trackFunnel('activation:first-draft-created', { source });
			const params = new URLSearchParams({ id: String(result.id) });
			if (scheduledFor) params.set('prefill_schedule', scheduledFor);
			goto(`/drafts?${params.toString()}`);
		} catch {
			goto('/drafts?new=true');
		}
	}

	function handleSlotClick(date: Date, time: string) {
		createDraftAndRedirect(date, time, 'calendar-slot');
	}

	function handleDayClick(date: Date) {
		createDraftAndRedirect(date, null, 'calendar-day');
	}

	function handleCancel(id: number) { cancelScheduledItem(id); }
	function handleEditScheduled(id: number) { goto(`/drafts?id=${id}`); }
	function handleReschedule(id: number) { goto(`/drafts?id=${id}`); }

	async function handleUnschedule(id: number) {
		try {
			await api.draftStudio.unschedule(id);
			await loadCalendar();
		} catch {
			try {
				await api.content.cancelScheduled(id);
				await loadCalendar();
			} catch {
				// Silently fail — item may have already been unscheduled
			}
		}
	}

	onMount(() => {
		loadSchedule();
		loadCalendar();
		startAutoRefresh();
		const handler = () => { loadSchedule(); loadCalendar(); };
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () => window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});

	onDestroy(() => stopAutoRefresh());
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
	<button class="compose-btn" onclick={() => createDraftAndRedirect(new Date(), null, 'calendar-button')}>
		<Plus size={14} />
		New Draft
	</button>
</div>

<ContentCalendarControls
	headerLabel={headerLabel()}
	viewMode={$viewMode}
	onprev={goPrev}
	onnext={goNext}
	ontoday={goToday}
	onsetview={setViewMode}
/>

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
				timezone={$accountTimezone}
				onslotclick={handleSlotClick}
				ondayclick={handleDayClick}
				oncancel={handleCancel}
				onedit={handleEditScheduled}
				onreschedule={handleReschedule}
				onunschedule={handleUnschedule}
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
			description="Click a time slot or use the New Draft button to start writing in Draft Studio."
			actionLabel="New Draft"
			onaction={() => createDraftAndRedirect(new Date(), null, 'calendar-empty')}
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
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
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

	.legend-dot.posted { background: var(--color-accent); }
	.legend-dot.scheduled { border: 1px dashed var(--color-accent); background: transparent; }
	.legend-dot.pending { background: var(--color-warning); }
	.legend-dot.available { background: var(--color-border); border-radius: 50%; opacity: 0.5; }
</style>
