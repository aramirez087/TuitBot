<script lang="ts">
	interface Props {
		action: string;
		message: string;
		onConfirm: () => void;
		onCancel: () => void;
	}

	const { action, message, onConfirm, onCancel }: Props = $props();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="dialog-overlay"
	role="button"
	tabindex="-1"
	onclick={onCancel}
	onkeydown={(e) => {
		if (e.key === "Escape") onCancel();
	}}
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="dialog"
		role="dialog"
		aria-modal="true"
		tabindex="-1"
		onclick={(e) => e.stopPropagation()}
		onkeydown={(e) => e.stopPropagation()}
	>
		<h3>{action}</h3>
		<p>{message}</p>
		<div class="dialog-actions">
			<button class="btn-cancel" onclick={onCancel}>Cancel</button>
			<button class="btn-confirm" onclick={onConfirm}>Confirm</button>
		</div>
	</div>
</div>

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.dialog {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 24px;
		max-width: 420px;
		width: 90%;
		text-align: center;
	}

	.dialog h3 {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 8px 0;
	}

	.dialog p {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
		margin: 0 0 20px 0;
	}

	.dialog-actions {
		display: flex;
		gap: 8px;
		justify-content: center;
	}

	.btn-cancel,
	.btn-confirm {
		padding: 8px 20px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.btn-cancel {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text-muted);
	}

	.btn-cancel:hover {
		border-color: var(--color-text-subtle);
		color: var(--color-text);
	}

	.btn-confirm {
		background: var(--color-danger);
		border: 1px solid var(--color-danger);
		color: white;
	}

	.btn-confirm:hover {
		opacity: 0.9;
	}
</style>
