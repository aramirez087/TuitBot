<script lang="ts">
	import { Layers, ListChecks, Ban } from "lucide-svelte";
	import {
		policy,
		updatePolicy,
		templates,
		loadTemplates,
		applyTemplate,
	} from "$lib/stores/mcp";
	import { onMount } from "svelte";
	import PolicyConfigPanel from "./PolicyConfigPanel.svelte";
	import PolicyRuleList from "./PolicyRuleList.svelte";
	import ConfirmDialog from "./ConfirmDialog.svelte";

	let confirmDialog = $state<{
		action: string;
		message: string;
		onConfirm: () => void;
	} | null>(null);
	let pendingUpdate = $state(false);
	let templateLoading = $state(false);
	let newBlockedTool = $state("");
	let newApprovalTool = $state("");

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

	async function removeBlockedTool(tool: string) {
		if (!$policy) return;
		const updated = $policy.blocked_tools.filter((t) => t !== tool);
		await doUpdate({ blocked_tools: updated });
	}

	async function addBlockedTool() {
		if (!$policy || !newBlockedTool.trim()) return;
		const tool = newBlockedTool.trim();
		if ($policy.blocked_tools.includes(tool)) return;
		const updated = [...$policy.blocked_tools, tool];
		newBlockedTool = "";
		await doUpdate({ blocked_tools: updated });
	}

	async function removeApprovalTool(tool: string) {
		if (!$policy) return;
		confirmDialog = {
			action: "Remove Approval Requirement",
			message: `"${tool}" will no longer require approval before execution. It will execute immediately when called by MCP agents.`,
			onConfirm: async () => {
				confirmDialog = null;
				const updated = $policy!.require_approval_for.filter(
					(t) => t !== tool,
				);
				await doUpdate({ require_approval_for: updated });
			},
		};
	}

	async function addApprovalTool() {
		if (!$policy || !newApprovalTool.trim()) return;
		const tool = newApprovalTool.trim();
		if ($policy.require_approval_for.includes(tool)) return;
		const updated = [...$policy.require_approval_for, tool];
		newApprovalTool = "";
		await doUpdate({ require_approval_for: updated });
	}

	async function handleApplyTemplate(name: string) {
		if (name === "growth_aggressive") {
			confirmDialog = {
				action: "Apply Growth Aggressive Template",
				message:
					"This template allows most mutations without approval and raises rate limits significantly. Only use for established accounts.",
				onConfirm: async () => {
					confirmDialog = null;
					templateLoading = true;
					try {
						await applyTemplate(name);
					} finally {
						templateLoading = false;
					}
				},
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
</script>

{#if $policy}
	<!-- Template Picker -->
	<section class="card">
		<h2>
			<Layers size={16} />
			Policy Template
		</h2>
		<span class="section-hint"
			>Select a pre-built policy profile as a baseline.</span
		>
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
					<div class="template-name">
						{tpl.name.replace(/_/g, " ")}
					</div>
					<div class="template-desc">{tpl.description}</div>
					<div class="template-meta">
						{tpl.rules.length} rules, {tpl.rate_limits.length} rate limits
					</div>
				</button>
			{/each}
		</div>
	</section>

	<!-- Policy Configuration -->
	<PolicyConfigPanel />

	<!-- Policy Rules -->
	<PolicyRuleList />

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
					<button
						class="tag-remove"
						onclick={() => removeApprovalTool(tool)}>×</button
					>
				</span>
			{/each}
			<form
				class="tag-input-form"
				onsubmit={(e) => {
					e.preventDefault();
					addApprovalTool();
				}}
			>
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
		<span class="section-hint"
			>These tools are completely blocked from execution.</span
		>
		<div class="tag-list">
			{#each $policy.blocked_tools as tool}
				<span class="tag danger">
					{tool}
					<button
						class="tag-remove"
						onclick={() => removeBlockedTool(tool)}>×</button
					>
				</span>
			{/each}
			{#if $policy.blocked_tools.length === 0}
				<span class="empty-hint">No tools blocked</span>
			{/if}
			<form
				class="tag-input-form"
				onsubmit={(e) => {
					e.preventDefault();
					addBlockedTool();
				}}
			>
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

{#if confirmDialog}
	<ConfirmDialog
		action={confirmDialog.action}
		message={confirmDialog.message}
		onConfirm={confirmDialog.onConfirm}
		onCancel={() => (confirmDialog = null)}
	/>
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

	@media (max-width: 800px) {
		.template-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
