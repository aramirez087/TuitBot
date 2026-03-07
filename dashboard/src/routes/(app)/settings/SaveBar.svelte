<script lang="ts">
	import { Save, RotateCcw, Check, AlertTriangle, Info } from 'lucide-svelte';
	import { isDirty, saving, saveError } from '$lib/stores/settings';

	interface Props {
		showSaved: boolean;
		showDiscarded?: boolean;
		onSave: () => void;
		onDiscard: () => void;
	}

	let { showSaved, showDiscarded = false, onSave, onDiscard }: Props = $props();
</script>

{#if $isDirty || showSaved || showDiscarded || $saveError}
	<div class="save-bar" class:has-error={!!$saveError}>
		<div class="save-bar-content">
			{#if showDiscarded}
				<div class="save-status discarded">
					<Info size={16} />
					Unsaved changes were discarded
				</div>
			{:else if showSaved}
				<div class="save-status success">
					<Check size={16} />
					Settings saved
				</div>
			{:else if $saveError}
				<div class="save-status error">
					<AlertTriangle size={16} />
					{$saveError}
				</div>
			{:else}
				<span class="unsaved-text">You have unsaved changes</span>
			{/if}

			<div class="save-actions">
				{#if !showSaved && !showDiscarded}
					<button type="button" class="discard-btn" onclick={onDiscard} disabled={$saving}>
						<RotateCcw size={14} />
						Discard
					</button>
					<button
						type="button"
						class="save-btn"
						onclick={onSave}
						disabled={$saving}
					>
						{#if $saving}
							Saving...
						{:else}
							<Save size={14} />
							Save Changes
						{/if}
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.save-bar {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: 50;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border);
		padding: 12px 24px;
		animation: slideUp 0.2s ease-out;
	}

	@keyframes slideUp {
		from {
			transform: translateY(100%);
		}
		to {
			transform: translateY(0);
		}
	}

	.save-bar-content {
		max-width: 960px;
		margin: 0 auto;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.unsaved-text {
		font-size: 13px;
		color: var(--color-warning);
		font-weight: 500;
	}

	.save-status {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 500;
	}

	.save-status.success {
		color: var(--color-success);
	}

	.save-status.error {
		color: var(--color-danger);
	}

	.save-status.discarded {
		color: var(--color-warning);
	}

	.save-actions {
		display: flex;
		gap: 8px;
	}

	.discard-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		background: none;
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 13px;
		cursor: pointer;
		transition:
			background 0.15s,
			color 0.15s;
	}

	.discard-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.discard-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.save-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 16px;
		background: var(--color-accent);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition:
			background 0.15s,
			opacity 0.15s;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
