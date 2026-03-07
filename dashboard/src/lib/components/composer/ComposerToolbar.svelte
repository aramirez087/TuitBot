<script lang="ts">
	import { Sparkles, Search } from "lucide-svelte";
	import { formatCombo } from "$lib/utils/shortcuts";

	let {
		mode = "tweet",
		onaiassist,
		onopenpalette,
	}: {
		mode: "tweet" | "thread";
		onaiassist: () => void;
		onopenpalette: () => void;
	} = $props();
</script>

<div class="subheader-bar" role="toolbar" aria-label="Composer actions">
	<div class="bar-left">
		<button
			class="bar-btn ai-btn"
			onclick={onaiassist}
			title="AI improve ({formatCombo('cmd+shift+j')})"
			aria-label="AI improve selection or post"
		>
			<Sparkles size={14} />
		</button>

	</div>

	<div class="bar-right">
		<button
			class="shortcut-trigger"
			onclick={onopenpalette}
			aria-label="Command palette"
		>
			<Search size={12} />
			<kbd>{formatCombo("cmd+k")}</kbd>
			<span class="shortcut-label">All shortcuts</span>
		</button>
	</div>
</div>

<style>
	.subheader-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 20px;
		border-bottom: 1px solid
			color-mix(in srgb, var(--color-border-subtle) 35%, transparent);
		flex-shrink: 0;
		min-height: 36px;
	}

	.bar-left {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.bar-right {
		display: flex;
		align-items: center;
	}

	.bar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.12s ease;
		padding: 0;
	}

	.bar-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}

	.bar-btn.ai-btn:hover:not(:disabled) {
		color: var(--color-warning, #d29922);
		background: color-mix(
			in srgb,
			var(--color-warning, #d29922) 10%,
			transparent
		);
	}

	.shortcut-trigger {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 4px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		transition: color 0.12s ease;
		opacity: 0.7;
	}

	.shortcut-trigger:hover {
		color: var(--color-text-muted);
		opacity: 1;
	}

	.shortcut-trigger kbd {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-surface-active) 60%, transparent);
		color: var(--color-text-subtle);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 500;
		border: 1px solid
			color-mix(in srgb, var(--color-border-subtle) 40%, transparent);
		line-height: 1.5;
	}

	@media (max-width: 480px) {
		.shortcut-label {
			display: none;
		}

		.subheader-bar {
			padding: 4px 12px;
		}
	}

	@media (pointer: coarse) {
		.bar-btn {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.bar-btn,
		.shortcut-trigger {
			transition: none;
		}
	}
</style>
