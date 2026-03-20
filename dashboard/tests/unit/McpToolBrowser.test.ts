/**
 * McpToolBrowser.test.ts — Unit tests for McpToolBrowser.svelte
 *
 * Covers: render states (loading/error/empty/populated), search filtering,
 * expand/collapse, param table rendering, accessibility attributes.
 *
 * WHY: Discovery panel is new with no prior tests. Mocking api.mcp.tools()
 * to avoid real network calls.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('$lib/api', () => ({
	api: {
		mcp: {
			tools: vi.fn()
		}
	}
}));

import { api } from '$lib/api';
import McpToolBrowser from '../../src/routes/(app)/mcp/McpToolBrowser.svelte';
import type { McpAvailableTool } from '$lib/api/types';

// ─── Fixtures ────────────────────────────────────────────────────────────────

function makeTool(overrides: Partial<McpAvailableTool> = {}): McpAvailableTool {
	return {
		name: 'compose_tweet',
		description: 'Compose and queue a tweet for review',
		category: 'mutation',
		params: [
			{ name: 'content', type: 'string', description: 'Tweet text', required: true },
			{ name: 'schedule_at', type: 'string', description: 'ISO8601 datetime', required: false }
		],
		...overrides
	};
}

const TOOLS: McpAvailableTool[] = [
	makeTool({ name: 'compose_tweet', category: 'mutation' }),
	makeTool({ name: 'list_drafts', category: 'query', description: 'List draft posts', params: [] }),
	makeTool({ name: 'schedule_post', category: 'mutation', description: 'Schedule a post' })
];

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('McpToolBrowser', () => {
	beforeEach(() => {
		vi.resetAllMocks();
	});

	it('renders loading state initially', () => {
		vi.mocked(api.mcp.tools).mockReturnValue(new Promise(() => {})); // never resolves
		const { container } = render(McpToolBrowser);
		expect(container.textContent).toContain('Loading tools');
	});

	it('renders tool list after load', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue(TOOLS);
		const { container } = render(McpToolBrowser);

		await waitFor(() => {
			expect(container.textContent).toContain('compose_tweet');
		});
		expect(container.textContent).toContain('list_drafts');
		expect(container.textContent).toContain('schedule_post');
	});

	it('renders empty state when no tools', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => {
			expect(container.textContent).toContain('No MCP tools available');
		});
	});

	it('renders error state on API failure', async () => {
		vi.mocked(api.mcp.tools).mockRejectedValue(new Error('Network error'));
		const { container } = render(McpToolBrowser);

		await waitFor(() => {
			expect(container.textContent).toContain('Network error');
		});
	});

	it('expands tool details on click', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([makeTool()]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const btn = container.querySelector('.tool-header') as HTMLButtonElement;
		expect(btn).toBeTruthy();

		// Initially collapsed — params table hidden
		expect(container.querySelector('.params-table')).toBeFalsy();

		await fireEvent.click(btn);

		// After click — tool description and params visible
		expect(container.textContent).toContain('Compose and queue a tweet');
		expect(container.querySelector('.params-table')).toBeTruthy();
		expect(btn.getAttribute('aria-expanded')).toBe('true');
	});

	it('collapses tool details on second click', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([makeTool()]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const btn = container.querySelector('.tool-header') as HTMLButtonElement;
		await fireEvent.click(btn); // expand
		await fireEvent.click(btn); // collapse

		expect(container.querySelector('.params-table')).toBeFalsy();
		expect(btn.getAttribute('aria-expanded')).toBe('false');
	});

	it('renders required and optional param badges', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([makeTool()]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const btn = container.querySelector('.tool-header') as HTMLButtonElement;
		await fireEvent.click(btn);

		expect(container.querySelector('.req-badge')).toBeTruthy();
		expect(container.querySelector('.opt-label')).toBeTruthy();
	});

	it('shows mutation badge for mutation tools', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([makeTool({ category: 'mutation' })]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const badge = container.querySelector('.tool-category.mutation');
		expect(badge).toBeTruthy();
	});

	it('does not show mutation badge for query tools', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([makeTool({ category: 'query' })]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		expect(container.querySelector('.tool-category.mutation')).toBeFalsy();
		expect(container.querySelector('.tool-category')).toBeTruthy();
	});

	it('filters tools by search query', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue(TOOLS);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const searchInput = container.querySelector('.search') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'list_drafts' } });

		// Only matching tool visible
		expect(container.textContent).toContain('list_drafts');
		expect(container.textContent).not.toContain('compose_tweet');
	});

	it('shows no-match message when search finds nothing', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue(TOOLS);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const searchInput = container.querySelector('.search') as HTMLInputElement;
		await fireEvent.input(searchInput, { target: { value: 'zzznomatch' } });

		expect(container.textContent).toContain('No tools match your search');
	});

	it('renders param names and types in expanded view', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([makeTool()]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const btn = container.querySelector('.tool-header') as HTMLButtonElement;
		await fireEvent.click(btn);

		expect(container.textContent).toContain('content');
		expect(container.textContent).toContain('schedule_at');
		expect(container.textContent).toContain('string');
	});

	it('shows no-params message for tools with no parameters', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([
			makeTool({ name: 'list_drafts', params: [] })
		]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('list_drafts'));

		const btn = container.querySelector('.tool-header') as HTMLButtonElement;
		await fireEvent.click(btn);

		expect(container.textContent).toContain('No parameters');
		expect(container.querySelector('.params-table')).toBeFalsy();
	});

	it('renders search input with aria-label', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue([]);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('No MCP tools'));

		const input = container.querySelector('input[aria-label]');
		expect(input).toBeTruthy();
	});

	it('renders tool list with role=list for accessibility', async () => {
		vi.mocked(api.mcp.tools).mockResolvedValue(TOOLS);
		const { container } = render(McpToolBrowser);

		await waitFor(() => expect(container.textContent).toContain('compose_tweet'));

		const list = container.querySelector('[role="list"]');
		expect(list).toBeTruthy();
	});
});
