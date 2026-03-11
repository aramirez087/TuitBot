<script lang="ts">
	import type { ScheduleConfig } from '$lib/api';
	import SchedulePicker from '../SchedulePicker.svelte';

	let {
		open = false,
		schedule = null,
		selectedDate = null,
		selectedTime = null,
		onschedule,
		onunschedule,
		onclose,
	}: {
		open?: boolean;
		schedule?: ScheduleConfig | null;
		selectedDate?: string | null;
		selectedTime?: string | null;
		onschedule?: (date: string, time: string) => void;
		onunschedule?: () => void;
		onclose?: () => void;
	} = $props();

	let sheetEl: HTMLDivElement | undefined = $state();
	let isMobile = $state(false);

	$effect(() => {
		const mql = window.matchMedia('(max-width: 640px)');
		isMobile = mql.matches;
		const handler = (e: MediaQueryListEvent) => {
			isMobile = e.matches;
		};
		mql.addEventListener('change', handler);
		return () => mql.removeEventListener('change', handler);
	});

	$effect(() => {
		if (!open) return;

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') {
				e.stopPropagation();
				onclose?.();
			}
		}

		function handleClickOutside(e: MouseEvent) {
			if (sheetEl && !sheetEl.contains(e.target as Node)) {
				onclose?.();
			}
		}

		document.addEventListener('keydown', handleKeydown, true);
		// Delay click listener to avoid immediate close from the trigger click
		const timer = setTimeout(() => {
			document.addEventListener('mousedown', handleClickOutside);
		}, 10);

		return () => {
			document.removeEventListener('keydown', handleKeydown, true);
			clearTimeout(timer);
			document.removeEventListener('mousedown', handleClickOutside);
		};
	});

	function handleSchedule(date: string, time: string) {
		onschedule?.(date, time);
		onclose?.();
	}

	function handleUnschedule() {
		onunschedule?.();
		onclose?.();
	}
</script>

{#if open}
	<div
		class="sheet-wrapper"
		class:mobile={isMobile}
		bind:this={sheetEl}
		role="dialog"
		aria-label="Schedule post"
	>
		<div class="sheet-header">
			<span class="sheet-title">Schedule</span>
		</div>
		<div class="sheet-body">
			<SchedulePicker
				timezone={schedule?.timezone ?? 'UTC'}
				preferredTimes={schedule?.preferred_times ?? []}
				{selectedDate}
				{selectedTime}
				status="draft"
				onschedule={handleSchedule}
				onunschedule={handleUnschedule}
			/>
		</div>
	</div>
{/if}

<style>
	.sheet-wrapper {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 6px;
		width: 320px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		box-shadow:
			0 8px 32px rgba(0, 0, 0, 0.4),
			0 2px 8px rgba(0, 0, 0, 0.2);
		z-index: 100;
		animation: sheet-in 0.15s ease-out;
	}

	.sheet-wrapper.mobile {
		position: fixed;
		top: auto;
		bottom: 0;
		left: 0;
		right: 0;
		width: 100%;
		margin-top: 0;
		border-radius: 14px 14px 0 0;
		max-height: 70vh;
		overflow-y: auto;
		animation: slide-up 0.2s ease-out;
	}

	.sheet-header {
		padding: 10px 14px 0;
	}

	.sheet-title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-muted);
	}

	.sheet-body {
		padding: 10px 14px 14px;
	}

	@keyframes sheet-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes slide-up {
		from {
			transform: translateY(100%);
		}
		to {
			transform: translateY(0);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.sheet-wrapper,
		.sheet-wrapper.mobile {
			animation: none;
		}
	}
</style>
