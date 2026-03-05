<script lang="ts">
	import { Sparkles, MessageSquare, List, Search } from "lucide-svelte";
	import { formatCombo } from "$lib/utils/shortcuts";

	let {
		mode = "tweet",
		onaiassist,
		onswitchmode,
		onopenpalette,
	}: {
		mode: "tweet" | "thread";
		onaiassist: () => void;
		onswitchmode: () => void;
		onopenpalette: () => void;
	} = $props();

	const ModeIcon = $derived(mode === "tweet" ? MessageSquare : List);
	const modeLabel = $derived(mode === "tweet" ? "Tweet" : "Thread");
	const switchLabel = $derived(
		mode === "tweet" ? "Switch to thread" : "Switch to tweet",
	);
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

		<span class="bar-divider" aria-hidden="true"></span>

		<button
			class="mode-tab"
			onclick={onswitchmode}
			title={switchLabel}
			aria-label={switchLabel}
		>
			<ModeIcon size={13} />
			<span class="mode-label">{modeLabel}</span>
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

	.bar-divider {
		width: 1px;
		height: 16px;
		background: color-mix(
			in srgb,
			var(--color-border-subtle) 50%,
			transparent
		);
		margin: 0 6px;
	}

	.mode-tab {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.mode-tab:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}

	.mode-label {
		letter-spacing: 0.01em;
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

		.mode-label {
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

		.mode-tab {
			min-height: 36px;
			padding: 4px 12px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.bar-btn,
		.mode-tab,
		.shortcut-trigger {
			transition: none;
		}
	}
</style>
