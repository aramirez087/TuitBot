<script lang="ts">
	import { Send, Clock } from "lucide-svelte";
	import type { Snippet } from "svelte";
	import type { ScheduleConfig } from "$lib/api";
	import { formatInAccountTz } from "$lib/utils/timezone";
	import ScheduleComposerSheet from "./ScheduleComposerSheet.svelte";

	let {
		canSubmit,
		submitting,
		selectedTime,
		submitError,
		canPublish = true,
		inspectorOpen = false,
		embedded = false,
		timezone = null,
		scheduledDate = null,
		schedule = null,
		scheduledFor = null,
		onsubmit,
		onscheduleselect,
		onunschedule,
		children,
		toolbar,
		inspector,
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		submitError: string | null;
		canPublish?: boolean;
		inspectorOpen?: boolean;
		embedded?: boolean;
		timezone?: string | null;
		scheduledDate?: string | null;
		schedule?: ScheduleConfig | null;
		scheduledFor?: string | null;
		onsubmit: () => void;
		onscheduleselect?: (date: string, time: string) => void;
		onunschedule?: () => void;
		children: Snippet;
		toolbar?: Snippet;
		inspector?: Snippet;
	} = $props();

	let scheduleSheetOpen = $state(false);

	const isScheduled = $derived(!!selectedTime && !!scheduledDate);

	const submitLabel = $derived(() => {
		if (submitting) return isScheduled ? "Scheduling\u2026" : "Submitting\u2026";
		if (isScheduled && scheduledFor && timezone) {
			return formatInAccountTz(scheduledFor, timezone, {
				month: "short",
				day: "numeric",
				hour: "numeric",
				minute: "2-digit",
				timeZoneName: "short",
			});
		}
		if (selectedTime) return "Schedule";
		if (canPublish) return "Post now";
		return "Save to Calendar";
	});
</script>

<div class="canvas" class:with-inspector={inspectorOpen && inspector}>
	<div class="canvas-main">
		{@render children()}

		{#if submitError}
			<div class="error-msg" role="alert">{submitError}</div>
		{/if}

		{#if toolbar}
			{@render toolbar()}
		{/if}

		{#if !embedded}
			<div class="submit-anchor">
				<div class="submit-group">
					<div class="schedule-trigger-wrap">
						<button
							class="schedule-trigger-btn"
							onclick={() => { scheduleSheetOpen = !scheduleSheetOpen; }}
							aria-label={scheduleSheetOpen ? "Close schedule picker" : "Open schedule picker"}
							title="Schedule"
						>
							<Clock size={14} />
						</button>
						<ScheduleComposerSheet
							open={scheduleSheetOpen}
							{schedule}
							selectedDate={scheduledDate}
							{selectedTime}
							onschedule={(date, time) => { onscheduleselect?.(date, time); }}
							onunschedule={() => { onunschedule?.(); }}
							onclose={() => { scheduleSheetOpen = false; }}
						/>
					</div>
					<button
						class="submit-pill"
						onclick={onsubmit}
						disabled={!canSubmit || submitting}
						title={!canPublish && !selectedTime ? 'Connect X API to publish directly' : ''}
					>
						{#if isScheduled}
							<Clock size={14} />
						{:else}
							<Send size={14} />
						{/if}
						{submitLabel()}
					</button>
				</div>
			</div>
		{/if}
	</div>

	{#if inspectorOpen && inspector}
		<div class="canvas-inspector">
			{@render inspector()}
		</div>
	{/if}
</div>

<style>
	.canvas {
		display: flex;
		flex: 1;
		min-height: 0;
		position: relative;
	}

	.canvas.with-inspector {
		display: flex;
	}

	.canvas-main {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow-y: auto;
		max-width: 660px;
		margin-left: auto;
		margin-right: auto;
		width: 100%;
	}

	.canvas-main > :global(:first-child) {
		padding-top: 12px;
	}

	.canvas-main {
		padding: 0 24px 24px;
	}

	.canvas-inspector {
		width: 260px;
		flex-shrink: 0;
		border-left: 1px solid var(--color-border-subtle);
		overflow-y: auto;
		padding: 12px 16px;
		background: color-mix(
			in srgb,
			var(--color-base) 50%,
			var(--color-surface)
		);
	}

	.error-msg {
		margin-top: 12px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 12px;
	}

	.submit-anchor {
		position: sticky;
		bottom: 0;
		display: flex;
		justify-content: flex-end;
		padding: 12px 0 0;
		pointer-events: none;
	}

	.submit-group {
		display: flex;
		align-items: center;
		gap: 6px;
		pointer-events: auto;
	}

	.schedule-trigger-wrap {
		position: relative;
	}

	.schedule-trigger-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: 1px solid var(--color-border);
		border-radius: 20px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.schedule-trigger-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: var(--color-surface-hover);
	}

	.submit-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 40px;
		padding: 0 24px;
		border: none;
		border-radius: 20px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		pointer-events: auto;
		transition: all 0.15s ease;
		box-shadow: 0 2px 12px rgba(0, 0, 0, 0.3);
	}

	.submit-pill:hover:not(:disabled) {
		background: var(--color-accent-hover);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
	}

	.submit-pill:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	@media (pointer: coarse) {
		.submit-pill {
			min-height: 44px;
		}
	}

	@media (max-width: 768px) {
		.canvas-inspector {
			display: none;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.submit-pill {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.canvas-main {
			padding: 0 16px 16px;
		}

		.submit-anchor {
			padding-bottom: env(safe-area-inset-bottom, 0px);
		}

		.submit-pill {
			width: 100%;
			justify-content: center;
		}
	}
</style>
