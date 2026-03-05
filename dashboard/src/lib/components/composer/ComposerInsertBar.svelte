<script lang="ts">
	import { Smile, Hash, AtSign, ListOrdered, Quote, Lightbulb } from "lucide-svelte";

	let {
		oninsert,
	}: {
		oninsert: (text: string) => void;
	} = $props();

	let emojiOpen = $state(false);
	let emojiAnchor: HTMLButtonElement | undefined = $state();

	const EMOJI_GRID = [
		"😀", "😂", "🥹", "😍", "🤔", "😎", "🥳", "🤩",
		"👍", "👎", "👏", "🙌", "🤝", "💪", "🫡", "🔥",
		"❤️", "💯", "✨", "⭐", "🚀", "💡", "🎯", "🏆",
		"✅", "❌", "⚡", "🎉", "📈", "📉", "🧵", "💬",
		"👀", "🤷", "🫠", "😤", "📝", "🗣️", "💎", "🌟",
	];

	function toggleEmoji() {
		emojiOpen = !emojiOpen;
	}

	function pickEmoji(emoji: string) {
		oninsert(emoji);
		emojiOpen = false;
	}

	function handleClickOutside(e: MouseEvent) {
		if (!emojiOpen) return;
		const target = e.target as HTMLElement;
		if (emojiAnchor?.contains(target)) return;
		const popover = document.querySelector(".emoji-popover");
		if (popover?.contains(target)) return;
		emojiOpen = false;
	}
</script>

<svelte:window onclick={handleClickOutside} />

<div class="insert-bar" role="toolbar" aria-label="Insert formatting">
	<div class="insert-group">
		<div class="emoji-wrapper">
			<button
				bind:this={emojiAnchor}
				class="insert-btn"
				class:active={emojiOpen}
				onclick={toggleEmoji}
				title="Insert emoji"
				aria-label="Insert emoji"
				aria-expanded={emojiOpen}
			>
				<Smile size={15} />
			</button>
			{#if emojiOpen}
				<div class="emoji-popover" role="group" aria-label="Emoji picker">
					{#each EMOJI_GRID as emoji}
						<button
							class="emoji-cell"
							onclick={() => pickEmoji(emoji)}
							aria-label={emoji}
						>{emoji}</button>
					{/each}
				</div>
			{/if}
		</div>

		<button
			class="insert-btn"
			onclick={() => oninsert("#")}
			title="Insert hashtag"
			aria-label="Insert hashtag"
		>
			<Hash size={15} />
		</button>

		<button
			class="insert-btn"
			onclick={() => oninsert("@")}
			title="Insert mention"
			aria-label="Insert mention"
		>
			<AtSign size={15} />
		</button>

		<span class="insert-divider" aria-hidden="true"></span>

		<button
			class="insert-btn"
			onclick={() => oninsert("\n- ")}
			title="Insert list item"
			aria-label="Insert list item"
		>
			<ListOrdered size={15} />
		</button>

		<button
			class="insert-btn"
			onclick={() => oninsert("\u201C\u201D")}
			title="Insert quotes"
			aria-label="Insert quote marks"
		>
			<Quote size={15} />
		</button>

		<button
			class="insert-btn"
			onclick={() => oninsert("\u2728 ")}
			title="Insert sparkle"
			aria-label="Insert sparkle"
		>
			<Lightbulb size={15} />
		</button>
	</div>
</div>

<style>
	.insert-bar {
		display: flex;
		align-items: center;
		padding: 6px 0;
		margin-top: 4px;
	}

	.insert-group {
		display: flex;
		align-items: center;
		gap: 1px;
	}

	.insert-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.12s ease;
		padding: 0;
		opacity: 0.6;
	}

	.insert-btn:hover {
		opacity: 1;
		color: var(--color-text-muted);
		background: var(--color-surface-hover);
	}

	.insert-btn.active {
		opacity: 1;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.insert-divider {
		width: 1px;
		height: 16px;
		background: color-mix(
			in srgb,
			var(--color-border-subtle) 35%,
			transparent
		);
		margin: 0 4px;
	}

	/* ── Emoji Popover ──────────────── */
	.emoji-wrapper {
		position: relative;
	}

	.emoji-popover {
		position: absolute;
		bottom: calc(100% + 8px);
		left: 0;
		display: grid;
		grid-template-columns: repeat(8, 1fr);
		gap: 2px;
		padding: 8px;
		border-radius: 10px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		box-shadow:
			0 8px 32px rgba(0, 0, 0, 0.25),
			0 2px 8px rgba(0, 0, 0, 0.1);
		z-index: 20;
		width: 280px;
	}

	.emoji-cell {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 6px;
		background: transparent;
		font-size: 18px;
		cursor: pointer;
		transition: background 0.1s ease;
		padding: 0;
		line-height: 1;
	}

	.emoji-cell:hover {
		background: var(--color-surface-hover);
	}

	@media (pointer: coarse) {
		.insert-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.emoji-cell {
			width: 40px;
			height: 40px;
		}

		.emoji-popover {
			width: 336px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.insert-btn,
		.emoji-cell {
			transition: none;
		}
	}

	@media (max-width: 480px) {
		.emoji-popover {
			grid-template-columns: repeat(6, 1fr);
			width: 216px;
		}
	}
</style>
