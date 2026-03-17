<script lang="ts">
	import { onMount } from 'svelte';
	import { BarChart } from 'lucide-svelte';
	import type { PerformanceItem } from '$lib/api';

	let { items = [] }: { items?: PerformanceItem[] } = $props();

	let canvasEl: any = $state();
	let chart: any = $state(null);

	const aggregateByDate = (items: PerformanceItem[]) => {
		const map = new Map<string, { likes: number; retweets: number; replies: number }>();

		items.forEach((item) => {
			const key = item.content_preview.substring(0, 20);
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

		const { Chart } = await import('chart.js');
		if (!canvasEl) return;

		if (!canvasEl) return;
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
						backgroundColor: 'rgb(239, 68, 68)',
						borderColor: 'rgb(239, 68, 68)',
						borderWidth: 0
					},
					{
						label: 'Retweets',
						data: retweets,
						backgroundColor: 'rgb(34, 197, 94)',
						borderColor: 'rgb(34, 197, 94)',
						borderWidth: 0
					},
					{
						label: 'Replies',
						data: replies,
						backgroundColor: 'rgb(59, 130, 246)',
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
							font: { size: 11 }
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

<div class="w-full h-80 p-4 border border-slate-200 rounded-lg bg-slate-50">
	{#if items.length === 0}
		<div class="h-full flex flex-col items-center justify-center gap-3 text-slate-500">
			<BarChart size={32} />
			<p class="text-sm m-0">No engagement data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl} class="max-h-full"></canvas>
	{/if}
</div>
