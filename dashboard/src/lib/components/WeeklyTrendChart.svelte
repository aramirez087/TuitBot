<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Chart,
		LineController,
		LineElement,
		PointElement,
		LinearScale,
		CategoryScale,
		Tooltip,
		Legend
	} from 'chart.js';
	import type { StrategyReport } from '$lib/api';

	Chart.register(LineController, LineElement, PointElement, LinearScale, CategoryScale, Tooltip, Legend);

	interface Props {
		reports: StrategyReport[];
	}

	let { reports }: Props = $props();

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;

	function buildChart(data: StrategyReport[]) {
		const reversed = [...data].reverse();
		const labels = reversed.map((r) => {
			const d = new Date(r.week_start);
			return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
		});

		const accentColor =
			getComputedStyle(document.documentElement)
				.getPropertyValue('--color-accent')
				.trim() || '#58a6ff';
		const successColor =
			getComputedStyle(document.documentElement)
				.getPropertyValue('--color-success')
				.trim() || '#3fb950';
		const warnColor = '#d29922';

		const datasets = [
			{
				label: 'Follower Delta',
				data: reversed.map((r) => r.follower_delta),
				borderColor: successColor,
				backgroundColor: successColor + '20',
				borderWidth: 2,
				pointRadius: 3,
				tension: 0.3
			},
			{
				label: 'Avg Engagement',
				data: reversed.map((r) => {
					const scores = [r.avg_reply_score, r.avg_tweet_score].filter((s) => s > 0);
					return scores.length > 0 ? scores.reduce((a, b) => a + b, 0) / scores.length : 0;
				}),
				borderColor: accentColor,
				backgroundColor: accentColor + '20',
				borderWidth: 2,
				pointRadius: 3,
				tension: 0.3
			},
			{
				label: 'Acceptance Rate (%)',
				data: reversed.map((r) => r.reply_acceptance_rate * 100),
				borderColor: warnColor,
				backgroundColor: warnColor + '20',
				borderWidth: 2,
				pointRadius: 3,
				tension: 0.3
			}
		];

		if (chart) {
			chart.data.labels = labels;
			chart.data.datasets = datasets;
			chart.update();
			return;
		}

		chart = new Chart(canvas, {
			type: 'line',
			data: { labels, datasets },
			options: {
				responsive: true,
				maintainAspectRatio: false,
				interaction: { mode: 'index', intersect: false },
				plugins: {
					tooltip: {
						backgroundColor: '#161b22',
						titleColor: '#e6edf3',
						bodyColor: '#8b949e',
						borderColor: '#30363d',
						borderWidth: 1,
						padding: 10
					},
					legend: {
						labels: {
							color: '#8b949e',
							boxWidth: 12,
							padding: 16,
							font: { size: 11 }
						}
					}
				},
				scales: {
					x: {
						grid: { color: '#21262d' },
						ticks: { color: '#6e7681', maxTicksLimit: 8 }
					},
					y: {
						grid: { color: '#21262d' },
						ticks: { color: '#6e7681' }
					}
				}
			}
		});
	}

	$effect(() => {
		if (canvas && reports.length > 0) {
			buildChart(reports);
		}
	});

	onDestroy(() => {
		chart?.destroy();
		chart = null;
	});
</script>

<div class="chart-container">
	<div class="chart-header">
		<h3>Weekly Trends</h3>
	</div>
	{#if reports.length === 0}
		<div class="empty-chart">
			<span>No historical data yet. Reports accumulate weekly.</span>
		</div>
	{:else}
		<div class="chart-body">
			<canvas bind:this={canvas}></canvas>
		</div>

		<div class="history-table">
			<table>
				<thead>
					<tr>
						<th>Week</th>
						<th>Followers</th>
						<th>Replies</th>
						<th>Tweets</th>
						<th>Avg Score</th>
						<th>Acceptance</th>
					</tr>
				</thead>
				<tbody>
					{#each reports as report}
						<tr>
							<td>{report.week_start}</td>
							<td class:positive={report.follower_delta > 0} class:negative={report.follower_delta < 0}>
								{report.follower_delta > 0 ? '+' : ''}{report.follower_delta}
							</td>
							<td>{report.replies_sent}</td>
							<td>{report.tweets_posted}</td>
							<td>
								{(() => {
									const scores = [report.avg_reply_score, report.avg_tweet_score].filter((s) => s > 0);
									return scores.length > 0
										? (scores.reduce((a, b) => a + b, 0) / scores.length).toFixed(1)
										: '0.0';
								})()}
							</td>
							<td>{(report.reply_acceptance_rate * 100).toFixed(0)}%</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.chart-container {
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		padding: 20px;
	}

	.chart-header {
		margin-bottom: 16px;
	}

	h3 {
		margin: 0;
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
	}

	.chart-body {
		position: relative;
		height: 240px;
		margin-bottom: 20px;
	}

	.empty-chart {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 120px;
		color: var(--color-text-subtle);
		font-size: 13px;
	}

	.history-table {
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12px;
	}

	th {
		padding: 6px 10px;
		text-align: left;
		font-weight: 600;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		border-bottom: 1px solid var(--color-border-subtle);
	}

	td {
		padding: 6px 10px;
		color: var(--color-text-muted);
		border-bottom: 1px solid var(--color-border-subtle);
	}

	tr:last-child td {
		border-bottom: none;
	}

	.positive {
		color: var(--color-success);
	}

	.negative {
		color: var(--color-danger);
	}
</style>
