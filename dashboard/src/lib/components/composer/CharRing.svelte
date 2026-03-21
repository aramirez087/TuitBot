<script lang="ts">
	import { MAX_TWEET_CHARS } from "$lib/utils/tweetLength";

	let {
		current,
		max = MAX_TWEET_CHARS,
	}: {
		current: number;
		max?: number;
	} = $props();

	const RADIUS = 11;
	const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
	const WARN_THRESHOLD = 260;
	const progress = $derived(Math.min(current / max, 1.15));
	const offset = $derived(CIRCUMFERENCE * (1 - Math.min(progress, 1)));
	const remaining = $derived(max - current);
	const overLimit = $derived(current > max);
	const warning = $derived(current > WARN_THRESHOLD && !overLimit);
	const visible = $derived(current > 0);
	const showCount = $derived(current > WARN_THRESHOLD);
	const tooltip = $derived(`${current}/${max}`);

	const strokeColor = $derived(
		overLimit
			? "var(--color-danger)"
			: warning
				? "var(--color-warning)"
				: "var(--color-text-muted, var(--color-border-subtle))",
	);

	const progressOpacity = $derived(overLimit || warning ? "1" : "0.45");
	const trackOpacity = $derived(overLimit || warning ? "0.2" : "0.1");
</script>

<div
	class="char-ring"
	class:visible
	class:over={overLimit}
	class:warning
	aria-live="polite"
	aria-label={tooltip}
>
	<svg width="28" height="28" viewBox="0 0 28 28" class="ring-svg">
		<!-- Track -->
		<circle
			cx="14"
			cy="14"
			r={RADIUS}
			fill="none"
			stroke="var(--color-border-subtle)"
			stroke-width="2"
			opacity={trackOpacity}
		/>
		<!-- Progress arc -->
		<circle
			cx="14"
			cy="14"
			r={RADIUS}
			fill="none"
			stroke={strokeColor}
			stroke-width="2"
			stroke-dasharray={CIRCUMFERENCE}
			stroke-dashoffset={offset}
			stroke-linecap="round"
			transform="rotate(-90 14 14)"
			opacity={progressOpacity}
			class="ring-progress"
		/>
	</svg>
	{#if showCount}
		<span
			class="ring-count"
			class:over={overLimit}
			class:warning
			style:color={strokeColor}
		>
			{remaining}
		</span>
	{/if}
	<span class="ring-tooltip">{tooltip}</span>
</div>

<style>
	.char-ring {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		opacity: 0;
		transition: opacity 0.2s ease;
		flex-shrink: 0;
		cursor: default;
	}

	.char-ring.visible {
		opacity: 1;
	}

	.ring-svg {
		display: block;
	}

	.ring-progress {
		transition:
			stroke-dashoffset 0.15s ease,
			stroke 0.2s ease,
			opacity 0.2s ease;
	}

	.char-ring.over .ring-progress {
		animation: ring-pulse 1s ease-in-out infinite;
	}

	.ring-count {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 9px;
		font-family: var(--font-mono);
		font-weight: 700;
		line-height: 1;
		pointer-events: none;
		letter-spacing: -0.02em;
	}

	.ring-count.over {
		font-size: 9px;
	}

	/* CSS tooltip */
	.ring-tooltip {
		position: absolute;
		bottom: calc(100% + 6px);
		left: 50%;
		transform: translateX(-50%);
		white-space: nowrap;
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text);
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		padding: 3px 8px;
		border-radius: 5px;
		pointer-events: none;
		opacity: 0;
		transition: opacity 0.12s ease;
	}

	.char-ring:hover .ring-tooltip {
		opacity: 1;
	}

	@keyframes ring-pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.char-ring,
		.ring-progress {
			transition: none;
		}
		.char-ring.over .ring-progress {
			animation: none;
		}
	}
</style>
