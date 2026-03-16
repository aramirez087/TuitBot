<script lang="ts">
	import { ListChecks, ChevronDown, ChevronUp, Power } from "lucide-svelte";
	import { policy, updatePolicy } from "$lib/stores/mcp";
	import type { McpPolicyRule } from "$lib/api";

	let expandedRule = $state<string | null>(null);
	let pendingUpdate = $state(false);

	async function toggleRule(rule: McpPolicyRule) {
		if (!$policy) return;
		pendingUpdate = true;
		try {
			const updated = $policy.rules.map((r) =>
				r.id === rule.id ? { ...r, enabled: !r.enabled } : r,
			);
			await updatePolicy({ rules: updated });
		} finally {
			pendingUpdate = false;
		}
	}

	function actionLabel(action: McpPolicyRule["action"]): string {
		if (typeof action === "string") return action;
		if (action.type === "allow") return "Allow";
		if (action.type === "deny") return "Deny";
		if (action.type === "require_approval") return "Require Approval";
		if (action.type === "dry_run") return "Dry Run";
		return String(action.type);
	}

	function actionClass(action: McpPolicyRule["action"]): string {
		if (typeof action === "string") return action;
		if (action.type === "allow") return "allow";
		if (action.type === "deny") return "deny";
		if (action.type === "require_approval") return "approval";
		if (action.type === "dry_run") return "dry-run";
		return "";
	}
</script>

{#if $policy && $policy.rules && $policy.rules.length > 0}
	<section class="card">
		<h2>
			<ListChecks size={16} />
			Policy Rules
		</h2>
		<span class="section-hint"
			>Rules are evaluated by priority (lowest first). First match
			wins.</span
		>
		<div class="rule-list">
			{#each $policy.rules as rule}
				<div class="rule-item" class:disabled={!rule.enabled}>
					<div class="rule-header">
						<div class="rule-info">
							<span class="rule-priority"
								>P{rule.priority}</span
							>
							<span class="rule-label">{rule.label}</span>
							<span
								class="policy-badge {actionClass(rule.action)}"
							>
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
								onclick={() =>
									(expandedRule =
										expandedRule === rule.id
											? null
											: rule.id)}
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
									<span
										>{rule.conditions.tools.join(
											", ",
										)}</span
									>
								</div>
							{/if}
							{#if rule.conditions.categories && rule.conditions.categories.length > 0}
								<div class="detail-row">
									<span class="detail-label"
										>Categories:</span
									>
									<span
										>{rule.conditions.categories.join(
											", ",
										)}</span
									>
								</div>
							{/if}
							{#if rule.conditions.modes && rule.conditions.modes.length > 0}
								<div class="detail-row">
									<span class="detail-label">Modes:</span>
									<span
										>{rule.conditions.modes.join(
											", ",
										)}</span
									>
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</section>
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
</style>
