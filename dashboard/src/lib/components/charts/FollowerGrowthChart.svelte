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

<div class="w-full h-80 p-4 border border-slate-200 rounded-lg bg-slate-50">
	{#if snapshots.length === 0}
		<div class="h-full flex flex-col items-center justify-center gap-3 text-slate-500">
			<TrendingUp size={32} />
			<p class="text-sm m-0">No follower data available</p>
		</div>
	{:else}
		<canvas bind:this={canvasEl} class="max-h-full"></canvas>
	{/if}
</div>
