<script lang="ts">
	import { CheckCircle, XCircle, Pencil, X, Clock } from 'lucide-svelte';
	import { trackFunnel } from '$lib/analytics/funnel';
	import { type ApprovalItem } from '$lib/api';
	import ApprovalEditHistory from './ApprovalEditHistory.svelte';
	import ApprovalRejectDialog from './ApprovalRejectDialog.svelte';

	interface Props {
		item: ApprovalItem;
		editing: boolean;
		onApprove: (id: number) => void;
		onReject: (id: number, notes?: string) => void;
		onStartEdit: () => void;
		onAnnounce: (msg: string) => void;
	}

	const { item, editing, onApprove, onReject, onStartEdit, onAnnounce }: Props = $props();

	let showRejectDialog = $state(false);

	function handleApprove() {
		if (item.scheduled_for) {
			trackFunnel('schedule:approval-bridge', { has_scheduled_for: true });
			onAnnounce('Item approved and scheduled');
		} else {
			onAnnounce('Item approved');
		}
		onApprove(item.id);
	}
</script>

<ApprovalEditHistory approvalId={item.id} />

{#if showRejectDialog && item.status === 'pending'}
	<ApprovalRejectDialog
		itemId={item.id}
		onConfirm={(id, notes) => {
			showRejectDialog = false;
			onReject(id, notes || undefined);
		}}
		onCancel={() => (showRejectDialog = false)}
	/>
{:else if item.status === 'pending' && !editing}
	<div class="card-actions">
		<button class="action-btn approve" onclick={handleApprove}>
			<CheckCircle size={14} />
			Approve
			<kbd>a</kbd>
		</button>
		<button class="action-btn reject" onclick={() => (showRejectDialog = true)}>
			<XCircle size={14} />
			Reject
			<kbd>r</kbd>
		</button>
		<button class="action-btn edit" onclick={onStartEdit}>
			<Pencil size={14} />
			Edit
			<kbd>e</kbd>
		</button>
	</div>
{:else if item.status !== 'pending'}
	<div class="card-actions-readonly">
		{#if item.status === 'approved'}
			<span class="readonly-badge approved"><CheckCircle size={12} /> Approved</span>
		{:else if item.status === 'scheduled'}
			<span class="readonly-badge scheduled"><Clock size={12} /> Scheduled</span>
		{:else}
			<span class="readonly-badge rejected"><X size={12} /> Rejected</span>
		{/if}
	</div>
{/if}

<style>
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

	.action-btn kbd {
		font-size: 10px;
		padding: 0 4px;
		border-radius: 3px;
		background-color: var(--color-base);
		color: var(--color-text-subtle);
		font-family: var(--font-mono);
		border: 1px solid var(--color-border-subtle);
	}

	.action-btn.approve {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 30%, var(--color-border));
	}

	.action-btn.approve:hover {
		background-color: color-mix(in srgb, var(--color-success) 10%, transparent);
	}

	.action-btn.reject {
		color: var(--color-danger);
		border-color: color-mix(in srgb, var(--color-danger) 30%, var(--color-border));
	}

	.action-btn.reject:hover {
		background-color: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.action-btn.edit {
		color: var(--color-accent);
		border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
	}

	.action-btn.edit:hover {
		background-color: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.card-actions-readonly {
		display: flex;
	}

	.readonly-badge {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		font-weight: 600;
		padding: 3px 8px;
		border-radius: 4px;
	}

	.readonly-badge.approved {
		background-color: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.readonly-badge.rejected {
		background-color: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
	}

	.readonly-badge.scheduled {
		background-color: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}
</style>
