<script lang="ts">
	import type { SyncStatus } from '$lib/utils/composerAutosave';

	let {
		status,
		onresolveconflict
	}: {
		status: SyncStatus;
		onresolveconflict?: (resolution: 'use-mine' | 'reload-server') => void;
	} = $props();

	const labels: Record<SyncStatus, string> = {
		saved: 'Saved',
		saving: 'Saving\u2026',
		unsaved: 'Unsaved',
		offline: 'Offline',
		conflict: 'Conflict'
	};
</script>

<div class="sync-badge" data-status={status} role="status" aria-live="polite">
	{#if status === 'saving'}
		<span class="sync-icon spinner" aria-hidden="true"></span>
	{:else if status === 'saved'}
		<svg class="sync-icon" width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
			<path d="M3 7.5L5.5 10L11 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
		</svg>
	{:else if status === 'unsaved'}
		<span class="sync-dot" aria-hidden="true"></span>
	{:else if status === 'offline'}
		<svg class="sync-icon" width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
			<path d="M2 2L12 12M3.5 5.5C4.5 4.5 6 3.8 7 3.8C8.5 3.8 9.8 4.5 10.8 5.5M5 7.5C5.7 6.8 6.3 6.5 7 6.5C7.7 6.5 8.3 6.8 9 7.5M6 9.5C6.3 9.2 6.6 9 7 9C7.4 9 7.7 9.2 8 9.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
		</svg>
	{:else if status === 'conflict'}
		<svg class="sync-icon" width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
			<path d="M7 4V8M7 10V10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			<circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.2"/>
		</svg>
	{/if}

	<span class="sync-label">{labels[status]}</span>

	{#if status === 'conflict' && onresolveconflict}
		<span class="conflict-sep" aria-hidden="true">&mdash;</span>
		<button type="button" class="conflict-btn" onclick={() => onresolveconflict?.('use-mine')}>
			Use mine
		</button>
		<button type="button" class="conflict-btn" onclick={() => onresolveconflict?.('reload-server')}>
			Reload
		</button>
	{/if}
</div>

<style>
	.sync-badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: 12px;
		line-height: 1;
		padding: 4px 8px;
		border-radius: 4px;
		user-select: none;
		transition: opacity 0.2s ease;
	}

	.sync-badge[data-status="saved"] {
		color: var(--color-text-subtle);
	}

	.sync-badge[data-status="saving"] {
		color: var(--color-text-subtle);
	}

	.sync-badge[data-status="unsaved"] {
		color: var(--color-warning, #d29922);
	}

	.sync-badge[data-status="offline"] {
		color: var(--color-warning, #d29922);
	}

	.sync-badge[data-status="conflict"] {
		color: var(--color-danger, #d93025);
		background: color-mix(in srgb, var(--color-danger, #d93025) 8%, transparent);
	}

	.sync-icon {
		flex-shrink: 0;
	}

	.spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 1.5px solid currentColor;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	.sync-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}

	.sync-label {
		font-weight: 500;
	}

	.conflict-sep {
		color: var(--color-text-subtle);
		font-size: 10px;
	}

	.conflict-btn {
		border: none;
		background: transparent;
		color: var(--color-danger, #d93025);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		text-decoration: underline;
		padding: 0 2px;
	}

	.conflict-btn:hover {
		opacity: 0.8;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	@media (prefers-reduced-motion: reduce) {
		.spinner {
			animation: none;
			opacity: 0.6;
		}
	}
</style>
