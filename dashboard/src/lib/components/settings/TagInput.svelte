<script lang="ts">
	import { X } from 'lucide-svelte';

	interface Props {
		value: string[];
		label: string;
		placeholder?: string;
		helpText?: string;
		error?: string;
		defaultValue?: string[];
		onchange: (tags: string[]) => void;
	}

	let {
		value,
		label,
		placeholder = 'Type and press Enter',
		helpText = '',
		error = '',
		defaultValue,
		onchange
	}: Props = $props();

	let inputValue = $state('');
	let inputEl: HTMLInputElement | undefined = $state();

	function addTag(raw: string) {
		const tag = raw.trim();
		if (tag && !value.includes(tag)) {
			onchange([...value, tag]);
		}
		inputValue = '';
	}

	function removeTag(index: number) {
		onchange(value.filter((_, i) => i !== index));
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ',') {
			e.preventDefault();
			if (inputValue.trim()) {
				addTag(inputValue);
			}
		} else if (e.key === 'Backspace' && !inputValue && value.length > 0) {
			removeTag(value.length - 1);
		}
	}

	function handlePaste(e: ClipboardEvent) {
		e.preventDefault();
		const text = e.clipboardData?.getData('text') ?? '';
		const tags = text
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t && !value.includes(t));
		if (tags.length > 0) {
			onchange([...value, ...tags]);
		}
	}

	function handleBlur() {
		if (inputValue.trim()) {
			addTag(inputValue);
		}
	}

	const showDefault = $derived(
		defaultValue && JSON.stringify(defaultValue) !== JSON.stringify(value)
	);

	const inputId = $derived(label.toLowerCase().replace(/[^a-z0-9]+/g, '-'));
</script>

<div class="field">
	<label class="field-label" for="tag-{inputId}">{label}</label>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="tag-container" class:has-error={!!error} onclick={() => inputEl?.focus()}>
		{#each value as tag, i}
			<span class="tag">
				<span class="tag-text">{tag}</span>
				<button
					type="button"
					class="tag-remove"
					onclick={(e) => {
						e.stopPropagation();
						removeTag(i);
					}}
					aria-label="Remove {tag}"
				>
					<X size={12} />
				</button>
			</span>
		{/each}
		<input
			id="tag-{inputId}"
			bind:this={inputEl}
			bind:value={inputValue}
			{placeholder}
			class="tag-input"
			onkeydown={handleKeydown}
			onpaste={handlePaste}
			onblur={handleBlur}
		/>
	</div>
	{#if error}
		<span class="field-error">{error}</span>
	{:else if helpText}
		<span class="field-hint">{helpText}</span>
	{/if}
	{#if showDefault}
		<span class="field-default">Default: {defaultValue?.join(', ')}</span>
	{/if}
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.tag-container {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 8px 10px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		cursor: text;
		min-height: 38px;
		transition: border-color 0.15s;
	}

	.tag-container:focus-within {
		border-color: var(--color-accent);
	}

	.tag-container.has-error {
		border-color: var(--color-danger);
	}

	.tag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
		border-radius: 4px;
		font-size: 12px;
		line-height: 1.5;
	}

	.tag-text {
		max-width: 200px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.tag-remove {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		border: none;
		background: none;
		color: var(--color-accent);
		cursor: pointer;
		opacity: 0.6;
		transition: opacity 0.15s;
		line-height: 1;
	}

	.tag-remove:hover {
		opacity: 1;
	}

	.tag-input {
		flex: 1;
		min-width: 100px;
		border: none;
		background: none;
		color: var(--color-text);
		font-size: 13px;
		outline: none;
		padding: 2px 0;
	}

	.tag-input::placeholder {
		color: var(--color-text-subtle);
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.field-default {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-style: italic;
	}
</style>
