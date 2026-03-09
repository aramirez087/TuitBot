<script lang="ts">
	import { Copy, Trash2, RotateCcw } from "lucide-svelte";
	import type { DraftSummary } from "$lib/api/types";

	type TabKey = "active" | "scheduled" | "posted" | "archive";

	let {
		draft,
		selected,
		focused,
		tabindex,
		tab,
		onselect,
		ondelete,
		onduplicate,
		onrestore,
	}: {
		draft: DraftSummary;
		selected: boolean;
		focused: boolean;
		tabindex: number;
		tab: TabKey;
		onselect: () => void;
		ondelete: () => void;
		onduplicate: () => void;
		onrestore: () => void;
	} = $props();

	function relativeTime(dateStr: string): string {
		const now = Date.now();
		const then = new Date(dateStr).getTime();
		const diffMs = now - then;
		const diffSec = Math.floor(diffMs / 1000);
		if (diffSec < 60) return "now";
		const diffMin = Math.floor(diffSec / 60);
		if (diffMin < 60) return `${diffMin}m`;
		const diffHr = Math.floor(diffMin / 60);
		if (diffHr < 24) return `${diffHr}h`;
		const diffDays = Math.floor(diffHr / 24);
		if (diffDays === 1) return "yesterday";
		if (diffDays < 7) return `${diffDays}d`;
		const d = new Date(dateStr);
		return d.toLocaleDateString("en-US", {
			month: "short",
			day: "numeric",
		});
	}

	const displayTitle = $derived(
		draft.title ?? (draft.content_preview?.trim() || "Untitled draft"),
	);

	let rootEl: HTMLDivElement | undefined = $state();

	function stopProp(fn: () => void) {
		return (e: MouseEvent) => {
			e.stopPropagation();
			fn();
		};
	}

	export function focus() {
		rootEl?.focus();
	}

	export function scrollIntoViewIfNeeded() {
		rootEl?.scrollIntoView({ block: "nearest" });
	}
</script>

<div
	bind:this={rootEl}
	class="rail-item"
	class:selected
	class:focused
	role="option"
	aria-selected={selected}
	{tabindex}
	onclick={onselect}
	onkeydown={(e) => {
		if (e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			onselect();
		}
	}}
>
	<div class="item-row">
		<span
			class="item-dot"
			class:has-content={(draft.content_preview?.trim().length ?? 0) > 10}
		></span>
		<span class="item-title">{displayTitle}</span>
		<span class="item-time">{relativeTime(draft.updated_at)}</span>
	</div>

	<div
		class="item-actions"
		data-rail-actions
		role="group"
		aria-label="Draft actions"
	>
		{#if tab === "archive"}
			<button
				class="action-btn"
				title="Restore (R)"
				type="button"
				onclick={stopProp(onrestore)}
			>
				<RotateCcw size={12} />
			</button>
		{:else}
			<button
				class="action-btn"
				title="Duplicate (D)"
				type="button"
				onclick={stopProp(onduplicate)}
			>
				<Copy size={12} />
			</button>
			<button
				class="action-btn action-btn--danger"
				title="Delete (Del)"
				type="button"
				onclick={stopProp(() => {
					const label =
						draft.title ??
						draft.content_preview?.trim() ??
						"this draft";
					const truncated =
						label.length > 60 ? label.slice(0, 60) + "…" : label;
					if (
						window.confirm(
							`Delete "${truncated}"? This cannot be undone.`,
						)
					) {
						ondelete();
					}
				})}
			>
				<Trash2 size={12} />
			</button>
		{/if}
	</div>
</div>

<style>
	.rail-item {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 7px 10px;
		border-radius: 6px;
		background: transparent;
		cursor: pointer;
		text-align: left;
		transition: background-color 0.12s ease;
		outline: none;
		position: relative;
	}

	.rail-item:hover {
		background: var(--color-surface-hover);
	}

	.rail-item.selected {
		background: var(--color-surface-active);
	}

	.rail-item.focused {
		box-shadow: inset 0 0 0 1.5px var(--color-accent);
	}

	.item-row {
		display: flex;
		align-items: center;
		gap: 7px;
		flex: 1;
		min-width: 0;
	}

	.item-dot {
		flex-shrink: 0;
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--color-text-subtle);
		opacity: 0.35;
		transition:
			background 0.15s,
			opacity 0.15s;
	}

	.item-dot.has-content {
		background: var(--color-success, #2ea043);
		opacity: 0.8;
	}

	.item-title {
		flex: 1;
		font-size: 13px;
		font-weight: 450;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.item-time {
		flex-shrink: 0;
		font-size: 11px;
		color: var(--color-text-subtle);
		font-variant-numeric: tabular-nums;
	}

	.item-actions {
		display: flex;
		gap: 1px;
		opacity: 0;
		flex-shrink: 0;
		transition: opacity 0.12s ease;
	}

	.rail-item:hover .item-actions,
	.rail-item.focused .item-actions {
		opacity: 1;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.1s ease;
		padding: 0;
	}

	.action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.action-btn--danger:hover {
		background: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}
</style>
