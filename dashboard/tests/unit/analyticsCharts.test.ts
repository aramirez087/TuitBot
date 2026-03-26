/**
 * analyticsCharts.test.ts
 *
 * Smoke tests for the analytics chart components added in PR #211.
 * Covers: mount without errors, empty-state rendering, prop passing.
 * Chart.js canvas rendering is mocked (no actual canvas in jsdom).
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

// Mock Chart.js — jsdom has no canvas.
// Use a real class so `new Chart(...)` works as a constructor.
vi.mock('chart.js', () => {
	class MockChart {
		destroy = vi.fn();
		update = vi.fn();
	}
	return { Chart: MockChart, registerables: [] };
});

// Stub canvas getContext so chart onMount handlers don't throw in jsdom
if (typeof HTMLCanvasElement !== 'undefined') {
	HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue({
		clearRect: vi.fn(),
		fillRect: vi.fn(),
		beginPath: vi.fn(),
		arc: vi.fn(),
		fill: vi.fn(),
		stroke: vi.fn(),
		measureText: vi.fn().mockReturnValue({ width: 0 }),
		fillText: vi.fn()
	}) as unknown as typeof HTMLCanvasElement.prototype.getContext;
}

// Mock lucide-svelte icons used by components
vi.mock('lucide-svelte', () => ({
	BarChart: vi.fn().mockReturnValue(null),
	BarChart2: vi.fn().mockReturnValue(null),
	TrendingUp: vi.fn().mockReturnValue(null),
	TrendingDown: vi.fn().mockReturnValue(null),
	Eye: vi.fn().mockReturnValue(null),
	Calendar: vi.fn().mockReturnValue(null),
	Loader2: vi.fn().mockReturnValue(null),
	Users: vi.fn().mockReturnValue(null),
	Clock: vi.fn().mockReturnValue(null),
	LineChart: vi.fn().mockReturnValue(null),
	MessageSquare: vi.fn().mockReturnValue(null),
	Repeat2: vi.fn().mockReturnValue(null),
	RefreshCw: vi.fn().mockReturnValue(null)
}));

import EngagementChart from '../../src/lib/components/charts/EngagementChart.svelte';
import ReachChart from '../../src/lib/components/charts/ReachChart.svelte';
import FollowerGrowthChart from '../../src/lib/components/charts/FollowerGrowthChart.svelte';
import BestTimeHeatmap from '../../src/lib/components/charts/BestTimeHeatmap.svelte';
import AnalyticsDashboard from '../../src/lib/components/AnalyticsDashboard.svelte';
import { recentPerformance, followerSnapshots, loading, error } from '../../src/lib/stores/analytics';

const mockPerformanceItem = () => ({
	content_type: 'tweet',
	content_preview: 'Test tweet content here',
	posted_at: '2026-03-17T10:00:00Z',
	likes: 10,
	retweets: 5,
	replies_received: 2,
	impressions: 200,
	performance_score: 0.85
});

const mockFollowerSnapshot = () => ({
	snapshot_date: '2026-03-17',
	follower_count: 1500,
	following_count: 200,
	tweet_count: 42
});

describe('EngagementChart', () => {
	it('renders without throwing with empty items', () => {
		expect(() => render(EngagementChart, { props: { items: [] } })).not.toThrow();
	});

	it('renders without throwing with data', () => {
		expect(() =>
			render(EngagementChart, { props: { items: [mockPerformanceItem()] } })
		).not.toThrow();
	});
});

describe('ReachChart', () => {
	it('renders without throwing with empty items', () => {
		expect(() => render(ReachChart, { props: { items: [] } })).not.toThrow();
	});

	it('renders without throwing with data', () => {
		expect(() =>
			render(ReachChart, { props: { items: [mockPerformanceItem()] } })
		).not.toThrow();
	});
});

describe('FollowerGrowthChart', () => {
	it('renders without throwing with empty snapshots', () => {
		expect(() => render(FollowerGrowthChart, { props: { snapshots: [] } })).not.toThrow();
	});

	it('renders without throwing with data', () => {
		expect(() =>
			render(FollowerGrowthChart, { props: { snapshots: [mockFollowerSnapshot()] } })
		).not.toThrow();
	});
});

describe('BestTimeHeatmap', () => {
	it('renders without throwing with empty items', () => {
		expect(() => render(BestTimeHeatmap, { props: { items: [] } })).not.toThrow();
	});

	it('renders without throwing with data', () => {
		expect(() =>
			render(BestTimeHeatmap, { props: { items: [mockPerformanceItem()] } })
		).not.toThrow();
	});
});

describe('AnalyticsDashboard', () => {
	it('renders without throwing with no props', () => {
		expect(() => render(AnalyticsDashboard)).not.toThrow();
	});

	it('renders without throwing when stores have data', () => {
		// AnalyticsDashboard reads from stores, not props — populate stores before render
		recentPerformance.set([mockPerformanceItem()]);
		followerSnapshots.set([mockFollowerSnapshot()]);
		loading.set(false);
		error.set(null);
		expect(() => render(AnalyticsDashboard)).not.toThrow();
	});
});
