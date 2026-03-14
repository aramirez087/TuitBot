<script lang="ts">
	import { fly } from 'svelte/transition';
	import { MessageSquare, FileText, BookOpen } from 'lucide-svelte';
	import { type ApprovalItem } from '$lib/api';
	import { formatInAccountTz } from '$lib/utils/timezone';
	import ApprovalCardHeader from './ApprovalCardHeader.svelte';
	import ApprovalCardBody from './ApprovalCardBody.svelte';
	import ApprovalCardActions from './ApprovalCardActions.svelte';

	interface Props {
		item: ApprovalItem;
		focused: boolean;
		editing: boolean;
		timezone?: string;
		onApprove: (id: number) => void;
		onReject: (id: number, notes?: string) => void;
		onStartEdit: (id: number) => void;
		onSaveEdit: (id: number, content: string) => void;
		onCancelEdit: () => void;
	}

	let {
		item,
		focused,
		editing,
		timezone = 'UTC',
		onApprove,
		onReject,
		onStartEdit,
		onSaveEdit,
		onCancelEdit
	}: Props = $props();

	let editContent = $state('');
	let textareaEl = $state<HTMLTextAreaElement | undefined>(undefined);
	let announcement = $state('');

	const charCount = $derived(editing ? editContent.length : item.generated_content.length);
	const isOverLimit = $derived(charCount > 280);

	/* eslint-disable @typescript-eslint/no-explicit-any */
	const iconMap: Record<string, any> = {
		reply: MessageSquare,
		tweet: FileText,
		thread_tweet: BookOpen
	};
	/* eslint-enable @typescript-eslint/no-explicit-any */

	const Icon = $derived(iconMap[item.action_type] ?? FileText);
	const typeLabel = $derived(item.action_type.replace(/_/g, ' '));
	const mediaPaths = $derived(item.media_paths ?? []);
	const risks = $derived(Array.isArray(item.detected_risks) ? item.detected_risks : []);

	const statusClass = $derived(
		item.status === 'pending'
			? 'status-pending'
			: item.status === 'approved'
				? 'status-approved'
				: item.status === 'scheduled'
					? 'status-scheduled'
					: 'status-rejected'
	);

	const scheduledLabel = $derived(
		item.scheduled_for
			? formatInAccountTz(item.scheduled_for, timezone, {
					month: 'short',
					day: 'numeric',
					hour: '2-digit',
					minute: '2-digit',
					timeZoneName: 'short'
				})
			: null
	);

	function handleStartEdit() {
		editContent = item.generated_content;
		onStartEdit(item.id);
	}

	function handleSave() {
		if (editContent.trim() && !isOverLimit) {
			onSaveEdit(item.id, editContent.trim());
		}
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			onCancelEdit();
		} else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			handleSave();
		}
	}

	$effect(() => {
		if (editing && textareaEl) {
			textareaEl.focus();
			textareaEl.setSelectionRange(textareaEl.value.length, textareaEl.value.length);
		}
	});
</script>

<div
	class="card"
	class:focused
	class:editing
	transition:fly={{ x: 300, duration: 250 }}
	aria-label="{typeLabel} approval item"
>
	<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{announcement}</div>
	<div class="card-icon {statusClass}">
		<Icon size={16} />
	</div>

	<div class="card-body">
		<ApprovalCardHeader
			{item}
			{statusClass}
			{typeLabel}
			{scheduledLabel}
		/>
		<ApprovalCardBody
			{item}
			{editing}
			bind:editContent
			bind:textareaEl
			{isOverLimit}
			{charCount}
			{mediaPaths}
			{risks}
			onSave={handleSave}
			{onCancelEdit}
			onEditKeydown={handleEditKeydown}
		/>
		<ApprovalCardActions
			{item}
			{editing}
			{onApprove}
			{onReject}
			onStartEdit={handleStartEdit}
			onAnnounce={(msg) => (announcement = msg)}
		/>
	</div>
</div>

<style>
	.card {
		display: flex;
		gap: 12px;
		padding: 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		transition: background-color 0.1s ease;
		position: relative;
	}

	.card:last-child {
		border-bottom: none;
	}

	.card:hover {
		background-color: var(--color-surface-hover);
	}

	.card.focused {
		background-color: var(--color-surface-active);
		border-left: 3px solid var(--color-accent);
		padding-left: 13px;
	}

	.card.editing {
		background-color: var(--color-surface-active);
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
	}

	.card-icon.status-pending {
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		color: var(--color-warning);
	}

	.card-icon.status-approved {
		background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}

	.card-icon.status-rejected {
		background-color: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.card-icon.status-scheduled {
		background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
	}

	.card-body {
		flex: 1;
		min-width: 0;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}
</style>
