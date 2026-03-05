<script lang="ts">
	let {
		altText = '',
		onchange
	}: {
		altText?: string;
		onchange: (text: string) => void;
	} = $props();

	let expanded = $state(false);
	let inputEl: HTMLInputElement | undefined = $state();

	function toggle() {
		expanded = !expanded;
		if (expanded) {
			requestAnimationFrame(() => inputEl?.focus());
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === 'Escape') {
			e.preventDefault();
			expanded = false;
		}
	}

	function handleBlur() {
		expanded = false;
	}

	const hasAlt = $derived(altText.trim().length > 0);
</script>

<div class="alt-badge-wrap">
	<button
		class="alt-badge"
		class:has-alt={hasAlt}
		onclick={toggle}
		title={hasAlt ? `Alt: ${altText}` : 'Add alt text'}
		aria-label={hasAlt ? `Edit alt text: ${altText}` : 'Add alt text for accessibility'}
	>
		ALT
		{#if hasAlt}
			<span class="alt-dot" aria-hidden="true"></span>
		{/if}
	</button>

	{#if expanded}
		<div class="alt-input-wrap">
			<input
				bind:this={inputEl}
				type="text"
				class="alt-input"
				placeholder="Describe this image..."
				value={altText}
				oninput={(e) => onchange(e.currentTarget.value)}
				onkeydown={handleKeydown}
				onblur={handleBlur}
				maxlength={1000}
				aria-label="Alt text description"
			/>
		</div>
	{/if}
</div>

<style>
	.alt-badge-wrap {
		position: absolute;
		bottom: 6px;
		right: 6px;
		z-index: 2;
	}

	.alt-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.04em;
		padding: 3px 7px;
		border-radius: 4px;
		background: rgba(0, 0, 0, 0.65);
		color: rgba(255, 255, 255, 0.7);
		border: none;
		cursor: pointer;
		backdrop-filter: blur(4px);
		transition: all 0.15s ease;
	}

	.alt-badge:hover {
		background: rgba(0, 0, 0, 0.85);
		color: #fff;
	}

	.alt-badge.has-alt {
		color: #fff;
		background: rgba(0, 0, 0, 0.8);
	}

	.alt-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--color-success, #22c55e);
	}

	.alt-input-wrap {
		position: absolute;
		bottom: calc(100% + 4px);
		right: 0;
		width: 200px;
	}

	.alt-input {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid color-mix(in srgb, var(--color-accent) 40%, transparent);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-sans);
		outline: none;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
	}

	.alt-input::placeholder {
		color: var(--color-text-subtle);
	}

	.alt-input:focus {
		border-color: var(--color-accent);
	}

	@media (pointer: coarse) {
		.alt-badge {
			min-width: 44px;
			min-height: 28px;
			justify-content: center;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.alt-badge {
			transition: none;
		}
	}
</style>
