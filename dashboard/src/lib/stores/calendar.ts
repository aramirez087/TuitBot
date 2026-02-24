import { writable, derived, get } from 'svelte/store';
import { api, type CalendarItem, type ScheduleConfig, type ComposeRequest } from '$lib/api';
import { events as wsEvents } from './websocket';

// --- Writable stores ---

export const calendarItems = writable<CalendarItem[]>([]);
export const schedule = writable<ScheduleConfig | null>(null);
export const loading = writable(true);
export const error = writable<string | null>(null);
export const currentDate = writable(new Date());
export const viewMode = writable<'week' | 'month'>('week');

// --- Derived stores ---

/** Start of the current week (Monday). */
export const weekStart = derived(currentDate, ($date) => {
	const d = new Date($date);
	const day = d.getDay();
	const diff = d.getDate() - day + (day === 0 ? -6 : 1); // Monday
	d.setDate(diff);
	d.setHours(0, 0, 0, 0);
	return d;
});

/** Array of 7 Date objects for the current week (Mon-Sun). */
export const weekDays = derived(weekStart, ($start) => {
	return Array.from({ length: 7 }, (_, i) => {
		const d = new Date($start);
		d.setDate(d.getDate() + i);
		return d;
	});
});

/** First day of the current month. */
export const monthStart = derived(currentDate, ($date) => {
	const d = new Date($date);
	d.setDate(1);
	d.setHours(0, 0, 0, 0);
	return d;
});

/** Array of Date objects for all days visible in the month calendar grid (includes padding). */
export const monthDays = derived(monthStart, ($start) => {
	const year = $start.getFullYear();
	const month = $start.getMonth();

	// First day of month
	const firstDay = new Date(year, month, 1);
	// Last day of month
	const lastDay = new Date(year, month + 1, 0);

	// Pad to start on Monday
	const startDow = firstDay.getDay();
	const padBefore = startDow === 0 ? 6 : startDow - 1;

	// Pad to end on Sunday
	const endDow = lastDay.getDay();
	const padAfter = endDow === 0 ? 0 : 7 - endDow;

	const days: Date[] = [];

	// Previous month padding
	for (let i = padBefore; i > 0; i--) {
		const d = new Date(firstDay);
		d.setDate(d.getDate() - i);
		days.push(d);
	}

	// Current month
	for (let i = 1; i <= lastDay.getDate(); i++) {
		days.push(new Date(year, month, i));
	}

	// Next month padding
	for (let i = 1; i <= padAfter; i++) {
		const d = new Date(lastDay);
		d.setDate(d.getDate() + i);
		days.push(d);
	}

	return days;
});

// --- Data loading ---

function formatDateISO(d: Date): string {
	return d.toISOString().replace('Z', '');
}

function getDateRange(): { from: string; to: string } {
	const mode = get(viewMode);
	const date = get(currentDate);

	if (mode === 'week') {
		const start = get(weekStart);
		const end = new Date(start);
		end.setDate(end.getDate() + 7);
		return { from: formatDateISO(start), to: formatDateISO(end) };
	} else {
		const start = get(monthStart);
		const end = new Date(start.getFullYear(), start.getMonth() + 1, 1);
		// Include padding days
		const startDow = start.getDay();
		const padBefore = startDow === 0 ? 6 : startDow - 1;
		const paddedStart = new Date(start);
		paddedStart.setDate(paddedStart.getDate() - padBefore);
		return { from: formatDateISO(paddedStart), to: formatDateISO(end) };
	}
}

export async function loadCalendar() {
	loading.set(true);
	error.set(null);

	try {
		const { from, to } = getDateRange();
		const data = await api.content.calendar(from, to);
		calendarItems.set(data);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load calendar');
	} finally {
		loading.set(false);
	}
}

export async function loadSchedule() {
	try {
		const data = await api.content.schedule();
		schedule.set(data);
	} catch {
		// Silent fail on schedule refresh
	}
}

export async function composeContent(data: ComposeRequest) {
	try {
		const result = await api.content.compose(data);
		// Reload calendar to show the new item
		await loadCalendar();
		return result;
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to compose content');
		throw e;
	}
}

export async function cancelScheduledItem(id: number) {
	try {
		await api.content.cancelScheduled(id);
		calendarItems.update(($items) => $items.filter((i) => !(i.id === id && i.source === 'manual')));
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to cancel item');
	}
}

export async function updateScheduledItem(id: number, data: { content?: string; scheduled_for?: string }) {
	try {
		await api.content.updateScheduled(id, data);
		await loadCalendar();
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to update item');
	}
}

// --- Navigation ---

export function goNext() {
	const mode = get(viewMode);
	currentDate.update(($d) => {
		const d = new Date($d);
		if (mode === 'week') {
			d.setDate(d.getDate() + 7);
		} else {
			d.setMonth(d.getMonth() + 1);
		}
		return d;
	});
	loadCalendar();
}

export function goPrev() {
	const mode = get(viewMode);
	currentDate.update(($d) => {
		const d = new Date($d);
		if (mode === 'week') {
			d.setDate(d.getDate() - 7);
		} else {
			d.setMonth(d.getMonth() - 1);
		}
		return d;
	});
	loadCalendar();
}

export function goToday() {
	currentDate.set(new Date());
	loadCalendar();
}

export function setViewMode(mode: 'week' | 'month') {
	viewMode.set(mode);
	loadCalendar();
}

// --- WebSocket integration ---

let lastEventCount = 0;

wsEvents.subscribe(($events) => {
	if ($events.length === 0 || $events.length === lastEventCount) return;
	lastEventCount = $events.length;

	const latest = $events[0];

	if (latest.type === 'ContentScheduled') {
		loadCalendar();
	}
});

// --- Auto-refresh ---

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export function startAutoRefresh(intervalMs: number = 30_000) {
	stopAutoRefresh();
	refreshInterval = setInterval(() => {
		loadCalendar();
	}, intervalMs);
}

export function stopAutoRefresh() {
	if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = null;
	}
}
