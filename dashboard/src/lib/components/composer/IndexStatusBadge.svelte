<script lang="ts">
	import type { IndexStatusResponse } from '$lib/api/types';

	let {
		status,
		compact = false,
	}: {
		status: IndexStatusResponse | null;
		compact?: boolean;
	} = $props();

	const badgeState = $derived.by(() => {
		if (!status) return { color: 'gray', label: 'Loading index status...' };
		if (!status.provider_configured) return { color: 'gray', label: 'No embedding provider configured' };
		if (!status.index_loaded && status.total_chunks === 0) return { color: 'gray', label: 'No index — vault not yet indexed' };
		if (status.freshness_pct >= 95) return { color: 'green', label: `Index fresh (${status.freshness_pct}%)` };
		if (status.freshness_pct >= 50) return { color: 'amber', label: `Index partially stale (${status.freshness_pct}% fresh)` };
		return { color: 'red', label: `Index stale (${status.freshness_pct}% fresh)` };
	});

	let showPopover = $state(false);

	function togglePopover() {
		if (compact) return;
		showPopover = !showPopover;
	}
</script>

<div class="index-status-badge" class:compact>
	<button
		class="badge-dot-btn"
		class:green={badgeState.color === 'green'}
		class:amber={badgeState.color === 'amber'}
		class:red={badgeState.color === 'red'}
		class:gray={badgeState.color === 'gray'}
		title={badgeState.label}
		aria-label={badgeState.label}
		onclick={togglePopover}
	>
		<span class="badge-dot" class:pulse={badgeState.color === 'amber'}></span>
	</button>

	{#if showPopover && status}
		<div class="badge-popover" role="tooltip">
			<div class="popover-row">
				<span class="popover-label">Status</span>
				<span class="popover-value">{badgeState.label}</span>
			</div>
			<div class="popover-row">
				<span class="popover-label">Chunks</span>
				<span class="popover-value">{status.embedded_chunks} / {status.total_chunks}</span>
			</div>
			{#if status.model_id}
				<div class="popover-row">
					<span class="popover-label">Model</span>
					<span class="popover-value">{status.model_id}</span>
				</div>
			{/if}
			{#if status.last_indexed_at}
				<div class="popover-row">
					<span class="popover-label">Last indexed</span>
					<span class="popover-value">{new Date(status.last_indexed_at).toLocaleString()}</span>
				</div>
			{/if}
			<button class="popover-reindex-btn" disabled title="Reindex available in a future update">
				Reindex Now
			</button>
		</div>
	{/if}
</div>

<style>
	.index-status-badge {
		position: relative;
		display: inline-flex;
		align-items: center;
	}

	.badge-dot-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 2px;
		border: none;
		background: none;
		cursor: pointer;
		border-radius: 50%;
	}

	.badge-dot-btn:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.badge-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		display: block;
	}

	.green .badge-dot { background: #22c55e; }
	.amber .badge-dot { background: #f59e0b; }
	.red .badge-dot { background: #ef4444; }
	.gray .badge-dot { background: var(--color-text-muted); opacity: 0.5; }

	.pulse {
		animation: dot-pulse 1.5s ease-in-out infinite;
	}

	@keyframes dot-pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}

	.badge-popover {
		position: absolute;
		top: calc(100% + 6px);
		right: 0;
		min-width: 220px;
		padding: 10px 12px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
		z-index: 10;
		font-size: 11px;
	}

	.popover-row {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		padding: 3px 0;
	}

	.popover-label {
		color: var(--color-text-muted);
		font-weight: 500;
	}

	.popover-value {
		color: var(--color-text);
		text-align: right;
	}

	.popover-reindex-btn {
		margin-top: 8px;
		width: 100%;
		padding: 5px 8px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: not-allowed;
		opacity: 0.5;
	}

	@media (prefers-reduced-motion: reduce) {
		.pulse { animation: none; }
	}
</style>
