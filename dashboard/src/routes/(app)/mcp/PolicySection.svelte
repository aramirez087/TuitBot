<script lang="ts">
	import {
		Shield,
		Lock,
		Unlock,
		FlaskConical,
		ListChecks,
		Ban,
		Layers,
		ChevronDown,
		ChevronUp,
		Power
	} from 'lucide-svelte';
	import { policy, updatePolicy, templates, loadTemplates, applyTemplate } from '$lib/stores/mcp';
	import type { McpPolicyRule, McpPolicyTemplate } from '$lib/api';

	let confirmDialog = $state<{ action: string; message: string; onConfirm: () => void } | null>(
		null
	);
	let pendingUpdate = $state(false);
	let expandedRule = $state<string | null>(null);
	let templateLoading = $state(false);

	import { onMount } from 'svelte';
	onMount(() => {
		loadTemplates();
	});

	async function doUpdate(patch: Record<string, unknown>) {
		pendingUpdate = true;
		try {
			await updatePolicy(patch);
		} finally {
			pendingUpdate = false;
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

	async function handleApplyTemplate(name: string) {
		if (name === 'growth_aggressive') {
			confirmDialog = {
				action: 'Apply Growth Aggressive Template',
				message:
					'This template allows most mutations without approval and raises rate limits significantly. Only use for established accounts.',
				onConfirm: async () => {
					confirmDialog = null;
					templateLoading = true;
					try {
						await applyTemplate(name);
					} finally {
						templateLoading = false;
					}
				}
			};
			return;
		}
		templateLoading = true;
		try {
			await applyTemplate(name);
		} finally {
			templateLoading = false;
		}
	}

	async function toggleRule(rule: McpPolicyRule) {
		if (!$policy) return;
		const updated = $policy.rules.map((r) =>
			r.id === rule.id ? { ...r, enabled: !r.enabled } : r
		);
		await doUpdate({ rules: updated });
	}

	function actionLabel(action: McpPolicyRule['action']): string {
		if (typeof action === 'string') return action;
		if (action.type === 'allow') return 'Allow';
		if (action.type === 'deny') return 'Deny';
		if (action.type === 'require_approval') return 'Require Approval';
		if (action.type === 'dry_run') return 'Dry Run';
		return String(action.type);
	}

	function actionClass(action: McpPolicyRule['action']): string {
		if (typeof action === 'string') return action;
		if (action.type === 'allow') return 'allow';
		if (action.type === 'deny') return 'deny';
		if (action.type === 'require_approval') return 'approval';
		if (action.type === 'dry_run') return 'dry-run';
		return '';
	}
</script>

{#if $policy}
	<!-- Template Picker -->
	<section class="card">
		<h2>
			<Layers size={16} />
			Policy Template
		</h2>
		<span class="section-hint">Select a pre-built policy profile as a baseline.</span>
		{#if $policy.template}
			<div class="current-template">
				Active: <strong>{$policy.template}</strong>
			</div>
		{/if}
		<div class="template-grid">
			{#each $templates as tpl}
				<button
					class="template-card"
					class:active={$policy.template === tpl.name}
					disabled={templateLoading}
					onclick={() => handleApplyTemplate(tpl.name)}
				>
					<div class="template-name">{tpl.name.replace(/_/g, ' ')}</div>
					<div class="template-desc">{tpl.description}</div>
					<div class="template-meta">
						{tpl.rules.length} rules, {tpl.rate_limits.length} rate limits
					</div>
				</button>
			{/each}
		</div>
	</section>

	<!-- Policy Configuration -->
	<section class="card">
		<h2>
			<Shield size={16} />
			Policy Configuration
		</h2>

		<div class="policy-grid">
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

	<!-- Active Rules -->
	{#if $policy.rules && $policy.rules.length > 0}
		<section class="card">
			<h2>
				<ListChecks size={16} />
				Policy Rules
			</h2>
			<span class="section-hint">Rules are evaluated by priority (lowest first). First match wins.</span>
			<div class="rule-list">
				{#each $policy.rules as rule}
					<div class="rule-item" class:disabled={!rule.enabled}>
						<div class="rule-header">
							<div class="rule-info">
								<span class="rule-priority">P{rule.priority}</span>
								<span class="rule-label">{rule.label}</span>
								<span class="policy-badge {actionClass(rule.action)}">
									{actionLabel(rule.action)}
								</span>
							</div>
							<div class="rule-actions">
								<button
									class="toggle-btn small"
									class:active={rule.enabled}
									onclick={() => toggleRule(rule)}
									disabled={pendingUpdate}
								>
									<Power size={12} />
								</button>
								<button
									class="expand-btn"
									onclick={() => (expandedRule = expandedRule === rule.id ? null : rule.id)}
								>
									{#if expandedRule === rule.id}
										<ChevronUp size={14} />
									{:else}
										<ChevronDown size={14} />
									{/if}
								</button>
							</div>
						</div>
						{#if expandedRule === rule.id}
							<div class="rule-details">
								<div class="detail-row">
									<span class="detail-label">ID:</span>
									<code>{rule.id}</code>
								</div>
								{#if rule.conditions.tools && rule.conditions.tools.length > 0}
									<div class="detail-row">
										<span class="detail-label">Tools:</span>
										<span>{rule.conditions.tools.join(', ')}</span>
									</div>
								{/if}
								{#if rule.conditions.categories && rule.conditions.categories.length > 0}
									<div class="detail-row">
										<span class="detail-label">Categories:</span>
										<span>{rule.conditions.categories.join(', ')}</span>
									</div>
								{/if}
								{#if rule.conditions.modes && rule.conditions.modes.length > 0}
									<div class="detail-row">
										<span class="detail-label">Modes:</span>
										<span>{rule.conditions.modes.join(', ')}</span>
									</div>
								{/if}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</section>
	{/if}

	<!-- Approval Requirements -->
	<section class="card">
		<h2>
			<ListChecks size={16} />
			Approval Requirements
		</h2>
		<span class="section-hint">Tools listed here must be approved before execution.</span>
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

<!-- Confirm Dialog -->
{#if confirmDialog}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="dialog-overlay" onclick={() => (confirmDialog = null)}>
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<div class="dialog" onclick={(e) => e.stopPropagation()}>
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
	.card {
		padding: 18px;
		background-color: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
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

	.section-hint {
		display: block;
		font-size: 12px;
		color: var(--color-text-muted);
		margin-bottom: 12px;
	}

	/* Template picker */
	.current-template {
		font-size: 12px;
		color: var(--color-text-muted);
		margin-bottom: 12px;
	}

	.current-template strong {
		color: var(--color-accent);
		text-transform: capitalize;
	}

	.template-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
	}

	.template-card {
		padding: 14px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: var(--color-base);
		cursor: pointer;
		text-align: left;
		transition: all 0.15s;
	}

	.template-card:hover:not(:disabled) {
		border-color: var(--color-accent);
	}

	.template-card.active {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
	}

	.template-card:disabled {
		opacity: 0.5;
		cursor: wait;
	}

	.template-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		text-transform: capitalize;
		margin-bottom: 4px;
	}

	.template-desc {
		font-size: 11px;
		color: var(--color-text-muted);
		line-height: 1.4;
		margin-bottom: 6px;
	}

	.template-meta {
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	/* Policy grid */
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

	.toggle-btn.small {
		padding: 3px 8px;
		font-size: 11px;
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

	/* Rule list */
	.rule-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.rule-item {
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		padding: 10px 14px;
		background: var(--color-base);
	}

	.rule-item.disabled {
		opacity: 0.5;
	}

	.rule-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.rule-info {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.rule-priority {
		font-size: 10px;
		font-weight: 700;
		font-family: var(--font-mono, monospace);
		color: var(--color-text-subtle);
		background: var(--color-surface-active);
		padding: 1px 6px;
		border-radius: 3px;
	}

	.rule-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.rule-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.expand-btn {
		border: none;
		background: none;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 2px;
	}

	.expand-btn:hover {
		color: var(--color-text);
	}

	.rule-details {
		margin-top: 10px;
		padding-top: 10px;
		border-top: 1px solid var(--color-border-subtle);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.detail-row {
		display: flex;
		gap: 8px;
		font-size: 12px;
	}

	.detail-label {
		font-weight: 600;
		color: var(--color-text-muted);
		min-width: 80px;
	}

	code {
		font-family: var(--font-mono, monospace);
		font-size: 11px;
		color: var(--color-text-muted);
	}

	/* Tag lists */
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

	/* Policy badges */
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

	@media (max-width: 800px) {
		.template-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
