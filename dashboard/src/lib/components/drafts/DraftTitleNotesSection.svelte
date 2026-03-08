<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import type { ScheduledContentItem } from '$lib/api/types';

	let {
		draft,
		onupdatemeta
	}: {
		draft: ScheduledContentItem | null;
		onupdatemeta: (data: { title?: string; notes?: string }) => void;
	} = $props();

	let titleValue = $state('');
	let notesValue = $state('');
	let titleTimer: ReturnType<typeof setTimeout> | undefined;
	let notesTimer: ReturnType<typeof setTimeout> | undefined;
	let notesExpanded = $state(false);

	$effect(() => {
		if (draft) {
			titleValue = draft.title ?? '';
			notesValue = draft.notes ?? '';
			notesExpanded = !!(draft.notes && draft.notes.length > 0);
		}
	});

	function handleTitleInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		titleValue = value;
		clearTimeout(titleTimer);
		titleTimer = setTimeout(() => {
			onupdatemeta({ title: value || undefined });
		}, 800);
	}

	function handleNotesInput(e: Event) {
		const value = (e.target as HTMLTextAreaElement).value;
		notesValue = value;
		clearTimeout(notesTimer);
		notesTimer = setTimeout(() => {
			onupdatemeta({ notes: value || undefined });
		}, 800);
	}

	function handleTitleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			clearTimeout(titleTimer);
			onupdatemeta({ title: titleValue || undefined });
			(e.target as HTMLInputElement).blur();
		}
	}
</script>

<div class="field">
	<label class="field-label" for="draft-title">Title</label>
	<input
		id="draft-title"
		class="field-input"
		type="text"
		placeholder="Add a title..."
		value={titleValue}
		oninput={handleTitleInput}
		onkeydown={handleTitleKeydown}
	/>
</div>

<div class="field">
	<div class="field-label-row">
		<label class="field-label" for="draft-notes">Notes</label>
		{#if !notesExpanded && !notesValue}
			<button
				class="expand-btn"
				type="button"
				onclick={() => { notesExpanded = true; }}
			>
				<Plus size={11} /> Add
			</button>
		{/if}
	</div>
	{#if notesExpanded || notesValue}
		<textarea
			id="draft-notes"
			class="field-textarea"
			placeholder="Internal notes..."
			value={notesValue}
			oninput={handleNotesInput}
			rows="3"
		></textarea>
	{/if}
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.field-label-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.field-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.field-input {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid transparent;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		outline: none;
		transition: all 0.15s ease;
	}

	.field-input:hover {
		border-color: var(--color-border-subtle);
	}

	.field-input:focus {
		border-color: var(--color-accent);
		background: var(--color-base);
	}

	.field-input::placeholder {
		color: var(--color-text-subtle);
	}

	.field-textarea {
		width: 100%;
		padding: 6px 8px;
		border: 1px solid transparent;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		line-height: 1.5;
		outline: none;
		resize: vertical;
		min-height: 56px;
		max-height: 160px;
		font-family: inherit;
		transition: all 0.15s ease;
	}

	.field-textarea:hover {
		border-color: var(--color-border-subtle);
	}

	.field-textarea:focus {
		border-color: var(--color-accent);
		background: var(--color-base);
	}

	.field-textarea::placeholder {
		color: var(--color-text-subtle);
	}

	.expand-btn {
		display: flex;
		align-items: center;
		gap: 2px;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		padding: 2px 4px;
		border-radius: 3px;
		transition: all 0.1s ease;
	}

	.expand-btn:hover {
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}
</style>
