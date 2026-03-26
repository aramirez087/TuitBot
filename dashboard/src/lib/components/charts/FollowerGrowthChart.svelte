<script lang="ts">
	import { onMount } from 'svelte';
	import { TrendingUp } from 'lucide-svelte';
	import type { FollowerSnapshot } from '$lib/api';

	let { snapshots = [] }: { snapshots?: FollowerSnapshot[] } = $props();

	let canvasEl: any = $state();
	let chart: any = $state(null);

	onMount(async () => {
		if (!canvasEl || snapshots.length === 0) return;

		const labels = snapshots.map((s) => {
			const date = new Date(s.snapshot_date);
			return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
		});

		const followerCounts = snapshots.map((s) => s.follower_count);

		const { Chart } = await import('chart.js');
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		chart = new Chart(ctx, {
			type: 'line',
			data: {
				labels,
				datasets: [
					{
						label: 'Followers',
						data: followerCounts,
						borderColor: 'rgb(168, 85, 247)',
						backgroundColor: 'rgba(168, 85, 247, 0.1)',
						borderWidth: 2,
						fill: true,
						tension: 0.3,
						pointBackgroundColor: 'rgb(168, 85, 247)',
						pointBorderColor: 'white',
						pointBorderWidth: 2,
						pointRadius: 4,
						pointHoverRadius: 6
					}
				]
			},
			options: {
				responsive: true,
				maintainAspectRatio: true,
				interaction: {
					mode: 'index' as const,
					intersect: false
				},
				plugins: {
					legend: {
						display: true,
						position: 'top' as const,
						labels: {
							padding: 12,
							font: { size: 12 },
							color: 'var(--color-text-muted)'
						}
					}
				},
				scales: {
					y: {
						beginAtZero: false,
						ticks: {
							color: 'var(--color-text-muted)',
							font: { size: 11 },
							callback: function (value: any) {
								return value.toLocaleString();
							}
						},
						grid: {
							color: 'var(--color-border-subtle)'
						}
					},
					x: {
						ticks: {
							color: 'var(--color-text-muted)',
							font: { size: 11 }
						},
						grid: {
							display: false
						}
					}
				}
			}
		});
	});
</script>

<div class="chart-container">
	{#if snapshots.length === 0}
		<div class="chart-empty">
			<TrendingUp size={32} />
			<p class="chart-empty-text">No follower data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl} class="chart-canvas"></canvas>
	{/if}
</div>

<style>
	.chart-container {
		width: 100%;
		height: 320px;
		padding: 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: var(--color-surface);
	}

	.chart-empty {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		color: var(--color-text-subtle);
	}

	.chart-empty-text {
		font-size: 13px;
		margin: 0;
		color: var(--color-text-muted);
	}

	.chart-canvas {
		max-height: 100%;
	}
</style>
