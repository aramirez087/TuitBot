<script lang="ts">
	import { Plus, Loader2 } from "lucide-svelte";
	import { createAccount } from "$lib/stores/accounts";

	interface Props {
		onCreated: () => void;
	}

	const { onCreated }: Props = $props();

	let newLabel = $state("");
	let creating = $state(false);
	let createError = $state("");

	async function handleCreate() {
		const label = newLabel.trim();
		if (!label || creating) return;
		creating = true;
		createError = "";
		try {
			await createAccount(label);
			newLabel = "";
			onCreated();
		} catch (e) {
			createError =
				e instanceof Error ? e.message : "Failed to create account";
		} finally {
			creating = false;
		}
	}
</script>

<div class="create-form">
	<div class="create-row">
		<input
			class="create-input"
			type="text"
			placeholder="Account label (e.g. My Brand)"
			bind:value={newLabel}
			disabled={creating}
			onkeydown={(e: KeyboardEvent) => {
				if (e.key === "Enter") handleCreate();
			}}
		/>
		<button
			class="create-btn"
			onclick={handleCreate}
			disabled={creating || !newLabel.trim()}
		>
			{#if creating}
				<Loader2 size={14} class="spinning" />
				Creating...
			{:else}
				<Plus size={14} />
				Add Account
			{/if}
		</button>
	</div>
	{#if createError}
		<p class="inline-error">{createError}</p>
	{/if}
</div>

<style>
	.create-form {
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.create-row {
		display: flex;
		gap: 8px;
	}

	.create-input {
		flex: 1;
		padding: 8px 12px;
		font-size: 13px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
	}

	.create-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.create-input::placeholder {
		color: var(--color-text-subtle);
	}

	.create-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		font-size: 13px;
		font-weight: 500;
		color: white;
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		cursor: pointer;
		transition: opacity 0.15s;
		white-space: nowrap;
	}

	.create-btn:hover:not(:disabled) {
		opacity: 0.9;
	}

	.create-btn:disabled {
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
