/**
 * Scheduling timezone utilities.
 *
 * Contract:
 * - All scheduled_for values sent to the server are UTC ISO-8601 with Z suffix.
 * - The account timezone (from ScheduleConfig.timezone) is the canonical
 *   user-facing timezone for display and input.
 * - Browser timezone is irrelevant — never used for scheduling logic.
 */

/**
 * Build a UTC ISO-8601 string from a user-selected date and time
 * in the account timezone.
 *
 * @param date - "YYYY-MM-DD" from a date picker
 * @param time - "HH:MM" from a time picker
 * @param timezone - IANA timezone string (e.g. "America/New_York")
 * @returns UTC ISO-8601 string with Z suffix (e.g. "2026-03-10T18:00:00Z")
 */
export function buildScheduledFor(date: string, time: string, timezone: string): string {
	// Parse the date and time parts
	const [year, month, day] = date.split('-').map(Number);
	const [hours, minutes] = time.split(':').map(Number);

	// Use a binary search approach to find the UTC offset for this date/time/timezone.
	// We construct a UTC Date, format it in the target timezone, and compare.
	// Start with a rough guess assuming UTC, then adjust.
	const guessUtc = new Date(Date.UTC(year, month - 1, day, hours, minutes, 0));

	// Get the offset by formatting the guess in the target timezone
	const offset = getUtcOffsetMinutes(guessUtc, timezone);

	// The actual UTC time is the local time minus the offset
	const actualUtc = new Date(guessUtc.getTime() - offset * 60_000);

	// Verify: format actualUtc in timezone should give us back the original date/time
	// If not (DST edge case), adjust once more
	const verifyOffset = getUtcOffsetMinutes(actualUtc, timezone);
	let finalUtc = actualUtc;
	if (verifyOffset !== offset) {
		finalUtc = new Date(guessUtc.getTime() - verifyOffset * 60_000);
	}

	return formatUtcIso(finalUtc);
}

/**
 * Format a UTC ISO-8601 timestamp for display in the account timezone.
 */
export function formatInAccountTz(
	utcIso: string,
	timezone: string,
	options?: Intl.DateTimeFormatOptions
): string {
	const dt = parseUtcIso(utcIso);
	const defaults: Intl.DateTimeFormatOptions = {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		timeZone: timezone,
		...options
	};
	return new Intl.DateTimeFormat('en-US', defaults).format(dt);
}

/**
 * Extract date ("YYYY-MM-DD") and time ("HH:MM") parts from a UTC
 * timestamp, converting to the account timezone for form inputs.
 */
export function toAccountTzParts(
	utcIso: string,
	timezone: string
): { date: string; time: string } {
	const dt = parseUtcIso(utcIso);

	const yearFmt = new Intl.DateTimeFormat('en-CA', {
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		timeZone: timezone
	});
	const dateParts = yearFmt.format(dt); // "YYYY-MM-DD" in en-CA locale

	const timeFmt = new Intl.DateTimeFormat('en-GB', {
		hour: '2-digit',
		minute: '2-digit',
		hour12: false,
		timeZone: timezone
	});
	const timeParts = timeFmt.format(dt); // "HH:MM"

	return { date: dateParts, time: timeParts };
}

/**
 * Get current date and time parts in the account timezone.
 */
export function nowInAccountTz(timezone: string): { date: string; time: string } {
	const now = new Date();
	return toAccountTzParts(now.toISOString(), timezone);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/**
 * Get the UTC offset in minutes for a given UTC Date in a timezone.
 * Positive = east of UTC (e.g., +330 for IST), negative = west (e.g., -300 for EST).
 */
function getUtcOffsetMinutes(utcDate: Date, timezone: string): number {
	// Format the UTC date in the target timezone to get local components
	const parts = new Intl.DateTimeFormat('en-US', {
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit',
		hour12: false,
		timeZone: timezone
	}).formatToParts(utcDate);

	const get = (type: Intl.DateTimeFormatPartTypes): number => {
		const part = parts.find((p) => p.type === type);
		return part ? parseInt(part.value, 10) : 0;
	};

	// Reconstruct what the local time is (as a UTC timestamp for comparison)
	const localAsUtc = Date.UTC(
		get('year'),
		get('month') - 1,
		get('day'),
		get('hour'),
		get('minute'),
		get('second')
	);

	return (localAsUtc - utcDate.getTime()) / 60_000;
}

/** Parse a UTC ISO string (with or without Z) into a Date. */
function parseUtcIso(iso: string): Date {
	const s = iso.endsWith('Z') ? iso : iso + 'Z';
	return new Date(s);
}

/** Format a Date as "YYYY-MM-DDTHH:MM:SSZ". */
function formatUtcIso(date: Date): string {
	const y = date.getUTCFullYear();
	const mo = String(date.getUTCMonth() + 1).padStart(2, '0');
	const d = String(date.getUTCDate()).padStart(2, '0');
	const h = String(date.getUTCHours()).padStart(2, '0');
	const mi = String(date.getUTCMinutes()).padStart(2, '0');
	const s = String(date.getUTCSeconds()).padStart(2, '0');
	return `${y}-${mo}-${d}T${h}:${mi}:${s}Z`;
}
