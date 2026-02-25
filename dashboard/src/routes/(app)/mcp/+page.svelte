<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Shield,
		Activity,
		AlertTriangle,
		Clock,
		CheckCircle,
		XCircle,
		Gauge,
		Hash,
		Lock,
		Unlock,
		FlaskConical,
		ListChecks,
		Ban
	} from 'lucide-svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import {
		policy,
		summary,
		metrics,
		errors,
		recentExecutions,
		loading,
		error,
		loadMcpData,
		updatePolicy,
		startAutoRefresh,
		stopAutoRefresh
	} from '$lib/stores/mcp';

	let hours = $state(24);
	let activeTab = $state<'overview' | 'tools' | 'errors' | 'executions'>('overview');
	let confirmDialog = $state<{ action: string; message: string; onConfirm: () => void } | null>(
		null
	);
	let pendingUpdate = $state(false);

	onMount(() => {
		loadMcpData(hours);
		startAutoRefresh(30_000, hours);
	});

	onDestroy(() => {
		stopAutoRefresh();
	});

	function handleHoursChange(newHours: number) {
		hours = newHours;
		loadMcpData(hours);
		stopAutoRefresh();
		startAutoRefresh(30_000, hours);
	}

	function formatRate(rate: number): string {
		return (rate * 100).toFixed(1) + '%';
	}

	function formatLatency(ms: number): string {
		if (ms < 1) return '<1ms';
		if (ms >= 1000) return (ms / 1000).toFixed(1) + 's';
		return Math.round(ms) + 'ms';
	}

	function formatTime(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
		} catch {
			return iso;
		}
	}

	function formatDate(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
		} catch {
			return iso;
		}
	}

	async function toggleEnforcement() {
		if (!$policy) return;
		const newValue = !$policy.enforce_for_mutations;

		if (!newValue) {
			confirmDialog = {
				action: 'Disable Policy Enforcement',
				message:
					'This will allow all MCP mutations without policy checks. Mutations will not be rate-limited, blocked, or routed to approval.',
				onConfirm: async () => {
					confirmDialog = null;
					await doUpdate({ enforce_for_mutations: false });
				}
			};
			return;
		}
		await doUpdate({ enforce_for_mutations: true });
	}

	async function toggleDryRun() {
		if (!$policy) return;
		await doUpdate({ dry_run_mutations: !$policy.dry_run_mutations });
	}

	async function doUpdate(patch: Record<string, unknown>) {
		pendingUpdate = true;
		try {
			await updatePolicy(patch);
		} finally {
			pendingUpdate = false;
		}
	}

	async function removeBlockedTool(tool: string) {
		if (!$policy) return;
		const updated = $policy.blocked_tools.filter((t) => t !== tool);
		await doUpdate({ blocked_tools: updated });
	}

	let newBlockedTool = $state('');
	async function addBlockedTool() {
		if (!$policy || !newBlockedTool.trim()) return;
		const tool = newBlockedTool.trim();
		if ($policy.blocked_tools.includes(tool)) return;
		const updated = [...$policy.blocked_tools, tool];
		newBlockedTool = '';
		await doUpdate({ blocked_tools: updated });
	}

	async function removeApprovalTool(tool: string) {
		if (!$policy) return;

		confirmDialog = {
			action: 'Remove Approval Requirement',
			message: `"${tool}" will no longer require approval before execution. It will execute immediately when called by MCP agents.`,
			onConfirm: async () => {
				confirmDialog = null;
				const updated = $policy!.require_approval_for.filter((t) => t !== tool);
				await doUpdate({ require_approval_for: updated });
			}
		};
	}

	let newApprovalTool = $state('');
	async function addApprovalTool() {
		if (!$policy || !newApprovalTool.trim()) return;
		const tool = newApprovalTool.trim();
		if ($policy.require_approval_for.includes(tool)) return;
		const updated = [...$policy.require_approval_for, tool];
		newApprovalTool = '';
		await doUpdate({ require_approval_for: updated });
	}

	const policyDecisionCounts = $derived(() => {
		if (!$summary) return [];
		return Object.entries($summary.policy_decisions)
			.sort((a, b) => b[1] - a[1])
			.map(([decision, count]) => ({ decision, count }));
	});
</script>

<div class="page">
	<div class="page-header">
		<div class="page-title">
			<h1>MCP Governance</h1>
			<span class="subtitle">Policy, telemetry, and tool execution insights</span>
		</div>
		<div class="header-controls">
			<div class="tab-switcher">
				<button
					class="tab-btn"
					class:active={activeTab === 'overview'}
					onclick={() => (activeTab = 'overview')}>Overview</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'tools'}
					onclick={() => (activeTab = 'tools')}>Tools</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'errors'}
					onclick={() => (activeTab = 'errors')}>Errors</button
				>
				<button
					class="tab-btn"
					class:active={activeTab === 'executions'}
					onclick={() => (activeTab = 'executions')}>Executions</button
				>
			</div>
			<div class="period-selector">
				<button
					class="period-btn"
					class:active={hours === 1}
					onclick={() => handleHoursChange(1)}>1h</button
				>
				<button
					class="period-btn"
					class:active={hours === 24}
					onclick={() => handleHoursChange(24)}>24h</button
				>
				<button
					class="period-btn"
					class:active={hours === 168}
					onclick={() => handleHoursChange(168)}>7d</button
				>
			</div>
		</div>
	</div>

	{#if $error}
		<ErrorState message={$error} onretry={() => loadMcpData(hours)} />
	{:else if $loading}
		<div class="loading">Loading MCP data...</div>
	{:else if activeTab === 'overview'}
		<!-- Stats Row -->
		<div class="stat-grid">
			<StatCard label="Total Calls" value={$summary?.total_calls ?? 0}>
				{#snippet icon()}<Hash size={18} />{/snippet}
			</StatCard>
			<StatCard label="Success Rate" value={formatRate($summary?.overall_success_rate ?? 0)}>
				{#snippet icon()}<CheckCircle size={18} />{/snippet}
			</StatCard>
			<StatCard label="Avg Latency" value={formatLatency($summary?.avg_latency_ms ?? 0)}>
				{#snippet icon()}<Gauge size={18} />{/snippet}
			</StatCard>
			<StatCard label="Unique Tools" value={$summary?.unique_tools ?? 0}>
				{#snippet icon()}<Activity size={18} />{/snippet}
			</StatCard>
		</div>

		<!-- Policy Section -->
		{#if $policy}
			<section class="card">
				<h2>
					<Shield size={16} />
					Policy Configuration
				</h2>

				<div class="policy-grid">
					<!-- Enforcement toggle -->
					<div class="policy-item">
						<div class="policy-header">
							<span class="policy-label">Policy Enforcement</span>
							<button
								class="toggle-btn"
								class:active={$policy.enforce_for_mutations}
								onclick={toggleEnforcement}
								disabled={pendingUpdate}
							>
								{#if $policy.enforce_for_mutations}
									<Lock size={14} />
									<span>Enabled</span>
								{:else}
									<Unlock size={14} />
									<span>Disabled</span>
								{/if}
							</button>
						</div>
						<span class="policy-hint">
							{#if $policy.enforce_for_mutations}
								Mutations are checked against rate limits, block lists, and approval requirements.
							{:else}
								<span class="warning-text">All mutations execute without policy checks.</span>
							{/if}
						</span>
					</div>

					<!-- Dry-run toggle -->
					<div class="policy-item">
						<div class="policy-header">
							<span class="policy-label">Dry-Run Mode</span>
							<button
								class="toggle-btn"
								class:active={$policy.dry_run_mutations}
								onclick={toggleDryRun}
								disabled={pendingUpdate}
							>
								<FlaskConical size={14} />
								<span>{$policy.dry_run_mutations ? 'Active' : 'Off'}</span>
							</button>
						</div>
						<span class="policy-hint">
							{#if $policy.dry_run_mutations}
								Mutations return simulated responses without executing.
							{:else}
								Mutations execute normally when allowed by policy.
							{/if}
						</span>
					</div>

					<!-- Rate limit -->
					<div class="policy-item">
						<div class="policy-header">
							<span class="policy-label">Rate Limit</span>
							<span class="rate-badge">
								{$policy.rate_limit.used} / {$policy.rate_limit.max} per hour
							</span>
						</div>
						<div class="rate-bar">
							<div
								class="rate-fill"
								class:warning={$policy.rate_limit.used / $policy.rate_limit.max > 0.75}
								class:danger={$policy.rate_limit.used / $policy.rate_limit.max > 0.9}
								style="width: {Math.min(($policy.rate_limit.used / Math.max($policy.rate_limit.max, 1)) * 100, 100)}%"
							></div>
						</div>
					</div>

					<!-- Mode -->
					<div class="policy-item">
						<div class="policy-header">
							<span class="policy-label">Operating Mode</span>
							<span class="mode-badge" class:composer={$policy.mode === 'composer'}>
								{$policy.mode}
							</span>
						</div>
						<span class="policy-hint">
							{#if $policy.mode === 'composer'}
								Read-only + posting queue. All mutations route to approval.
							{:else}
								Full automation with policy-based gating.
							{/if}
						</span>
					</div>
				</div>
			</section>

			<!-- Approval Requirements -->
			<section class="card">
				<h2>
					<ListChecks size={16} />
					Approval Requirements
				</h2>
				<span class="section-hint"
					>Tools listed here must be approved before execution.</span
				>
				<div class="tag-list">
					{#each $policy.require_approval_for as tool}
						<span class="tag">
							{tool}
							<button class="tag-remove" onclick={() => removeApprovalTool(tool)}>×</button>
						</span>
					{/each}
					<form class="tag-input-form" onsubmit={(e) => { e.preventDefault(); addApprovalTool(); }}>
						<input
							type="text"
							class="tag-input"
							placeholder="Add tool..."
							bind:value={newApprovalTool}
						/>
					</form>
				</div>
			</section>

			<!-- Blocked Tools -->
			<section class="card">
				<h2>
					<Ban size={16} />
					Blocked Tools
				</h2>
				<span class="section-hint">These tools are completely blocked from execution.</span>
				<div class="tag-list">
					{#each $policy.blocked_tools as tool}
						<span class="tag danger">
							{tool}
							<button class="tag-remove" onclick={() => removeBlockedTool(tool)}>×</button>
						</span>
					{/each}
					{#if $policy.blocked_tools.length === 0}
						<span class="empty-hint">No tools blocked</span>
					{/if}
					<form class="tag-input-form" onsubmit={(e) => { e.preventDefault(); addBlockedTool(); }}>
						<input
							type="text"
							class="tag-input"
							placeholder="Block tool..."
							bind:value={newBlockedTool}
						/>
					</form>
				</div>
			</section>
		{/if}

		<!-- Policy Decision Breakdown -->
		{#if policyDecisionCounts().length > 0}
			<section class="card">
				<h2>
					<Shield size={16} />
					Policy Decisions
				</h2>
				<div class="decision-grid">
					{#each policyDecisionCounts() as { decision, count }}
						<div class="decision-item">
							<span class="decision-label">{decision}</span>
							<span class="decision-count">{count}</span>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{:else if activeTab === 'tools'}
		<!-- Tool Metrics Table -->
		{#if $metrics.length > 0}
			<section class="card">
				<h2>Tool Performance</h2>
				<div class="table-wrapper">
					<table>
						<thead>
							<tr>
								<th>Tool</th>
								<th>Category</th>
								<th class="right">Calls</th>
								<th class="right">Success</th>
								<th class="right">Failures</th>
								<th class="right">Rate</th>
								<th class="right">Avg</th>
								<th class="right">P50</th>
								<th class="right">P95</th>
							</tr>
						</thead>
						<tbody>
							{#each $metrics as tool}
								<tr>
									<td class="tool-name">{tool.tool_name}</td>
									<td>
										<span
											class="category-badge"
											class:mutation={tool.category === 'mutation'}
										>
											{tool.category}
										</span>
									</td>
									<td class="right">{tool.total_calls}</td>
									<td class="right text-success">{tool.success_count}</td>
									<td class="right">
										{#if tool.failure_count > 0}
											<span class="text-danger">{tool.failure_count}</span>
										{:else}
											<span class="text-muted">0</span>
										{/if}
									</td>
									<td class="right">
										<span
											class:text-success={tool.success_rate >= 0.95}
											class:text-warning={tool.success_rate >= 0.8 &&
												tool.success_rate < 0.95}
											class:text-danger={tool.success_rate < 0.8}
										>
											{formatRate(tool.success_rate)}
										</span>
									</td>
									<td class="right">{formatLatency(tool.avg_latency_ms)}</td>
									<td class="right">{formatLatency(tool.p50_latency_ms)}</td>
									<td class="right">{formatLatency(tool.p95_latency_ms)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{:else}
			<div class="empty-state">
				<Activity size={32} />
				<p>No tool executions recorded in this time window.</p>
			</div>
		{/if}
	{:else if activeTab === 'errors'}
		<!-- Error Breakdown -->
		{#if $errors.length > 0}
			<section class="card">
				<h2>Error Breakdown</h2>
				<div class="table-wrapper">
					<table>
						<thead>
							<tr>
								<th>Tool</th>
								<th>Error Code</th>
								<th class="right">Count</th>
								<th class="right">Last Seen</th>
							</tr>
						</thead>
						<tbody>
							{#each $errors as err}
								<tr>
									<td class="tool-name">{err.tool_name}</td>
									<td>
										<span class="error-badge">{err.error_code}</span>
									</td>
									<td class="right">{err.count}</td>
									<td class="right text-muted">
										{formatDate(err.latest_at)} {formatTime(err.latest_at)}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{:else}
			<div class="empty-state">
				<CheckCircle size={32} />
				<p>No errors recorded in this time window.</p>
			</div>
		{/if}
	{:else if activeTab === 'executions'}
		<!-- Recent Executions -->
		{#if $recentExecutions.length > 0}
			<section class="card">
				<h2>Recent Executions</h2>
				<div class="table-wrapper">
					<table>
						<thead>
							<tr>
								<th>Time</th>
								<th>Tool</th>
								<th>Category</th>
								<th>Status</th>
								<th>Policy</th>
								<th class="right">Latency</th>
							</tr>
						</thead>
						<tbody>
							{#each $recentExecutions as entry}
								<tr>
									<td class="text-muted">
										{formatDate(entry.created_at)}
										{formatTime(entry.created_at)}
									</td>
									<td class="tool-name">{entry.tool_name}</td>
									<td>
										<span
											class="category-badge"
											class:mutation={entry.category === 'mutation'}
										>
											{entry.category}
										</span>
									</td>
									<td>
										{#if entry.success}
											<span class="status-badge success">
												<CheckCircle size={12} />
												OK
											</span>
										{:else}
											<span class="status-badge failure">
												<XCircle size={12} />
												{entry.error_code ?? 'Error'}
											</span>
										{/if}
									</td>
									<td>
										{#if entry.policy_decision}
											<span
												class="policy-badge"
												class:allow={entry.policy_decision === 'allow'}
												class:deny={entry.policy_decision === 'deny'}
												class:dry-run={entry.policy_decision === 'dry_run'}
												class:approval={entry.policy_decision === 'route_to_approval'}
											>
												{entry.policy_decision}
											</span>
										{:else}
											<span class="text-muted">—</span>
										{/if}
									</td>
									<td class="right">{formatLatency(entry.latency_ms)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
		{:else}
			<div class="empty-state">
				<Clock size={32} />
				<p>No recent executions recorded.</p>
			</div>
		{/if}
	{/if}
</div>

<!-- Confirm Dialog -->
{#if confirmDialog}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="dialog-overlay" onclick={() => (confirmDialog = null)}>
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<div class="dialog" onclick={(e) => e.stopPropagation()}>
			<div class="dialog-icon">
				<AlertTriangle size={24} />
			</div>
			<h3>{confirmDialog.action}</h3>
			<p>{confirmDialog.message}</p>
			<div class="dialog-actions">
				<button class="btn-cancel" onclick={() => (confirmDialog = null)}>Cancel</button>
				<button class="btn-confirm" onclick={confirmDialog.onConfirm}>Confirm</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 20px;
		max-width: 1000px;
	}

	.page-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 16px;
	}

	.page-title {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	h1 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	h2 {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 14px 0;
	}

	.header-controls {
		display: flex;
		gap: 8px;
		align-items: center;
		flex-shrink: 0;
	}

	.stat-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
	}

	.card {
		padding: 18px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
	}

	.tab-switcher {
		display: flex;
		gap: 4px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 3px;
	}

	.tab-btn {
		padding: 4px 14px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.tab-btn.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.tab-btn:hover:not(.active) {
		color: var(--color-text);
	}

	.period-selector {
		display: flex;
		gap: 4px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 3px;
	}

	.period-btn {
		padding: 4px 12px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.period-btn.active {
		background-color: var(--color-surface-active);
		color: var(--color-text);
	}

	.period-btn:hover:not(.active) {
		color: var(--color-text);
	}

	/* Policy section */
	.policy-grid {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.policy-item {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.policy-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.policy-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
	}

	.policy-hint {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.warning-text {
		color: var(--color-warning);
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.toggle-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-text);
	}

	.toggle-btn.active {
		border-color: var(--color-success);
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 8%, transparent);
	}

	.toggle-btn:disabled {
		opacity: 0.5;
		cursor: wait;
	}

	.rate-badge {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		font-family: var(--font-mono, monospace);
	}

	.rate-bar {
		height: 6px;
		background: var(--color-surface-active);
		border-radius: 3px;
		overflow: hidden;
		margin-top: 4px;
	}

	.rate-fill {
		height: 100%;
		background: var(--color-accent);
		border-radius: 3px;
		transition: width 0.3s ease;
	}

	.rate-fill.warning {
		background: var(--color-warning);
	}

	.rate-fill.danger {
		background: var(--color-danger);
	}

	.mode-badge {
		display: inline-block;
		padding: 3px 10px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 600;
		text-transform: capitalize;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.mode-badge.composer {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	/* Tag lists */
	.section-hint {
		display: block;
		font-size: 12px;
		color: var(--color-text-muted);
		margin-bottom: 12px;
	}

	.tag-list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
	}

	.tag {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border-radius: 4px;
		font-size: 12px;
		font-weight: 500;
		font-family: var(--font-mono, monospace);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}

	.tag.danger {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
	}

	.tag-remove {
		border: none;
		background: none;
		color: inherit;
		cursor: pointer;
		font-size: 14px;
		padding: 0 2px;
		opacity: 0.6;
		line-height: 1;
	}

	.tag-remove:hover {
		opacity: 1;
	}

	.tag-input-form {
		display: inline-flex;
	}

	.tag-input {
		width: 120px;
		padding: 4px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: var(--color-base);
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-mono, monospace);
	}

	.tag-input::placeholder {
		color: var(--color-text-subtle);
	}

	.empty-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	/* Decision breakdown */
	.decision-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
		gap: 8px;
	}

	.decision-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background: var(--color-surface-active);
		border-radius: 6px;
	}

	.decision-label {
		font-size: 12px;
		color: var(--color-text-muted);
		text-transform: capitalize;
	}

	.decision-count {
		font-size: 14px;
		font-weight: 700;
		color: var(--color-text);
		font-family: var(--font-mono, monospace);
	}

	/* Tables */
	.table-wrapper {
		overflow-x: auto;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	th {
		text-align: left;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	td {
		padding: 10px 12px;
		color: var(--color-text);
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.right {
		text-align: right;
	}

	.tool-name {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		font-weight: 500;
	}

	.category-badge {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 3px;
		font-size: 11px;
		font-weight: 600;
		background-color: var(--color-surface-active);
		color: var(--color-text-muted);
	}

	.category-badge.mutation {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	.error-badge {
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		padding: 2px 8px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		font-weight: 500;
	}

	.status-badge.success {
		color: var(--color-success);
	}

	.status-badge.failure {
		color: var(--color-danger);
	}

	.policy-badge {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 3px;
		font-size: 11px;
		font-weight: 600;
		background: var(--color-surface-active);
		color: var(--color-text-muted);
	}

	.policy-badge.allow {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.policy-badge.deny {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	.policy-badge.dry-run {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	.policy-badge.approval {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.text-success {
		color: var(--color-success);
	}

	.text-warning {
		color: var(--color-warning);
	}

	.text-danger {
		color: var(--color-danger);
	}

	.text-muted {
		color: var(--color-text-subtle);
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 48px 24px;
		color: var(--color-text-muted);
		text-align: center;
	}

	.empty-state p {
		margin: 0;
		font-size: 14px;
	}

	.loading {
		text-align: center;
		padding: 48px 0;
		color: var(--color-text-muted);
		font-size: 14px;
	}

	/* Confirm dialog */
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.dialog {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 24px;
		max-width: 420px;
		width: 90%;
		text-align: center;
	}

	.dialog-icon {
		color: var(--color-warning);
		margin-bottom: 8px;
	}

	.dialog h3 {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 8px 0;
	}

	.dialog p {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
		margin: 0 0 20px 0;
	}

	.dialog-actions {
		display: flex;
		gap: 8px;
		justify-content: center;
	}

	.btn-cancel,
	.btn-confirm {
		padding: 8px 20px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.btn-cancel {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text-muted);
	}

	.btn-cancel:hover {
		border-color: var(--color-text-subtle);
		color: var(--color-text);
	}

	.btn-confirm {
		background: var(--color-danger);
		border: 1px solid var(--color-danger);
		color: white;
	}

	.btn-confirm:hover {
		opacity: 0.9;
	}

	/* Responsive */
	@media (max-width: 800px) {
		.stat-grid {
			grid-template-columns: repeat(2, 1fr);
		}

		.page-header {
			flex-direction: column;
			gap: 12px;
		}

		.header-controls {
			flex-wrap: wrap;
		}
	}
</style>
