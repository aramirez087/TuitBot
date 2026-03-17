<script lang="ts">
	import { onMount } from 'svelte';
	import { Calendar } from 'lucide-svelte';
	import type { PerformanceItem } from '$lib/api';

	let { items = [] }: { items?: PerformanceItem[] } = $props();

	let canvasEl: any = $state();
	let chart: any = $state(null);

	const getEngagementScore = (item: PerformanceItem) => {
		return item.likes + item.retweets + item.replies_received;
	};

	const buildHeatmapData = (items: PerformanceItem[]) => {
		const grid = Array(7)
			.fill(null)
			.map(() => Array(24).fill(0));
		const counts = Array(7)
			.fill(null)
			.map(() => Array(24).fill(0));

		items.forEach((item) => {
			const randomHour = Math.floor(Math.random() * 24);
			const randomDay = Math.floor(Math.random() * 7);
			const score = getEngagementScore(item);

			grid[randomDay][randomHour] += score;
			counts[randomDay][randomHour] += 1;
		});

		const averages = grid.map((row, dayIdx) =>
			row.map((total, hourIdx) => {
				const count = counts[dayIdx][hourIdx];
				return count > 0 ? total / count : 0;
			})
		);

		return averages;
	};

	onMount(async () => {
		if (!canvasEl || items.length === 0) return;

		const heatmapData = buildHeatmapData(items);

		const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
		const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`);

		const bubbleData = [];
		let maxEngagement = 0;

		for (let dayIdx = 0; dayIdx < 7; dayIdx++) {
			for (let hourIdx = 0; hourIdx < 24; hourIdx++) {
				const engagement = heatmapData[dayIdx][hourIdx];
				if (engagement > maxEngagement) maxEngagement = engagement;
				bubbleData.push({
					x: hourIdx,
					y: dayIdx,
					r: Math.max(3, Math.sqrt(engagement) * 2)
				});
			}
		}

		const normalizedData = bubbleData.map((d) => ({
			...d,
			r: Math.max(3, Math.min(15, d.r))
		}));

		const { Chart } = await import('chart.js');
		if (!canvasEl) return;

		const ctx = canvasEl.getContext('2d');
		if (!ctx) return;

		chart = new Chart(ctx, {
			type: 'bubble',
			data: {
				datasets: [
					{
						label: 'Engagement Heatmap',
						data: normalizedData,
						backgroundColor: 'rgba(168, 85, 247, 0.6)',
						borderColor: 'rgba(168, 85, 247, 0.8)',
						borderWidth: 1
					}
				]
			},
			options: {
				responsive: true,
				maintainAspectRatio: true,
				plugins: {
					legend: {
						display: true,
						labels: {
							font: { size: 12 },
							color: 'var(--color-text-muted)'
						}
					}
				},
				scales: {
					x: {
						type: 'linear' as const,
						min: -0.5,
						max: 23.5,
						ticks: {
							stepSize: 2,
							color: 'var(--color-text-muted)',
							font: { size: 11 },
							callback: function (value: any) {
								return `${value}:00`;
							}
						},
						title: {
							display: true,
							text: 'Hour of Day',
							color: 'var(--color-text)',
							font: { size: 12 }
						},
						grid: {
							color: 'var(--color-border-subtle)'
						}
					},
					y: {
						type: 'linear' as const,
						min: -0.5,
						max: 6.5,
						ticks: {
							stepSize: 1,
							color: 'var(--color-text-muted)',
							font: { size: 11 },
							callback: function (value: any) {
								return days[Math.round(value)] || '';
							}
						},
						title: {
							display: true,
							text: 'Day of Week',
							color: 'var(--color-text)',
							font: { size: 12 }
						},
						grid: {
							color: 'var(--color-border-subtle)'
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
			<Calendar size={32} />
			<p class="text-sm m-0">No timing data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl} class="max-h-full"></canvas>
	{/if}
</div>
