<script lang="ts">
	import { Search, Send, Scissors, Sparkles, List, MessageSquare } from 'lucide-svelte';
	import { formatCombo } from '$lib/utils/shortcuts';

	let {
		mode = 'tweet',
		onopenpalette,
		onswitchmode
	}: {
		mode: 'tweet' | 'thread';
		onopenpalette: () => void;
		onswitchmode: () => void;
	} = $props();

	const primaryAction = $derived(
		mode === 'thread'
			? { icon: Scissors, label: 'Split', combo: 'cmd+enter' }
			: { icon: Send, label: 'Publish', combo: 'cmd+enter' }
	);

	const PrimaryIcon = $derived(primaryAction.icon);

	const SwitchIcon = $derived(mode === 'tweet' ? List : MessageSquare);
	const switchLabel = $derived(mode === 'tweet' ? 'Thread' : 'Tweet');
</script>

<div class="shortcut-bar" role="region" aria-label="Keyboard shortcuts">
	<div class="bar-hints">
		<span class="hint">
			<span class="hint-icon"><PrimaryIcon size={12} /></span>
			<kbd>{formatCombo(primaryAction.combo)}</kbd>
			<span class="hint-label">{primaryAction.label}</span>
		</span>
		<button class="hint mode-hint" onclick={onswitchmode} title="Switch to {switchLabel.toLowerCase()} mode">
			<span class="hint-icon"><SwitchIcon size={12} /></span>
			<span class="hint-label">{switchLabel}</span>
		</button>
		<span class="hint ai-hint">
			<span class="hint-icon"><Sparkles size={12} /></span>
			<kbd>{formatCombo('cmd+shift+j')}</kbd>
			<span class="hint-label">AI</span>
		</span>
	</div>
	<button class="palette-btn" onclick={onopenpalette}>
		<Search size={12} />
		<kbd>{formatCombo('cmd+k')}</kbd>
		<span>All shortcuts</span>
	</button>
</div>

<style>
	.shortcut-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 16px;
		background: color-mix(in srgb, var(--color-accent) 3%, transparent);
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
		flex-shrink: 0;
		min-height: 28px;
	}

	.bar-hints {
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.hint {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--color-text-muted);
		font-size: 11px;
	}

	.hint-icon {
		display: flex;
		align-items: center;
		color: color-mix(in srgb, var(--color-accent) 60%, transparent);
		flex-shrink: 0;
	}

	.hint-label {
		opacity: 0.7;
	}

	.mode-hint {
		border: none;
		background: none;
		padding: 2px 4px;
		border-radius: 4px;
		cursor: pointer;
		transition: background 0.12s ease;
	}

	.mode-hint:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.mode-hint:hover .hint-label {
		opacity: 1;
		color: var(--color-accent);
	}

	kbd {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 500;
		border: 1px solid color-mix(in srgb, var(--color-accent) 12%, transparent);
		line-height: 1.5;
	}

	.palette-btn {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 2px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		cursor: pointer;
		transition: color 0.12s ease;
	}

	.palette-btn:hover {
		color: var(--color-accent);
	}

	.palette-btn kbd {
		font-size: 9px;
	}

	@media (max-width: 480px) {
		.ai-hint {
			display: none;
		}

		.palette-btn span {
			display: none;
		}
	}
</style>
