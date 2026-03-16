<script lang="ts">
	import { untrack } from "svelte";
	import { Check, X, Loader2 } from "lucide-svelte";
	import { renameAccount } from "$lib/stores/accounts";

	interface Props {
		accountId: string;
		initialLabel: string;
		onConfirm: () => void;
		onCancel: () => void;
	}

	const { accountId, initialLabel, onConfirm, onCancel }: Props = $props();

	// untrack: intentionally capture only the initial prop value as seed for the edit form
	let editLabel = $state(untrack(() => initialLabel));
	let renaming = $state(false);
	let renameError = $state("");

	function focus(node: HTMLElement) {
		node.focus();
	}

	async function handleRename() {
		if (renaming) return;
		const label = editLabel.trim();
		if (!label) return;
		renaming = true;
		renameError = "";
		try {
			await renameAccount(accountId, label);
			onConfirm();
		} catch (e) {
			renameError =
				e instanceof Error ? e.message : "Failed to rename account";
		} finally {
			renaming = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Enter") handleRename();
		if (e.key === "Escape") onCancel();
	}
</script>

<div class="rename-row">
	<input
		class="rename-input"
		type="text"
		bind:value={editLabel}
		onkeydown={handleKeydown}
		disabled={renaming}
		use:focus
	/>
	<button
		class="icon-btn confirm"
		onclick={handleRename}
		disabled={renaming || !editLabel.trim()}
		title="Save"
	>
		{#if renaming}
			<Loader2 size={14} class="spinning" />
		{:else}
			<Check size={14} />
		{/if}
	</button>
	<button
		class="icon-btn cancel"
		onclick={onCancel}
		disabled={renaming}
		title="Cancel"
	>
		<X size={14} />
	</button>
</div>
{#if renameError}
	<p class="inline-error">{renameError}</p>
{/if}

<style>
	.rename-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.rename-input {
		flex: 1;
		padding: 4px 8px;
		font-size: 13px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 5px;
		color: var(--color-text);
		min-width: 0;
	}

	.rename-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition:
			background 0.15s,
			color 0.15s;
	}

	.icon-btn.confirm:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
	}

	.icon-btn.cancel:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.icon-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.inline-error {
		font-size: 12px;
		color: var(--color-danger);
		margin: 6px 0 0;
	}

	:global(.spinning) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
