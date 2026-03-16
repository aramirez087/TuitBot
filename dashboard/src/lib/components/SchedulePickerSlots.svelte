<script lang="ts">
	import { Zap, X } from 'lucide-svelte';

	interface Props {
		preferredTimes: string[];
		selectedTime: string | null;
		hasSelection: boolean;
		status: 'draft' | 'scheduled' | 'posted';
		compact?: boolean;
		onselecttime: (time: string) => void;
		onquickslot: () => void;
		onunschedule?: () => void;
	}

	const {
		preferredTimes,
		selectedTime,
		hasSelection,
		status,
		compact = false,
		onselecttime,
		onquickslot,
		onunschedule,
	}: Props = $props();
</script>

{#if preferredTimes.length > 0}
	<div class="preferred-slots" class:compact>
		{#each preferredTimes as time}
			<button
				class="slot-pill"
				class:active={selectedTime === time}
				type="button"
				onclick={() => onselecttime(time)}
				aria-label="Schedule at {time}"
			>
				{time}
			</button>
		{/each}
	</div>
{/if}

<div class="quick-actions" class:compact>
	<button
		class="quick-btn"
		type="button"
		onclick={onquickslot}
		aria-label="Schedule for next free slot"
	>
		<Zap size={12} />
		Next free slot
	</button>
	{#if hasSelection && status !== 'scheduled'}
		<button
			class="quick-btn clear"
			type="button"
			onclick={onunschedule}
			aria-label="Clear schedule"
		>
			<X size={12} />
			Clear
		</button>
	{/if}
</div>

<style>
	.preferred-slots {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
	}

	.slot-pill {
		padding: 4px 12px;
		border-radius: 14px;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-mono);
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.slot-pill:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}

	.slot-pill.active {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.quick-actions {
		display: flex;
		gap: 6px;
	}

	.quick-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.quick-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.quick-btn.clear:hover {
		border-color: var(--color-danger);
		color: var(--color-danger);
	}

	.compact .slot-pill {
		padding: 3px 10px;
		font-size: 11px;
	}

	@media (prefers-reduced-motion: reduce) {
		.slot-pill,
		.quick-btn {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.slot-pill,
		.quick-btn {
			min-height: 44px;
		}
	}
</style>
