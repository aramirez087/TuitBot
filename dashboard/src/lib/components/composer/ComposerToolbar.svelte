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

<div class="toolbar-anchor">
	<div class="floating-toolbar" role="toolbar" aria-label="Composer actions">
		<div class="toolbar-left">
			<button
				class="toolbar-btn ai-btn"
				onclick={onaiassist}
				title="AI improve ({formatCombo('cmd+shift+j')})"
				aria-label="AI improve selection or post"
			>
				<Sparkles size={15} />
			</button>

			<span class="toolbar-divider" aria-hidden="true"></span>

			<button
				class="mode-badge"
				onclick={onswitchmode}
				title={switchLabel}
				aria-label={switchLabel}
			>
				<ModeIcon size={13} />
				<span class="mode-label">{modeLabel}</span>
			</button>
		</div>

		<div class="toolbar-right">
			<button
				class="palette-trigger"
				onclick={onopenpalette}
				aria-label="Command palette"
			>
				<Search size={12} />
				<kbd>{formatCombo("cmd+k")}</kbd>
				<span class="palette-label">All shortcuts</span>
			</button>
		</div>
	</div>
</div>

<style>
	.toolbar-anchor {
		position: sticky;
		bottom: 0;
		display: flex;
		justify-content: center;
		padding: 8px 16px 12px;
		pointer-events: none;
		z-index: 5;
	}

	.floating-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 6px 14px;
		border-radius: 22px;
		background: color-mix(in srgb, var(--color-surface) 85%, transparent);
		backdrop-filter: blur(16px) saturate(180%);
		-webkit-backdrop-filter: blur(16px) saturate(180%);
		border: 1px solid
			color-mix(in srgb, var(--color-border-subtle) 40%, transparent);
		box-shadow:
			0 4px 24px rgba(0, 0, 0, 0.15),
			0 1px 4px rgba(0, 0, 0, 0.08);
		pointer-events: auto;
		width: 100%;
		max-width: 520px;
	}

	.toolbar-left {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.toolbar-right {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.toolbar-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 50%;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.12s ease;
		padding: 0;
	}

	.toolbar-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}

	.toolbar-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.toolbar-btn.ai-btn:hover:not(:disabled) {
		color: var(--color-warning, #d29922);
		background: color-mix(
			in srgb,
			var(--color-warning, #d29922) 10%,
			transparent
		);
	}

	.toolbar-divider {
		width: 1px;
		height: 16px;
		background: color-mix(
			in srgb,
			var(--color-border-subtle) 50%,
			transparent
		);
		margin: 0 4px;
	}

	.mode-badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border: 1px solid
			color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
		border-radius: 12px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.mode-badge:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
	}

	.mode-label {
		letter-spacing: 0.01em;
	}

	.palette-trigger {
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

	.palette-trigger:hover {
		color: var(--color-accent);
	}

	.palette-trigger kbd {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 500;
		border: 1px solid
			color-mix(in srgb, var(--color-accent) 12%, transparent);
		line-height: 1.5;
	}

	@media (max-width: 480px) {
		.palette-label {
			display: none;
		}

		.palette-trigger kbd {
			font-size: 9px;
		}

		.mode-label {
			display: none;
		}

		.toolbar-right {
			gap: 6px;
		}

		.floating-toolbar {
			border-radius: 16px;
			padding: 4px 10px;
		}
	}

	@media (pointer: coarse) {
		.toolbar-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.mode-badge {
			min-height: 36px;
			padding: 4px 12px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.toolbar-btn,
		.mode-badge,
		.palette-trigger {
			transition: none;
		}
	}
</style>
