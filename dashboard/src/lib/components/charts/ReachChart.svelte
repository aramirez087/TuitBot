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

<div class="w-full h-80 p-4 border border-slate-200 rounded-lg bg-slate-50">
	{#if items.length === 0}
		<div class="h-full flex flex-col items-center justify-center gap-3 text-slate-500">
			<Eye size={32} />
			<p class="text-sm m-0">No reach data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl} class="max-h-full"></canvas>
	{/if}
</div>
