<script lang="ts">
	import { onMount } from "svelte";
	import type { ThreadBlock } from "$lib/api";
	import { tweetWeightedLen, MAX_TWEET_CHARS } from "$lib/utils/tweetLength";
	import { GripVertical, Merge, Trash2, Plus } from "lucide-svelte";
	import MediaSlot from "../MediaSlot.svelte";

	let {
		block,
		index,
		total,
		avatarUrl = null,
		displayName = null,
		handle = null,
		focused = false,
		assisting = false,
		dragging = false,
		dropTarget = false,
		ontext,
		onfocus,
		onblur,
		onkeydown,
		onmedia,
		onmerge,
		onremove,
		onaddafter,
		ondragstart,
		ondragend,
		ondragover,
		ondragenter,
		ondragleave,
		ondrop,
	}: {
		block: ThreadBlock;
		index: number;
		total: number;
		avatarUrl?: string | null;
		displayName?: string | null;
		handle?: string | null;
		focused?: boolean;
		assisting?: boolean;
		dragging?: boolean;
		dropTarget?: boolean;
		ontext: (text: string) => void;
		onfocus: () => void;
		onblur: () => void;
		onkeydown: (e: KeyboardEvent) => void;
		onmedia: (paths: string[]) => void;
		onmerge: () => void;
		onremove: () => void;
		onaddafter: () => void;
		ondragstart: (e: DragEvent) => void;
		ondragend: () => void;
		ondragover: (e: DragEvent) => void;
		ondragenter: (e: DragEvent) => void;
		ondragleave: (e: DragEvent) => void;
		ondrop: (e: DragEvent) => void;
	} = $props();

	const charCount = $derived(tweetWeightedLen(block.text));
	const overLimit = $derived(charCount > MAX_TWEET_CHARS);
	const warning = $derived(charCount > 260 && !overLimit);
	const isLast = $derived(index >= total - 1);
	const isFirst = $derived(index === 0);

	function handleKeydownGuarded(e: KeyboardEvent) {
		if (e.isComposing) return;
		onkeydown(e);
	}

	function autoResize(el: HTMLTextAreaElement) {
		el.style.height = "auto";
		el.style.height = el.scrollHeight + "px";
	}

	function handleInput(e: Event) {
		const textarea = e.currentTarget as HTMLTextAreaElement;
		ontext(textarea.value);
		autoResize(textarea);
	}

	let textareaEl: HTMLTextAreaElement | undefined = $state();

	onMount(() => {
		if (textareaEl && block.text) {
			autoResize(textareaEl);
		}
	});

	$effect(() => {
		void block.text;
		if (textareaEl) {
			requestAnimationFrame(() => {
				if (textareaEl) autoResize(textareaEl);
			});
		}
	});
</script>

<div
	class="flow-card"
	class:dragging
	class:drop-target={dropTarget}
	class:assisting
	class:focused
	data-block-id={block.id}
	role="listitem"
	ondragover={(e) => ondragover(e)}
	ondragenter={(e) => ondragenter(e)}
	ondragleave={(e) => ondragleave(e)}
	ondrop={(e) => ondrop(e)}
>
	<!-- Gutter: avatar + spine -->
	<div class="card-gutter">
		{#if avatarUrl}
			<img
				src={avatarUrl}
				alt=""
				class="gutter-avatar"
				class:over-limit={overLimit}
			/>
		{:else}
			<div
				class="gutter-avatar-placeholder"
				class:over-limit={overLimit}
			></div>
		{/if}
		{#if !isLast}
			<div class="gutter-spine"></div>
		{/if}
	</div>

	<!-- Main content area -->
	<div class="card-body">
		<div class="card-writing-area" class:over-limit={overLimit}>
			<div class="card-meta">
				{#if displayName}
					<span class="meta-name">{displayName}</span>
				{/if}
				{#if handle}
					<span class="meta-handle">@{handle}</span>
				{/if}
			</div>
			<textarea
				bind:this={textareaEl}
				class="flow-textarea"
				placeholder={isFirst ? "Start writing..." : "Continue..."}
				value={block.text}
				oninput={handleInput}
				onfocus={() => onfocus()}
				onblur={() => onblur()}
				onkeydown={handleKeydownGuarded}
				aria-label={`Post ${index + 1} of ${total}`}
			></textarea>
			<MediaSlot
				mediaPaths={block.media_paths}
				onmediachange={(paths) => onmedia(paths)}
			/>
		</div>

		<!-- Separator row: character count + tools -->
		<div class="card-separator" class:last={isLast}>
			<span class="sep-post-number">#{index + 1}</span>
			{#if charCount > 240 || overLimit}
				<span
					class="sep-char-count"
					class:over-limit={overLimit}
					class:warning
				>
					{charCount}/{MAX_TWEET_CHARS}
				</span>
			{/if}
			<div class="sep-tools">
				<div
					class="sep-handle"
					draggable="true"
					role="button"
					tabindex="-1"
					title="Drag to reorder"
					aria-label={`Reorder post ${index + 1}. Use Alt+Up or Alt+Down to move.`}
					ondragstart={(e) => ondragstart(e)}
					ondragend={() => ondragend()}
				>
					<GripVertical size={12} />
				</div>
				{#if total > 2}
					<button
						class="sep-action-btn"
						onclick={() => onmerge()}
						title="Merge with next ({'\u2318'}{'\u21e7'}M)"
						aria-label={`Merge post ${index + 1} with post ${index + 2}`}
					>
						<Merge size={12} />
					</button>
					<button
						class="sep-action-btn sep-remove"
						onclick={() => onremove()}
						title="Remove post"
						aria-label={`Remove post ${index + 1}`}
					>
						<Trash2 size={12} />
					</button>
				{/if}
			</div>
		</div>

		<!-- Add post below zone -->
		<button
			class="add-post-zone"
			tabindex="-1"
			aria-label={`Add post after post ${index + 1}`}
			onclick={() => onaddafter()}
		>
			<span class="add-post-icon"><Plus size={11} /></span>
			<span class="add-post-label">Add post below</span>
			<kbd class="add-post-kbd">⌘↩</kbd>
		</button>
	</div>
</div>

<style>
	.flow-card {
		position: relative;
		display: flex;
		gap: 12px;
		transition: opacity 0.15s ease;
	}

	.flow-card.dragging {
		opacity: 0.4;
	}

	.flow-card.assisting {
		opacity: 0.7;
		pointer-events: none;
	}

	/* ── Gutter (avatar + spine) ────────────── */
	.card-gutter {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 36px;
		flex-shrink: 0;
		padding-top: 4px;
	}

	.gutter-avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
		border: 2px solid transparent;
		transition: border-color 0.15s ease;
	}

	.flow-card.focused .gutter-avatar {
		border-color: var(--color-accent);
	}

	.gutter-avatar.over-limit {
		border-color: var(--color-danger);
	}

	.gutter-avatar-placeholder {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: var(--color-surface-active);
		border: 2px solid transparent;
		flex-shrink: 0;
		transition: border-color 0.15s ease;
	}

	.flow-card.focused .gutter-avatar-placeholder {
		border-color: var(--color-accent);
	}

	.gutter-avatar-placeholder.over-limit {
		border-color: var(--color-danger);
	}

	.gutter-spine {
		flex: 1;
		width: 2px;
		margin-top: 4px;
		background: color-mix(
			in srgb,
			var(--color-border-subtle) 35%,
			transparent
		);
		border-radius: 1px;
		min-height: 16px;
		transition: background 0.15s ease;
	}

	.flow-card.focused .gutter-spine {
		background: color-mix(in srgb, var(--color-accent) 30%, transparent);
	}

	/* ── Card body ──────────────────────────── */
	.card-body {
		flex: 1;
		min-width: 0;
	}

	.card-writing-area {
		position: relative;
		padding: 4px 0;
		transition: border-color 0.15s ease;
	}

	.card-writing-area.over-limit {
		border-left: 2px solid var(--color-danger);
		padding-left: 12px;
	}

	.flow-card.drop-target .card-writing-area {
		background: color-mix(in srgb, var(--color-accent) 4%, transparent);
		border-radius: 8px;
		padding-left: 12px;
		padding-right: 12px;
	}

	.card-meta {
		display: flex;
		align-items: baseline;
		gap: 5px;
		min-width: 0;
		margin-bottom: 2px;
	}

	.meta-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 160px;
		line-height: 1;
	}

	.meta-handle {
		font-size: 12px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 120px;
		line-height: 1;
	}

	.flow-textarea {
		width: 100%;
		padding: 4px 0;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 16px;
		font-family: var(--font-sans);
		line-height: 1.55;
		resize: none;
		outline: none;
		box-sizing: border-box;
		min-height: 56px;
		overflow: hidden;
	}

	.flow-textarea::placeholder {
		color: var(--color-text-subtle);
		opacity: 0.5;
	}

	/* ── Separator ──────────────────────────── */
	.card-separator {
		display: flex;
		align-items: center;
		height: 24px;
		padding: 0;
		margin: 0;
		gap: 8px;
	}

	.sep-post-number {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		opacity: 0.6;
	}

	.sep-char-count {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		min-width: 60px;
	}

	.sep-char-count.warning {
		color: var(--color-warning);
	}

	.sep-char-count.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	.sep-tools {
		display: flex;
		align-items: center;
		gap: 2px;
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	.card-separator:hover .sep-tools {
		opacity: 1;
	}

	.sep-handle {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		color: var(--color-text-subtle);
		cursor: grab;
		border: none;
		background: none;
		padding: 0;
		border-radius: 3px;
		transition:
			color 0.15s ease,
			background 0.15s ease;
	}

	.sep-handle:hover {
		color: var(--color-text);
		background: var(--color-surface-hover);
	}

	.sep-handle:active {
		cursor: grabbing;
	}

	.sep-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
		padding: 0;
	}

	.sep-action-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.sep-action-btn.sep-remove:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	/* ── Add Post Zone ──────────────────────── */
	.add-post-zone {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 6px 0;
		border: none;
		background: none;
		cursor: pointer;
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	.flow-card:hover .add-post-zone,
	.flow-card.focused .add-post-zone {
		opacity: 1;
	}

	.add-post-icon {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 1.5px solid
			color-mix(in srgb, var(--color-border-subtle) 60%, transparent);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		transition: all 0.15s ease;
		flex-shrink: 0;
	}

	.add-post-zone:hover .add-post-icon {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.add-post-label {
		font-size: 12px;
		color: var(--color-text-subtle);
		transition: color 0.15s ease;
	}

	.add-post-zone:hover .add-post-label {
		color: var(--color-accent);
	}

	.add-post-kbd {
		font-size: 10px;
		font-family: var(--font-mono);
		padding: 1px 5px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
		border: 1px solid
			color-mix(in srgb, var(--color-accent) 12%, transparent);
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	.add-post-zone:hover .add-post-kbd {
		opacity: 1;
	}

	/* Touch: always show tools */
	@media (hover: none) {
		.sep-tools {
			opacity: 1;
		}
		.add-post-zone {
			opacity: 1;
		}
		.add-post-kbd {
			opacity: 1;
		}
	}

	@media (pointer: coarse) {
		.sep-handle,
		.sep-action-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.add-post-zone {
			min-height: 44px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.flow-card,
		.card-writing-area,
		.sep-tools,
		.add-post-zone,
		.add-post-icon,
		.add-post-label,
		.add-post-kbd,
		.gutter-avatar,
		.gutter-avatar-placeholder,
		.gutter-spine,
		.sep-handle,
		.sep-action-btn {
			transition: none;
		}
	}

	@media (max-width: 640px) {
		.card-gutter {
			width: 28px;
		}

		.gutter-avatar,
		.gutter-avatar-placeholder {
			width: 28px;
			height: 28px;
		}

		.flow-card {
			gap: 8px;
		}

		.card-meta {
			display: none;
		}
	}
</style>
