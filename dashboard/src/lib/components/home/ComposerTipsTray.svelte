<script lang="ts">
	import { Scissors, Search, Sparkles } from 'lucide-svelte';

	let {
		visible,
		ondismiss
	}: {
		visible: boolean;
		ondismiss: () => void;
	} = $props();

	const isMac = typeof navigator !== 'undefined' && /Mac|iPod|iPhone|iPad/.test(navigator.userAgent);
	const mod = isMac ? '\u2318' : 'Ctrl+';

	const tips = [
		{ icon: Scissors, label: 'Split into thread', shortcut: `${mod}Enter` },
		{ icon: Search, label: 'Command palette', shortcut: `${mod}K` },
		{ icon: Sparkles, label: 'AI improve', shortcut: `${mod}J` }
	];
</script>

{#if visible}
	<div class="tips-tray" role="region" aria-label="Quick tips">
		<div class="tips-list">
			{#each tips as tip}
				<div class="tip-item">
					<span class="tip-icon">
						<tip.icon size={13} />
					</span>
					<span class="tip-label">{tip.label}</span>
					<kbd class="tip-kbd">{tip.shortcut}</kbd>
				</div>
			{/each}
		</div>
		<button class="tips-dismiss" onclick={ondismiss}>Got it</button>
	</div>
{/if}

<style>
	.tips-tray {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 6px 16px;
		background: color-mix(in srgb, var(--color-accent) 4%, transparent);
		border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 8%, transparent);
		flex-shrink: 0;
	}

	.tips-list {
		display: flex;
		align-items: center;
		gap: 16px;
		flex: 1;
		min-width: 0;
		overflow-x: auto;
	}

	.tip-item {
		display: flex;
		align-items: center;
		gap: 6px;
		white-space: nowrap;
	}

	.tip-icon {
		display: flex;
		align-items: center;
		color: var(--color-accent);
		flex-shrink: 0;
	}

	.tip-label {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.tip-kbd {
		display: inline-flex;
		align-items: center;
		padding: 1px 5px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 500;
		border: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent);
		line-height: 1.4;
	}

	.tips-dismiss {
		padding: 4px 12px;
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 4px;
		background: transparent;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.15s ease;
		flex-shrink: 0;
	}

	.tips-dismiss:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	@media (max-width: 640px) {
		.tips-tray {
			gap: 8px;
		}

		.tips-list {
			gap: 12px;
		}

		.tip-label {
			font-size: 11px;
		}
	}

	@media (max-width: 480px) {
		.tip-item:last-child {
			display: none;
		}
	}
</style>
