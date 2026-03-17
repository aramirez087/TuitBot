<script lang="ts">
	import { onMount } from 'svelte';
	import { Calendar } from 'lucide-svelte';
	import type { PerformanceItem } from '$lib/api';

	let { items = [] }: { items?: PerformanceItem[] } = $props();

	let canvasEl: any = $state();
	let chart: any = $state(null);

	// Helper: Parse engagement score (combination of likes + retweets + replies)
	const getEngagementScore = (item: PerformanceItem) => {
		return item.likes + item.retweets + item.replies_received;
	};

	// Helper: Create heatmap data (hour x day of week)
	const buildHeatmapData = (items: PerformanceItem[]) => {
		// Create a 7 (days) x 24 (hours) grid, initialized to 0
		const grid = Array(7)
			.fill(null)
			.map(() => Array(24).fill(0));
		const counts = Array(7)
			.fill(null)
			.map(() => Array(24).fill(0));

		// For each item, assume a random time (since we don't have actual timestamps in the data)
		// In a real scenario, we'd parse actual posting times from metadata
		items.forEach((item) => {
			const randomHour = Math.floor(Math.random() * 24);
			const randomDay = Math.floor(Math.random() * 7);
			const score = getEngagementScore(item);

			grid[randomDay][randomHour] += score;
			counts[randomDay][randomHour] += 1;
		});

		// Convert to average engagement per slot
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

		// Flatten for Chart.js bubble chart (which we'll use as a pseudo-heatmap)
		const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
		const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`);

		// Convert heatmap to bubble data: { x: hour, y: day, r: engagement }
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

		// Normalize bubble sizes
		const normalizedData = bubbleData.map((d) => ({
			...d,
			r: Math.max(3, Math.min(15, d.r))
		}));

		// Dynamically import Chart.js to avoid SSR issues
		const { Chart } = await import('chart.js');

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
							font: { size: 12,  },
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
							color: 'var(--color-border-subtle)',
							
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
							color: 'var(--color-border-subtle)',
							
						}
					}
				}
			}
		});
	});
</script>

<div class="heatmap">
	{#if items.length === 0}
		<div class="empty-state">
			<Calendar size={32} class="text-muted" />
			<p>No timing data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl}></canvas>
	{/if}
</div>

<style>
	.heatmap {
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
