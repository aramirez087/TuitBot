<script lang="ts">
	import { GripVertical, Merge, Trash2, Image, ChevronUp, ChevronDown } from 'lucide-svelte';
	import CharRing from './CharRing.svelte';

	interface Props {
		index: number;
		total: number;
		charCount: number;
		words: number;
		isFirst: boolean;
		isLast: boolean;
		onMediaAttach: () => void;
		onMoveUp?: () => void;
		onMoveDown?: () => void;
		onMerge: () => void;
		onRemove: () => void;
		onDragStart: (e: DragEvent) => void;
		onDragEnd: () => void;
	}

	const {
		index,
		total,
		charCount,
		words,
		isFirst,
		isLast,
		onMediaAttach,
		onMoveUp,
		onMoveDown,
		onMerge,
		onRemove,
		onDragStart,
		onDragEnd,
	}: Props = $props();
</script>

<div class="card-footer">
	<div class="footer-left">
		<span class="footer-badge">
			#{index + 1}{#if words > 0}&middot; {words} {words === 1 ? 'word' : 'words'}{/if}
		</span>
	</div>
	<div class="footer-actions">
		<CharRing current={charCount} />
		<div class="footer-action-group">
			<button
				class="footer-action-btn"
				onclick={onMediaAttach}
				title="Attach media"
				aria-label={`Attach media to post ${index + 1}`}
			>
				<Image size={13} />
			</button>
			{#if onMoveUp && !isFirst}
				<button
					class="footer-action-btn"
					onclick={() => onMoveUp?.()}
					title="Move up (Alt+↑)"
					aria-label={`Move post ${index + 1} up`}
				>
					<ChevronUp size={13} />
				</button>
			{/if}
			{#if onMoveDown && !isLast}
				<button
					class="footer-action-btn"
					onclick={() => onMoveDown?.()}
					title="Move down (Alt+↓)"
					aria-label={`Move post ${index + 1} down`}
				>
					<ChevronDown size={13} />
				</button>
			{/if}
			<div
				class="footer-handle"
				draggable="true"
				role="button"
				tabindex="-1"
				title="Drag to reorder"
				aria-label={`Reorder post ${index + 1}. Use Alt+Up or Alt+Down to move.`}
				ondragstart={onDragStart}
				ondragend={onDragEnd}
			>
				<GripVertical size={13} />
			</div>
			{#if total > 1 && !isLast}
				<button
					class="footer-action-btn"
					onclick={onMerge}
					title="Merge with next (⌘⇧M)"
					aria-label={`Merge post ${index + 1} with post ${index + 2}`}
				>
					<Merge size={13} />
				</button>
			{/if}
			<button
				class="footer-action-btn footer-remove"
				onclick={onRemove}
				title="Remove post"
				aria-label={`Remove post ${index + 1}`}
			>
				<Trash2 size={13} />
			</button>
		</div>
	</div>
</div>

<style>
	.card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 0 0;
		margin-top: 6px;
		min-height: 28px;
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	/* Show footer on hover/focus — parent sets class via :global or focus-within */
	:global(.flow-card:hover) .card-footer,
	:global(.flow-card.focused) .card-footer,
	:global(.flow-card:focus-within) .card-footer {
		opacity: 1;
	}

	.footer-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.footer-badge {
		font-size: 10px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		opacity: 0.6;
		padding: 1px 6px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-surface-active) 50%, transparent);
		letter-spacing: 0.02em;
	}

	.footer-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.footer-action-group {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.footer-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 0;
		transition:
			color 0.15s ease,
			background 0.15s ease;
	}

	.footer-action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.footer-remove:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	.footer-handle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		color: var(--color-text-subtle);
		cursor: grab;
		border: none;
		background: none;
		padding: 0;
		border-radius: 4px;
		transition:
			color 0.15s ease,
			background 0.15s ease;
	}

	.footer-handle:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}

	.footer-handle:active {
		cursor: grabbing;
	}

	/* Touch: always show tools */
	@media (hover: none) {
		.card-footer {
			opacity: 0.7;
		}
	}

	@media (pointer: coarse) {
		.footer-action-btn,
		.footer-handle {
			min-width: 44px;
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.card-footer,
		.footer-action-btn,
		.footer-handle {
			transition: none;
		}
	}
</style>
