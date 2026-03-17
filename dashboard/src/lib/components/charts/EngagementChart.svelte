<script lang="ts">
	import { onMount } from 'svelte';
	import { BarChart } from 'lucide-svelte';
	import type { PerformanceItem } from '$lib/api';

	let { items = [] }: { items?: PerformanceItem[] } = $props();

	let canvasEl: any = $state(); HTMLCanvasElement;
	let chart: any = $state(null);

	const aggregateByDate = (items: PerformanceItem[]) => {
		const map = new Map<string, { likes: number; retweets: number; replies: number }>();

		items.forEach((item) => {
			// Extract date from content preview or use a placeholder
			// For now, we'll aggregate all items into one group per item
			// In a real scenario, you'd extract dates from content metadata
			const key = item.content_preview.substring(0, 20); // Use first 20 chars as unique key
			if (!map.has(key)) {
				map.set(key, { likes: 0, retweets: 0, replies: 0 });
			}
			const entry = map.get(key)!;
			entry.likes += item.likes;
			entry.retweets += item.retweets;
			entry.replies += item.replies_received;
		});

		return Array.from(map.entries()).map(([key, data]) => ({
			label: key,
			...data
		}));
	};

	onMount(async () => {
		if (!canvasEl || items.length === 0) return;

		const aggregated = aggregateByDate(items);
		const labels = aggregated.map((d) => d.label);
		const likes = aggregated.map((d) => d.likes);
		const retweets = aggregated.map((d) => d.retweets);
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
						label: 'Likes',
						data: likes,
						backgroundColor: 'rgb(239, 68, 68)', // red-500
						borderColor: 'rgb(239, 68, 68)',
						borderWidth: 0
					},
					{
						label: 'Retweets',
						data: retweets,
						backgroundColor: 'rgb(34, 197, 94)', // green-500
						borderColor: 'rgb(34, 197, 94)',
						borderWidth: 0
					},
					{
						label: 'Replies',
						data: replies,
						backgroundColor: 'rgb(59, 130, 246)', // blue-500
						borderColor: 'rgb(59, 130, 246)',
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
							font: { size: 11 }
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

<div class="engagement-chart">
	{#if items.length === 0}
		<div class="empty-state">
			<BarChart size={32} class="text-muted" />
			<p>No engagement data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl}></canvas>
	{/if}
</div>

<style>
	.engagement-chart {
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
