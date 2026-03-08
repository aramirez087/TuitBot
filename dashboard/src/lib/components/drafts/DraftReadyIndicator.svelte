<script lang="ts">
	import type { DraftSummary } from '$lib/api/types';

	let {
		draftSummary
	}: {
		draftSummary: DraftSummary;
	} = $props();

	const isReady = $derived(
		(draftSummary.content_preview?.trim().length ?? 0) > 10 &&
			(draftSummary.title !== null || draftSummary.content_preview.length > 20)
	);
</script>

<div class="ready-section">
	<span class="ready-dot" class:ready={isReady}></span>
	<span class="ready-label">{isReady ? 'Ready' : 'Not ready'}</span>
</div>

<style>
	.ready-section {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 0;
	}

	.ready-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-warning, #d29922);
		flex-shrink: 0;
	}

	.ready-dot.ready {
		background: var(--color-success, #2ea043);
	}

	.ready-label {
		font-size: 12px;
		color: var(--color-text-muted);
	}
</style>
