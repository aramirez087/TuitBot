<script lang="ts">
	let {
		index,
		total,
		focused = false,
		overLimit = false,
	}: {
		index: number;
		total: number;
		focused?: boolean;
		overLimit?: boolean;
	} = $props();
</script>

<div class="card-identity" class:focused class:over-limit={overLimit}>
	<span class="identity-index"
		>#{index + 1}<span class="identity-total">/{total}</span></span
	>
</div>

<style>
	.card-identity {
		display: flex;
		align-items: center;
		gap: 8px;
		padding-bottom: 4px;
		margin-bottom: 2px;
		min-height: 20px;
	}

	.identity-index {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		flex-shrink: 0;
		letter-spacing: -0.02em;
		opacity: 0;
		transition: opacity 0.15s ease;
	}

	.card-identity.focused .identity-index,
	.card-identity:hover .identity-index {
		opacity: 1;
	}

	.card-identity.over-limit .identity-index {
		opacity: 1;
		color: var(--color-danger);
	}

	.identity-total {
		opacity: 0.5;
	}

	@media (prefers-reduced-motion: reduce) {
		.identity-index {
			transition: none;
		}
	}
</style>
