<script lang="ts">
	import { onMount } from 'svelte';
	import { Eye } from 'lucide-svelte';
	import type { PerformanceItem } from '$lib/api';

	let { items = [] }: { items?: PerformanceItem[] } = $props();

	let canvasEl: any = $state();
	let chart: any = $state(null);

	const aggregateReach = (items: PerformanceItem[]) => {
		const map = new Map<string, { impressions: number; replies: number }>();

		items.forEach((item) => {
			const key = item.content_type || 'unknown';
			if (!map.has(key)) {
				map.set(key, { impressions: 0, replies: 0 });
			}
			const entry = map.get(key)!;
			entry.impressions += item.impressions;
			entry.replies += item.replies_received;
		});

		return Array.from(map.entries()).map(([key, data]) => ({
			label: key.charAt(0).toUpperCase() + key.slice(1),
			...data
		}));
	};

	onMount(async () => {
		if (!canvasEl || items.length === 0) return;

		const aggregated = aggregateReach(items);
		const labels = aggregated.map((d) => d.label);
		const impressions = aggregated.map((d) => d.impressions);
		const replies = aggregated.map((d) => d.replies);

		const { Chart } = await import('chart.js');
		if (!canvasEl) return;
		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		chart = new Chart(ctx, {
			type: 'bar',
			data: {
				labels,
				datasets: [
					{
						label: 'Impressions',
						data: impressions,
						backgroundColor: 'rgb(129, 140, 248)',
						borderColor: 'rgb(129, 140, 248)',
						borderWidth: 0
					},
					{
						label: 'Replies',
						data: replies,
						backgroundColor: 'rgb(249, 115, 22)',
						borderColor: 'rgb(249, 115, 22)',
						borderWidth: 0
					}
				]
			},
			options: {
				responsive: true,
				maintainAspectRatio: true,
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
						beginAtZero: true,
						ticks: {
							color: 'var(--color-text-muted)',
							font: { size: 11 },
							callback: function (value: any) {
								if (value >= 1000) {
									return (value / 1000).toFixed(1) + 'k';
								}
								return value.toString();
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
	{#if items.length === 0}
		<div class="chart-empty">
			<Eye size={32} />
			<p class="chart-empty-text">No reach data available</p>
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
