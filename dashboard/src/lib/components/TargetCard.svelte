<script lang="ts">
	import { Target, Clock, Eye, Trash2 } from 'lucide-svelte';
	import type { TargetAccount } from '$lib/api';

	interface Props {
		target: TargetAccount;
		maxDailyReplies: number;
		onview: (username: string) => void;
		onremove: (username: string) => void;
	}

	let { target, maxDailyReplies, onview, onremove }: Props = $props();

	let confirmingRemove = $state(false);

	const dailyPercent = $derived(
		maxDailyReplies > 0
			? Math.min((target.interactions_today / maxDailyReplies) * 100, 100)
			: 0
	);
	const dailyBarClass = $derived(
		dailyPercent >= 80 ? 'danger' : dailyPercent >= 60 ? 'warning' : 'success'
	);

	function relativeTime(iso: string | null): string {
		if (!iso) return 'Never';
		const diff = Date.now() - new Date(iso).getTime();
		const mins = Math.floor(diff / 60_000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ago`;
	}

	function handleRemoveClick() {
		if (confirmingRemove) {
			onremove(target.username);
			confirmingRemove = false;
		} else {
			confirmingRemove = true;
		}
	}

	function cancelRemove() {
		confirmingRemove = false;
	}
</script>

<div class="card">
	<div class="card-icon">
		<Target size={16} />
	</div>

	<div class="card-body">
		<div class="card-header">
			<span class="card-username">@{target.username}</span>
			<span class="card-interactions">
				{target.total_replies_sent} interaction{target.total_replies_sent !== 1 ? 's' : ''}
			</span>
		</div>

		<div class="card-meta">
			<span class="meta-item">
				<Clock size={12} />
				{#if target.last_reply_at}
					Last interaction: {relativeTime(target.last_reply_at)}
				{:else}
					Added {relativeTime(target.first_engagement_at)}
				{/if}
			</span>
		</div>

		{#if maxDailyReplies > 0}
			<div class="daily-limit">
				<div class="daily-header">
					<span class="daily-label">Today</span>
					<span class="daily-count">
						{target.interactions_today}/{maxDailyReplies}
					</span>
				</div>
				<div class="daily-track">
					<div
						class="daily-fill {dailyBarClass}"
						style="width: {dailyPercent}%"
					></div>
				</div>
			</div>
		{/if}

		<div class="card-actions">
			<button class="action-btn view" onclick={() => onview(target.username)}>
				<Eye size={14} />
				View
			</button>
			{#if confirmingRemove}
				<button class="action-btn remove-confirm" onclick={handleRemoveClick}>
					Confirm
				</button>
				<button class="action-btn cancel" onclick={cancelRemove}>
					Cancel
				</button>
			{:else}
				<button class="action-btn remove" onclick={handleRemoveClick}>
					<Trash2 size={14} />
					Remove
				</button>
			{/if}
		</div>
	</div>
</div>

<style>
	.card {
		display: flex;
		gap: 12px;
		padding: 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		transition: background-color 0.1s ease;
	}

	.card:last-child {
		border-bottom: none;
	}

	.card:hover {
		background-color: var(--color-surface-hover);
	}

	.card-icon {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 2px;
		background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
	}

	.card-body {
		flex: 1;
		min-width: 0;
	}

	.card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 6px;
	}

	.card-username {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.card-interactions {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}

	.card-meta {
		margin-bottom: 10px;
	}

	.meta-item {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.daily-limit {
		margin-bottom: 10px;
	}

	.daily-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 4px;
	}

	.daily-label {
		font-size: 11px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.daily-count {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-muted);
		font-variant-numeric: tabular-nums;
	}

	.daily-track {
		height: 4px;
		border-radius: 2px;
		background-color: var(--color-surface-active);
		overflow: hidden;
	}

	.daily-fill {
		height: 100%;
		border-radius: 2px;
		transition: width 0.3s ease;
	}

	.daily-fill.success {
		background-color: var(--color-success);
	}

	.daily-fill.warning {
		background-color: var(--color-warning);
	}

	.daily-fill.danger {
		background-color: var(--color-danger);
	}

	.card-actions {
		display: flex;
		gap: 8px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background-color: var(--color-surface);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn.view {
		color: var(--color-accent);
		border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
	}

	.action-btn.view:hover {
		background-color: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.action-btn.remove {
		color: var(--color-text-muted);
		border-color: var(--color-border);
	}

	.action-btn.remove:hover {
		color: var(--color-danger);
		border-color: color-mix(in srgb, var(--color-danger) 30%, var(--color-border));
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.action-btn.remove-confirm {
		color: var(--color-danger);
		border-color: var(--color-danger);
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.action-btn.remove-confirm:hover {
		background-color: color-mix(in srgb, var(--color-danger) 20%, transparent);
	}

	.action-btn.cancel {
		color: var(--color-text-muted);
	}

	.action-btn.cancel:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}
</style>
