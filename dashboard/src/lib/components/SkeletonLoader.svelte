<script lang="ts">
	interface Props {
		lines?: number;
		blocks?: number;
	}

	let { lines = 3, blocks = 0 }: Props = $props();
</script>

<div class="skeleton-container">
	{#if blocks > 0}
		<div class="skeleton-blocks">
			{#each Array(blocks) as _}
				<div class="skeleton-block"></div>
			{/each}
		</div>
	{/if}
	{#each Array(lines) as _, i}
		<div
			class="skeleton-line"
			style:width={i === lines - 1 ? '60%' : '100%'}
		></div>
	{/each}
</div>

<style>
	.skeleton-container {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px 0;
	}

	.skeleton-blocks {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 16px;
		margin-bottom: 8px;
	}

	.skeleton-block {
		height: 100px;
		background: var(--color-surface);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.skeleton-line {
		height: 14px;
		background: var(--color-surface);
		border-radius: 4px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% {
			opacity: 0.4;
		}
		50% {
			opacity: 0.8;
		}
	}
</style>
