<script lang="ts">
	import { X, UserPlus } from 'lucide-svelte';

	interface Props {
		open: boolean;
		submitting: boolean;
		error: string | null;
		onclose: () => void;
		onsubmit: (username: string) => void;
	}

	let { open, submitting, error, onclose, onsubmit }: Props = $props();

	let username = $state('');

	const cleanUsername = $derived(username.trim().replace(/^@/, ''));
	const canSubmit = $derived(cleanUsername.length > 0 && !submitting);

	$effect(() => {
		if (open) {
			username = '';
		}
	});

	function handleSubmit() {
		if (!canSubmit) return;
		onsubmit(cleanUsername);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && canSubmit) {
			e.preventDefault();
			handleSubmit();
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}

	function handleWindowKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onclose();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={handleBackdropClick}>
		<div class="modal">
			<div class="modal-header">
				<h2>Add Target Account</h2>
				<button class="close-btn" onclick={onclose}>
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<label class="field-label" for="target-username">Username</label>
				<div class="input-row">
					<span class="at-prefix">@</span>
					<input
						id="target-username"
						type="text"
						class="username-input"
						placeholder="username"
						bind:value={username}
						onkeydown={handleKeydown}
						disabled={submitting}
					/>
				</div>
				<p class="field-hint">
					Enter the X username of the account you want to monitor and engage with.
				</p>

				{#if error}
					<div class="error-msg">{error}</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="cancel-btn" onclick={onclose}>Cancel</button>
				<button class="submit-btn" onclick={handleSubmit} disabled={!canSubmit}>
					<UserPlus size={14} />
					{submitting ? 'Adding...' : 'Add Target'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		width: 420px;
		max-width: 90vw;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.modal-body {
		padding: 20px;
	}

	.field-label {
		display: block;
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		margin-bottom: 6px;
	}

	.input-row {
		display: flex;
		align-items: center;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-base);
		overflow: hidden;
		transition: border-color 0.15s ease;
	}

	.input-row:focus-within {
		border-color: var(--color-accent);
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 20%, transparent);
	}

	.at-prefix {
		padding: 0 0 0 12px;
		font-size: 14px;
		color: var(--color-text-muted);
		user-select: none;
	}

	.username-input {
		flex: 1;
		padding: 10px 12px 10px 4px;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 14px;
		outline: none;
	}

	.username-input::placeholder {
		color: var(--color-text-subtle);
	}

	.username-input:disabled {
		opacity: 0.5;
	}

	.field-hint {
		margin: 8px 0 0;
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.error-msg {
		margin-top: 12px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 12px;
	}

	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 20px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.cancel-btn {
		padding: 8px 16px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 13px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.cancel-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.submit-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.submit-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.submit-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
