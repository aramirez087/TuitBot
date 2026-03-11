<script lang="ts">
	import { X, Tag } from 'lucide-svelte';
	import type { DraftSummary, ScheduledContentItem, ContentTag } from '$lib/api/types';
	import DraftTitleNotesSection from './DraftTitleNotesSection.svelte';
	import DraftTagsSection from './DraftTagsSection.svelte';
	import DraftMetadataSection from './DraftMetadataSection.svelte';
	import DraftScheduleSection from './DraftScheduleSection.svelte';
	import DraftReadyIndicator from './DraftReadyIndicator.svelte';

	let {
		draft,
		draftSummary,
		tags,
		allTags,
		prefillSchedule = null,
		timezone = 'UTC',
		preferredTimes = [],
		onupdatemeta,
		onassigntag,
		onunassigntag,
		oncreatetag,
		onschedule,
		onunschedule,
		onreschedule,
		onduplicate,
		onclose
	}: {
		draft: ScheduledContentItem | null;
		draftSummary: DraftSummary | null;
		tags: ContentTag[];
		allTags: ContentTag[];
		prefillSchedule?: string | null;
		timezone?: string;
		preferredTimes?: string[];
		onupdatemeta: (data: { title?: string; notes?: string }) => void;
		onassigntag: (tagId: number) => void;
		onunassigntag: (tagId: number) => void;
		oncreatetag: (name: string) => void;
		onschedule: (scheduledFor: string) => void;
		onunschedule: () => void;
		onreschedule: (scheduledFor: string) => void;
		onduplicate: () => void;
		onclose: () => void;
	} = $props();
</script>

{#if draftSummary}
	<div class="details-panel">
		<div class="panel-header">
			<span class="panel-title">Details</span>
			<button class="close-btn" type="button" onclick={onclose} title="Close details (Cmd+Shift+D)">
				<X size={14} />
			</button>
		</div>

		<div class="panel-body">
			<DraftTitleNotesSection {draft} {onupdatemeta} />

			<DraftTagsSection {tags} {allTags} {onassigntag} {onunassigntag} {oncreatetag} />

			<DraftMetadataSection {draftSummary} {timezone} />

			<DraftScheduleSection
				{draftSummary}
				{prefillSchedule}
				{timezone}
				{preferredTimes}
				{onschedule}
				{onunschedule}
				{onreschedule}
				{onduplicate}
			/>

			<DraftReadyIndicator {draftSummary} />
		</div>
	</div>
{:else}
	<div class="details-panel details-empty">
		<div class="panel-header">
			<span class="panel-title">Details</span>
			<button class="close-btn" type="button" onclick={onclose} title="Close details">
				<X size={14} />
			</button>
		</div>
		<div class="empty-msg">
			<Tag size={20} />
			<p>Select a draft to see details</p>
		</div>
	</div>
{/if}

<style>
	.details-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-surface);
		border-left: 1px solid var(--color-border-subtle);
		overflow-y: auto;
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 14px;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.panel-title {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		padding: 0;
	}

	.close-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.panel-body {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 14px;
	}

	.details-empty {
		justify-content: flex-start;
	}

	.empty-msg {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 40px 20px;
		color: var(--color-text-subtle);
	}

	.empty-msg p {
		font-size: 12px;
		margin: 0;
	}
</style>
