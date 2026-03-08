<script lang="ts">
	import { X, Plus } from 'lucide-svelte';
	import type { ContentTag } from '$lib/api/types';

	let {
		tags,
		allTags,
		onassigntag,
		onunassigntag,
		oncreatetag
	}: {
		tags: ContentTag[];
		allTags: ContentTag[];
		onassigntag: (tagId: number) => void;
		onunassigntag: (tagId: number) => void;
		oncreatetag: (name: string) => void;
	} = $props();

	let tagPickerOpen = $state(false);
	let newTagInput = $state('');

	const unassignedTags = $derived(
		allTags.filter((t) => !tags.some((at) => at.id === t.id))
	);

	function handleCreateTag() {
		const name = newTagInput.trim();
		if (!name) return;
		oncreatetag(name);
		newTagInput = '';
	}

	function handleNewTagKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			handleCreateTag();
		}
		if (e.key === 'Escape') {
			tagPickerOpen = false;
		}
	}
</script>

<div class="field">
	<div class="field-label-row">
		<span class="field-label">Tags</span>
	</div>
	<div class="tags-container">
		{#each tags as tag (tag.id)}
			<span class="tag-pill">
				{#if tag.color}
					<span class="tag-color" style="background: {tag.color}"></span>
				{/if}
				{tag.name}
				<button
					class="tag-remove"
					type="button"
					title="Remove tag"
					onclick={() => onunassigntag(tag.id)}
				>
					<X size={10} />
				</button>
			</span>
		{/each}
		<button
			class="tag-add-btn"
			type="button"
			onclick={() => { tagPickerOpen = !tagPickerOpen; }}
		>
			<Plus size={11} />
			{tags.length === 0 ? 'Add tag' : ''}
		</button>
	</div>

	{#if tagPickerOpen}
		<div class="tag-picker">
			{#if unassignedTags.length > 0}
				{#each unassignedTags as tag (tag.id)}
					<button
						class="tag-option"
						type="button"
						onclick={() => { onassigntag(tag.id); }}
					>
						{#if tag.color}
							<span class="tag-dot" style="background: {tag.color}"></span>
						{/if}
						{tag.name}
					</button>
				{/each}
				<div class="tag-divider"></div>
			{/if}
			<div class="tag-create-row">
				<input
					class="tag-create-input"
					type="text"
					placeholder="New tag..."
					bind:value={newTagInput}
					onkeydown={handleNewTagKeydown}
				/>
				{#if newTagInput.trim()}
					<button class="tag-create-btn" type="button" onclick={handleCreateTag}>
						<Plus size={11} />
					</button>
				{/if}
			</div>
		</div>
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

	.tags-container {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		align-items: center;
	}

	.tag-pill {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 10px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 500;
	}

	.tag-color {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tag-remove {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		height: 14px;
		border: none;
		border-radius: 50%;
		background: transparent;
		color: var(--color-accent);
		cursor: pointer;
		padding: 0;
		opacity: 0.6;
		transition: opacity 0.1s ease;
	}

	.tag-remove:hover {
		opacity: 1;
	}

	.tag-add-btn {
		display: flex;
		align-items: center;
		gap: 2px;
		padding: 2px 8px;
		border: 1px dashed var(--color-border-subtle);
		border-radius: 10px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.tag-add-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.tag-picker {
		margin-top: 4px;
		padding: 6px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-base);
	}

	.tag-option {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 5px 8px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		transition: background 0.1s ease;
	}

	.tag-option:hover {
		background: var(--color-surface-hover);
	}

	.tag-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.tag-divider {
		height: 1px;
		background: var(--color-border-subtle);
		margin: 4px 0;
	}

	.tag-create-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.tag-create-input {
		flex: 1;
		padding: 4px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		background: transparent;
		color: var(--color-text);
		font-size: 12px;
		outline: none;
	}

	.tag-create-input:focus {
		border-color: var(--color-accent);
	}

	.tag-create-input::placeholder {
		color: var(--color-text-subtle);
	}

	.tag-create-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: var(--color-accent);
		color: #fff;
		cursor: pointer;
		padding: 0;
	}
</style>
