<script lang="ts">
	import { trackFallbackOpened } from '$lib/analytics/hookMinerFunnel';

	let {
		reason,
		sessionId = 'unknown',
		acceptedCount = 0,
		onusegenerichooks,
		onbacktoneighbors,
	}: {
		reason?: string;
		sessionId?: string;
		acceptedCount?: number;
		onusegenerichooks: () => void;
		onbacktoneighbors: () => void;
	} = $props();

	$effect(() => {
		trackFallbackOpened(reason ?? 'weak_signal', sessionId, acceptedCount);
	});

	const heading = $derived(
		reason === 'timeout'
			? 'Mining took too long. Try again or use generic hooks.'
			: reason === 'parse_error'
				? "Couldn't parse mined angles. Try again or use generic hooks."
				: 'NOT ENOUGH SIGNAL'
	);

	const showBody = $derived(reason !== 'timeout' && reason !== 'parse_error');

	const primaryLabel = $derived(
		reason === 'timeout' || reason === 'parse_error' ? 'Use generic hooks' : 'Use generic hooks'
	);

	const secondaryLabel = $derived(
		reason === 'timeout' || reason === 'parse_error' ? 'Mine again' : '\u2190 Back to related notes'
	);
</script>

<div class="angle-fallback" role="status">
	<p class="angle-fallback-heading">{heading}</p>
	{#if showBody}
		<p class="angle-fallback-body">
			Your selected notes didn't surface enough evidence for mined angles.
			You can include more related notes or use generic hooks instead.
		</p>
	{/if}
	<div class="angle-fallback-actions">
		<button class="angle-fallback-primary" onclick={onusegenerichooks}>
			{primaryLabel}
		</button>
		<button class="angle-fallback-secondary" onclick={onbacktoneighbors}>
			{secondaryLabel}
		</button>
	</div>
</div>

<style>
	.angle-fallback {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 20px 16px;
		text-align: center;
	}

	.angle-fallback-heading {
		margin: 0;
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.angle-fallback-body {
		margin: 0;
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.5;
		max-width: 320px;
	}

	.angle-fallback-actions {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 4px;
	}

	.angle-fallback-primary {
		padding: 5px 14px;
		border: 1px solid var(--color-accent);
		border-radius: 5px;
		background: var(--color-accent);
		color: #fff;
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.angle-fallback-primary:hover {
		background: var(--color-accent-hover);
	}

	.angle-fallback-secondary {
		padding: 5px 14px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.angle-fallback-secondary:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	@media (pointer: coarse) {
		.angle-fallback-primary,
		.angle-fallback-secondary {
			min-height: 44px;
			padding: 10px 14px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.angle-fallback-primary,
		.angle-fallback-secondary {
			transition: none;
		}
	}
</style>
