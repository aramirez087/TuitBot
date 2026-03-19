<script lang="ts">
	/**
	 * McpToolBrowser.svelte — Read-only MCP tool discovery panel.
	 *
	 * Fetches available tools from GET /mcp/tools and renders them as an
	 * expandable list with tool name, description, and parameter hints.
	 * Read-only: no invocation UI. Discovery only.
	 *
	 * WHY this is a separate component from ToolsSection.svelte:
	 * ToolsSection shows telemetry metrics for past executions.
	 * This component shows what tools are available (schema/discovery).
	 * These are different data sources and different user needs.
	 */

	import { onMount } from 'svelte';
	import { Search, ChevronDown, ChevronRight, Box, AlertCircle } from 'lucide-svelte';
	import { api } from '$lib/api';
	import type { McpAvailableTool } from '$lib/api/types';

	let tools = $state<McpAvailableTool[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	// Set of expanded tool names
	let expanded = $state<Set<string>>(new Set());

	const filteredTools = $derived(() => {
		const q = searchQuery.trim().toLowerCase();
		if (!q) return tools;
		return tools.filter(
			(t) =>
				t.name.toLowerCase().includes(q) ||
				t.description.toLowerCase().includes(q) ||
				t.category.toLowerCase().includes(q)
		);
	});

	function toggleExpanded(name: string) {
		const next = new Set(expanded);
		if (next.has(name)) {
			next.delete(name);
		} else {
			next.add(name);
		}
		expanded = next;
	}

	onMount(async () => {
		try {
			tools = await api.mcp.tools();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load MCP tools';
		} finally {
			loading = false;
		}
	});
</script>

<div class="browser">
	<div class="browser-header">
		<h2>Available Tools</h2>
		<div class="search-wrap">
			<Search size={14} class="search-icon" />
			<input
				class="search"
				type="text"
				placeholder="Search tools..."
				bind:value={searchQuery}
				aria-label="Search MCP tools"
			/>
		</div>
	</div>

	{#if loading}
		<div class="state-msg">Loading tools...</div>
	{:else if error}
		<div class="state-error" role="alert">
			<AlertCircle size={16} />
			<span>{error}</span>
		</div>
	{:else if filteredTools().length === 0}
		<div class="state-msg">
			{searchQuery ? 'No tools match your search.' : 'No MCP tools available.'}
		</div>
	{:else}
		<ul class="tool-list" role="list" aria-label="MCP tools">
			{#each filteredTools() as tool (tool.name)}
				{@const isOpen = expanded.has(tool.name)}
				<li class="tool-item">
					<button
						class="tool-header"
						class:open={isOpen}
						onclick={() => toggleExpanded(tool.name)}
						aria-expanded={isOpen}
					>
						<span class="tool-icon"><Box size={14} /></span>
						<span class="tool-name">{tool.name}</span>
						<span class="tool-category" class:mutation={tool.category === 'mutation'}>
							{tool.category}
						</span>
						<span class="chevron">
							{#if isOpen}
								<ChevronDown size={14} />
							{:else}
								<ChevronRight size={14} />
							{/if}
						</span>
					</button>

					{#if isOpen}
						<div class="tool-body">
							<p class="tool-desc">{tool.description}</p>
							{#if tool.params.length > 0}
								<table class="params-table" aria-label="Parameters for {tool.name}">
									<thead>
										<tr>
											<th>Parameter</th>
											<th>Type</th>
											<th>Required</th>
											<th>Description</th>
										</tr>
									</thead>
									<tbody>
										{#each tool.params as param}
											<tr>
												<td class="param-name">{param.name}</td>
												<td class="param-type">{param.type}</td>
												<td class="param-req">
													{#if param.required}
														<span class="req-badge">required</span>
													{:else}
														<span class="opt-label">optional</span>
													{/if}
												</td>
												<td class="param-desc">{param.description}</td>
											</tr>
										{/each}
									</tbody>
								</table>
							{:else}
								<p class="no-params">No parameters.</p>
							{/if}
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.browser {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.browser-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}

	h2 {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.search-wrap {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-wrap :global(.search-icon) {
		position: absolute;
		left: 10px;
		color: var(--color-text-subtle);
		pointer-events: none;
	}

	.search {
		padding: 6px 10px 6px 30px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background-color: var(--color-surface);
		color: var(--color-text);
		font-size: 13px;
		width: 220px;
		outline: none;
		transition: border-color 0.15s ease;
	}

	.search:focus {
		border-color: var(--color-accent);
	}

	.search::placeholder {
		color: var(--color-text-subtle);
	}

	.state-msg {
		padding: 32px;
		text-align: center;
		color: var(--color-text-muted);
		font-size: 13px;
	}

	.state-error {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		border-radius: 6px;
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 13px;
	}

	.tool-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.tool-item {
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		overflow: hidden;
		background-color: var(--color-surface);
	}

	.tool-header {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 10px 14px;
		background: none;
		border: none;
		cursor: pointer;
		text-align: left;
		color: var(--color-text);
		transition: background-color 0.1s ease;
	}

	.tool-header:hover,
	.tool-header.open {
		background-color: var(--color-surface-hover);
	}

	.tool-icon {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.tool-name {
		font-family: var(--font-mono, monospace);
		font-size: 13px;
		font-weight: 500;
		flex: 1;
	}

	.tool-category {
		font-size: 11px;
		font-weight: 600;
		padding: 2px 8px;
		border-radius: 3px;
		background-color: var(--color-surface-active);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.tool-category.mutation {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	.chevron {
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	.tool-body {
		padding: 12px 14px 14px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.tool-desc {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0 0 12px;
		line-height: 1.5;
	}

	.no-params {
		font-size: 12px;
		color: var(--color-text-subtle);
		margin: 0;
	}

	.params-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12px;
	}

	.params-table th {
		text-align: left;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-subtle);
		padding: 6px 10px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.params-table td {
		padding: 8px 10px;
		border-bottom: 1px solid var(--color-border-subtle);
		color: var(--color-text);
		vertical-align: top;
	}

	.params-table tr:last-child td {
		border-bottom: none;
	}

	.param-name {
		font-family: var(--font-mono, monospace);
		font-weight: 500;
		color: var(--color-accent);
	}

	.param-type {
		font-family: var(--font-mono, monospace);
		color: var(--color-text-muted);
	}

	.param-desc {
		color: var(--color-text-muted);
		line-height: 1.4;
	}

	.req-badge {
		font-size: 10px;
		font-weight: 600;
		padding: 1px 6px;
		border-radius: 3px;
		background-color: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	.opt-label {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	@media (max-width: 640px) {
		.browser-header {
			flex-direction: column;
			align-items: flex-start;
		}

		.search {
			width: 100%;
		}

		.params-table th:last-child,
		.params-table td:last-child {
			display: none;
		}
	}
</style>
