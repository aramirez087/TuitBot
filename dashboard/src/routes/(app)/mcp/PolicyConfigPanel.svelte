<script lang="ts">
	import { Shield, Lock, Unlock, FlaskConical } from "lucide-svelte";
	import { policy, updatePolicy } from "$lib/stores/mcp";
	import ConfirmDialog from "./ConfirmDialog.svelte";

	let pendingUpdate = $state(false);
	let confirmDialog = $state<{
		action: string;
		message: string;
		onConfirm: () => void;
	} | null>(null);

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
		if ($policy.enforce_for_mutations) {
			confirmDialog = {
				action: "Disable Policy Enforcement",
				message:
					"This will allow all MCP mutations without policy checks. Mutations will not be rate-limited, blocked, or routed to approval.",
				onConfirm: async () => {
					confirmDialog = null;
					await doUpdate({ enforce_for_mutations: false });
				},
			};
			return;
		}
		await doUpdate({ enforce_for_mutations: true });
	}

	async function toggleDryRun() {
		if (!$policy) return;
		await doUpdate({ dry_run_mutations: !$policy.dry_run_mutations });
	}
</script>

{#if $policy}
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
						Mutations are checked against rate limits, block lists,
						and approval requirements.
					{:else}
						<span class="warning-text"
							>All mutations execute without policy checks.</span
						>
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
						<span
							>{$policy.dry_run_mutations
								? "Active"
								: "Off"}</span
						>
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
						{$policy.rate_limit.used} / {$policy.rate_limit.max} per
						hour
					</span>
				</div>
				<div class="rate-bar">
					<div
						class="rate-fill"
						class:warning={$policy.rate_limit.used /
							$policy.rate_limit.max >
							0.75}
						class:danger={$policy.rate_limit.used /
							$policy.rate_limit.max >
							0.9}
						style="width: {Math.min(
							($policy.rate_limit.used /
								Math.max($policy.rate_limit.max, 1)) *
								100,
							100,
						)}%"
					></div>
				</div>
			</div>

			<div class="policy-item">
				<div class="policy-header">
					<span class="policy-label">Operating Mode</span>
					<span
						class="mode-badge"
						class:composer={$policy.mode === "composer"}
					>
						{$policy.mode}
					</span>
				</div>
				<span class="policy-hint">
					{#if $policy.mode === "composer"}
						Read-only + posting queue. All mutations route to
						approval.
					{:else}
						Full automation with policy-based gating.
					{/if}
				</span>
			</div>
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

</style>
