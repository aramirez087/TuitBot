<script lang="ts">
	import { onMount } from 'svelte';
	import { Eye } from 'lucide-svelte';
	import type { PerformanceItem } from '$lib/api';

	let { items = [] }: { items?: PerformanceItem[] } = $props();

	let canvasEl: any = $state(); HTMLCanvasElement;
	let chart: any = $state(null);

	const aggregateReach = (items: PerformanceItem[]) => {
		const map = new Map<string, { impressions: number; replies: number }>();

		items.forEach((item) => {
			// Group by content type (tweets, threads, replies)
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

		// Dynamically import Chart.js to avoid SSR issues
		const { Chart } = await import('chart.js');

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
						backgroundColor: 'rgb(129, 140, 248)', // indigo-400
						borderColor: 'rgb(129, 140, 248)',
						borderWidth: 0
					},
					{
						label: 'Replies',
						data: replies,
						backgroundColor: 'rgb(249, 115, 22)', // amber-500
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
							font: { size: 12,  },
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
							color: 'var(--color-border-subtle)',
							
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

<div class="reach-chart">
	{#if items.length === 0}
		<div class="empty-state">
			<Eye size={32} class="text-muted" />
			<p>No reach data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl}></canvas>
	{/if}
</div>

<style>
	.reach-chart {
		width: 100%;
		height: 300px;
		padding: 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background-color: var(--color-surface);
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
		color: var(--color-text-muted);
	}

	.empty-state p {
		font-size: 14px;
		margin: 0;
	}

	canvas {
		max-height: 100%;
	}

	:global(.text-muted) {
		color: var(--color-text-muted);
	}
</style>
