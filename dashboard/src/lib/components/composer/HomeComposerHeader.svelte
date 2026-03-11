<script lang="ts">
	import {
		Send,
		Clock,
		ScanEye,
		PanelRight,
		Search,
		Loader2,
	} from "lucide-svelte";
	import type { Snippet } from "svelte";
	import { formatInAccountTz } from "$lib/utils/timezone";
	import ScheduleComposerSheet from "./ScheduleComposerSheet.svelte";
	import type { ScheduleConfig } from "$lib/api";

	let {
		canSubmit,
		submitting,
		selectedTime,
		inspectorOpen,
		previewVisible,
		canPublish = true,
		handle = null,
		avatarUrl = null,
		displayName = null,
		mode = "tweet",
		blockCount = 1,
		headerLeft,
		timezone = null,
		scheduledDate = null,
		schedule = null,
		scheduledFor = null,
		onsubmit,
		onpublishnow,
		onscheduleselect,
		onunschedule,
		ontoggleinspector,
		ontogglepreview,
		onopenpalette,
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		inspectorOpen: boolean;
		previewVisible: boolean;
		canPublish?: boolean;
		handle?: string | null;
		avatarUrl?: string | null;
		displayName?: string | null;
		mode?: "tweet" | "thread";
		blockCount?: number;
		headerLeft?: Snippet;
		timezone?: string | null;
		scheduledDate?: string | null;
		schedule?: ScheduleConfig | null;
		scheduledFor?: string | null;
		onsubmit: () => void;
		onpublishnow?: () => void;
		onscheduleselect?: (date: string, time: string) => void;
		onunschedule?: () => void;
		ontoggleinspector: () => void;
		ontogglepreview: () => void;
		onopenpalette: () => void;
	} = $props();

	let scheduleSheetOpen = $state(false);

	const isScheduled = $derived(!!selectedTime && !!scheduledDate);

	const scheduleLabel = $derived(() => {
		if (!scheduledFor || !timezone) return "Schedule";
		return formatInAccountTz(scheduledFor, timezone, {
			month: "short",
			day: "numeric",
			hour: "numeric",
			minute: "2-digit",
			timeZoneName: "short",
		});
	});
</script>

<header class="home-header" class:has-left={!!headerLeft}>
	{#if headerLeft}
		<div class="header-left">
			{@render headerLeft()}
		</div>
	{/if}

	<div class="header-right">
		{#if isScheduled}
			<div class="button-group">
				<button
					class="cta-pill schedule-pill"
					onclick={onsubmit}
					disabled={!canSubmit || submitting}
					title="Schedule post"
					aria-label={submitting ? "Scheduling" : `Schedule for ${scheduleLabel()}`}
				>
					{#if submitting}
						<Loader2 size={14} class="spin-icon" />
						<span>Scheduling&hellip;</span>
					{:else}
						<Clock size={14} />
						<span>{scheduleLabel()}</span>
					{/if}
				</button>
				{#if canPublish}
					<button
						class="cta-pill publish-now-btn"
						onclick={onpublishnow ?? onsubmit}
						disabled={!canSubmit || submitting}
						title="Publish immediately"
						aria-label="Publish now"
					>
						<Send size={14} />
					</button>
				{/if}
			</div>
		{:else if canPublish}
			<button
				class="cta-pill publish-pill"
				onclick={onsubmit}
				disabled={!canSubmit || submitting}
				title="Publish now"
				aria-label={submitting ? "Posting" : "Publish now"}
			>
				{#if submitting}
					<Loader2 size={14} class="spin-icon" />
				{:else}
					<Send size={14} />
				{/if}
				<span>{submitting ? "Posting\u2026" : "Publish"}</span>
			</button>
		{:else}
			<button
				class="cta-pill schedule-pill"
				onclick={onsubmit}
				disabled={!canSubmit || submitting}
				title="Connect X API credentials to publish directly"
				aria-label={submitting ? "Saving" : "Save to calendar"}
			>
				{#if submitting}
					<Loader2 size={14} class="spin-icon" />
				{:else}
					<Clock size={14} />
				{/if}
				<span>{submitting ? "Saving\u2026" : "Save to Calendar"}</span>
			</button>
		{/if}

		<div class="icon-tools">
			<div class="schedule-trigger-wrap">
				<button
					class="icon-btn"
					class:active={scheduleSheetOpen}
					onclick={() => { scheduleSheetOpen = !scheduleSheetOpen; }}
					aria-label={scheduleSheetOpen ? "Close schedule picker" : "Open schedule picker"}
					title="Schedule"
				>
					<Clock size={15} />
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
				class="icon-btn"
				class:active={previewVisible}
				onclick={ontogglepreview}
				aria-label={previewVisible ? "Close preview" : "Open preview"}
				title={previewVisible
					? "Close preview (\u2318\u21E7P)"
					: "Open preview (\u2318\u21E7P)"}
			>
				<ScanEye size={15} />
			</button>

			<button
				class="icon-btn"
				class:active={inspectorOpen}
				onclick={ontoggleinspector}
				aria-label={inspectorOpen
					? "Close inspector"
					: "Open inspector"}
				title={inspectorOpen
					? "Close inspector (\u2318I)"
					: "Open inspector (\u2318I)"}
			>
				<PanelRight size={15} />
			</button>

			<button
				class="icon-btn"
				onclick={onopenpalette}
				aria-label="Command palette"
				title="Commands (\u2318K)"
			>
				<Search size={15} />
			</button>
		</div>
	</div>
</header>

<style>
	.home-header {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 12px;
		padding: 8px 16px;
		flex-shrink: 0;
		border-bottom: 1px solid
			color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
	}

	.home-header.has-left {
		justify-content: space-between;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		flex-shrink: 1;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	.button-group {
		display: flex;
		align-items: center;
		gap: 3px;
	}

	.cta-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 38px;
		padding: 0 20px;
		border-radius: 20px;
		font-size: 13.5px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
		white-space: nowrap;
		border: none;
	}

	.publish-pill,
	.schedule-pill {
		background: var(--color-accent);
		color: #fff;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.publish-pill:hover:not(:disabled),
	.schedule-pill:hover:not(:disabled) {
		background: var(--color-accent-hover);
		box-shadow: 0 3px 12px rgba(0, 0, 0, 0.25);
		transform: translateY(-1px);
	}

	.publish-pill:disabled,
	.schedule-pill:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.publish-now-btn {
		background: transparent;
		border: 1.5px solid
			color-mix(in srgb, var(--color-accent) 50%, transparent);
		color: var(--color-accent);
		padding: 0 10px;
	}

	.publish-now-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-color: var(--color-accent);
	}

	.publish-now-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.schedule-trigger-wrap {
		position: relative;
	}

	.icon-tools {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-left: 8px;
		opacity: 0.7;
		transition: opacity 0.15s ease;
	}

	.icon-tools:hover {
		opacity: 1;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.icon-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.icon-btn.active {
		color: var(--color-accent);
	}

	.icon-btn.active:hover {
		color: var(--color-accent-hover);
	}

	:global(.spin-icon) {
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		:global(.spin-icon) {
			animation: none;
		}

		.icon-tools {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.icon-btn {
			min-width: 44px;
			min-height: 44px;
		}
		.cta-pill {
			min-height: 44px;
		}
	}

	@media (max-width: 640px) {
		.home-header {
			gap: 8px;
		}

		.icon-tools {
			display: none;
		}

		.cta-pill {
			padding: 0 12px;
			font-size: 12px;
			height: 34px;
		}
	}
</style>
