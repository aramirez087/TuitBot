<script lang="ts">
	import ApprovalCard from './ApprovalCard.svelte';
	import type { ApprovalItem } from '$lib/api';

	interface Props {
		item: ApprovalItem;
		focused?: boolean;
		editing?: boolean;
		timezone: string;
		selected?: boolean;
		onApprove: (id: number) => void;
		onReject: (id: number, notes?: string) => void;
		onStartEdit: (id: number) => void;
		onSaveEdit: (id: number, content: string) => void;
		onCancelEdit: () => void;
		onSelectionChange?: (itemId: number, selected: boolean) => void;
	}

	const {
		item,
		focused = false,
		editing = false,
		timezone,
		selected = false,
		onApprove,
		onReject,
		onStartEdit,
		onSaveEdit,
		onCancelEdit,
		onSelectionChange
	}: Props = $props();

	function handleCheckboxChange(e: Event) {
		const checked = (e.target as HTMLInputElement).checked;
		onSelectionChange?.(item.id, checked);
	}
</script>

<div class="selectable-card" class:selected>
	<div class="checkbox-wrapper">
		<input
			type="checkbox"
			class="checkbox"
			checked={selected}
			onchange={handleCheckboxChange}
			aria-label="Select item {item.id}"
		/>
	</div>

	<div class="card-content">
		<ApprovalCard
			{item}
			{focused}
			{editing}
			{timezone}
			{onApprove}
			{onReject}
			{onStartEdit}
			{onSaveEdit}
			{onCancelEdit}
		/>
	</div>
</div>

<style>
	.selectable-card {
		display: flex;
		gap: 12px;
		padding: 12px;
		border-left: 3px solid transparent;
		background: var(--color-base);
		transition: all 0.15s ease;
	}

	.selectable-card.selected {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border-left-color: var(--color-accent);
	}

	.checkbox-wrapper {
		display: flex;
		align-items: flex-start;
		padding-top: 12px;
		flex-shrink: 0;
	}

	.checkbox {
		width: 18px;
		height: 18px;
		cursor: pointer;
		accent-color: var(--color-accent);
	}

	.card-content {
		flex: 1;
		min-width: 0;
	}

	@media (max-width: 640px) {
		.selectable-card {
			gap: 8px;
			padding: 8px;
		}

		.checkbox {
			width: 16px;
			height: 16px;
		}
	}
</style>
