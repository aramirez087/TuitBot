/**
 * McpToolsSection.test.ts — Smoke tests for ToolsSection.svelte (MCP tools panel)
 *
 * Written by Sentinel (QA Lead) — 2026-03-19.
 * WHY: Q3 acceptance criteria — verify the MCP tools panel renders from a
 * mocked endpoint response without crashing and without console errors.
 * No real server required; store is seeded directly.
 *
 * U3 correlation: ToolsSection.svelte lives at
 * src/routes/(app)/mcp/ToolsSection.svelte and consumes the `metrics` store
 * from $lib/stores/mcp. We set the store directly to avoid any network calls.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import { metrics } from '$lib/stores/mcp';
import ToolsSection from '../../src/routes/(app)/mcp/ToolsSection.svelte';
import type { McpToolMetrics } from '$lib/api/types';

// ─── Console error spy — AC2: no console errors on render ────────────────────
// Capture and fail if any console.error fires during tests.
let consoleErrors: string[] = [];
beforeEach(() => {
	consoleErrors = [];
	vi.spyOn(console, 'error').mockImplementation((...args) => {
		consoleErrors.push(args.join(' '));
	});
});

// ─── Shared helpers ───────────────────────────────────────────────────────────

function makeTool(overrides: Partial<McpToolMetrics> = {}): McpToolMetrics {
	return {
		tool_name: 'compose_tweet',
		category: 'mutation',
		total_calls: 42,
		success_count: 40,
		failure_count: 2,
		success_rate: 0.952,
		avg_latency_ms: 180,
		p50_latency_ms: 150,
		p95_latency_ms: 420,
		min_latency_ms: 50,
		max_latency_ms: 890,
		...overrides,
	};
}

const formatRate = (n: number) => (n * 100).toFixed(1) + '%';
const formatLatency = (ms: number) => {
	if (ms < 1) return '<1ms';
	if (ms >= 1000) return (ms / 1000).toFixed(1) + 's';
	return Math.round(ms) + 'ms';
};

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('ToolsSection (MCP tools panel smoke tests)', () => {
	it('renders empty state when metrics store is empty — no crash, no console errors', () => {
		metrics.set([]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		expect(container).toBeTruthy();
		expect(consoleErrors).toHaveLength(0);
		// Empty state message must be visible
		expect(container.textContent).toContain('No tool executions recorded');
	});

	it('renders tools list from mocked store data — no crash, no console errors', () => {
		metrics.set([makeTool()]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		expect(container).toBeTruthy();
		expect(consoleErrors).toHaveLength(0);
		// Tool name appears in the rendered table
		expect(container.textContent).toContain('compose_tweet');
	});

	it('renders tool name, category, and call counts from mocked data', () => {
		metrics.set([
			makeTool({ tool_name: 'publish_thread', category: 'mutation', total_calls: 10, success_count: 9, failure_count: 1 }),
		]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		expect(container.textContent).toContain('publish_thread');
		expect(container.textContent).toContain('mutation');
		expect(container.textContent).toContain('10'); // total_calls
	});

	it('renders multiple tools without crash', () => {
		metrics.set([
			makeTool({ tool_name: 'compose_tweet', category: 'mutation' }),
			makeTool({ tool_name: 'list_drafts', category: 'query', success_rate: 1.0, failure_count: 0 }),
			makeTool({ tool_name: 'schedule_post', category: 'mutation', success_rate: 0.75, failure_count: 5 }),
		]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		expect(container).toBeTruthy();
		expect(consoleErrors).toHaveLength(0);
		expect(container.textContent).toContain('compose_tweet');
		expect(container.textContent).toContain('list_drafts');
		expect(container.textContent).toContain('schedule_post');
	});

	it('formats success rate correctly in the rendered output', () => {
		metrics.set([makeTool({ success_rate: 0.952 })]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		// 0.952 → "95.2%"
		expect(container.textContent).toContain('95.2%');
	});

	it('formats latency correctly in the rendered output', () => {
		metrics.set([makeTool({ avg_latency_ms: 180, p50_latency_ms: 150, p95_latency_ms: 420 })]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		expect(container.textContent).toContain('180ms');
		expect(container.textContent).toContain('150ms');
		expect(container.textContent).toContain('420ms');
	});

	it('shows failure count highlighted for tools with failures', () => {
		metrics.set([makeTool({ failure_count: 3 })]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		// Failure count must appear in text
		expect(container.textContent).toContain('3');
	});

	it('applies mutation badge class for mutation category tools', () => {
		metrics.set([makeTool({ category: 'mutation' })]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		const badge = container.querySelector('.category-badge.mutation');
		expect(badge).not.toBeNull();
	});

	it('does not apply mutation badge class for query category tools', () => {
		metrics.set([makeTool({ category: 'query' })]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		const mutationBadge = container.querySelector('.category-badge.mutation');
		expect(mutationBadge).toBeNull();
		// But a non-mutation badge must exist
		const badge = container.querySelector('.category-badge');
		expect(badge).not.toBeNull();
	});

	it('renders table headers when tools are present', () => {
		metrics.set([makeTool()]);
		const { container } = render(ToolsSection, { props: { formatRate, formatLatency } });
		const headers = container.querySelectorAll('th');
		expect(headers.length).toBeGreaterThan(0);
		// Spot-check key columns
		const headerText = Array.from(headers).map((h) => h.textContent?.trim() ?? '');
		expect(headerText.some((h) => h.toLowerCase().includes('tool'))).toBe(true);
		expect(headerText.some((h) => h.toLowerCase().includes('rate'))).toBe(true);
	});
});
