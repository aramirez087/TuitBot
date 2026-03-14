<script lang="ts">
	import { ChevronLeft, ChevronRight, Calendar, LayoutGrid } from 'lucide-svelte';

	interface Props {
		headerLabel: string;
		viewMode: 'week' | 'month';
		onprev: () => void;
		onnext: () => void;
		ontoday: () => void;
		onsetview: (mode: 'week' | 'month') => void;
	}

	const { headerLabel, viewMode, onprev, onnext, ontoday, onsetview }: Props = $props();
</script>

<div class="calendar-controls">
	<div class="nav-group">
		<button class="nav-btn" onclick={onprev} aria-label="Previous period">
			<ChevronLeft size={16} />
		</button>
		<button class="today-btn" onclick={ontoday}>Today</button>
		<button class="nav-btn" onclick={onnext} aria-label="Next period">
			<ChevronRight size={16} />
		</button>
		<span class="period-label">{headerLabel}</span>
	</div>

	<div class="view-toggle" role="group" aria-label="Calendar view">
		<button
			class="view-btn"
			class:active={viewMode === 'week'}
			onclick={() => onsetview('week')}
			aria-pressed={viewMode === 'week'}
		>
			<Calendar size={14} />
			Week
		</button>
		<button
			class="view-btn"
			class:active={viewMode === 'month'}
			onclick={() => onsetview('month')}
			aria-pressed={viewMode === 'month'}
		>
			<LayoutGrid size={14} />
			Month
		</button>
	</div>
</div>

<style>
	.calendar-controls {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
		flex-wrap: wrap;
		gap: 12px;
	}

	.nav-group {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.nav-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.today-btn {
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.today-btn:hover {
		background: var(--color-surface-hover);
	}

	.period-label {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin-left: 8px;
	}

	.view-toggle {
		display: flex;
		gap: 0;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		overflow: hidden;
	}

	.view-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 12px;
		border: none;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.view-btn:first-child {
		border-right: 1px solid var(--color-border);
	}

	.view-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.view-btn.active {
		background: var(--color-surface);
		color: var(--color-accent);
	}
</style>
