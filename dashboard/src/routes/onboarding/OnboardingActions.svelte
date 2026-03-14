<script lang="ts">
	import { ArrowLeft, ArrowRight, Loader2, SkipForward, Zap } from 'lucide-svelte';

	interface Props {
		currentStep: number;
		isLastStep: boolean;
		isClaimStep: boolean;
		canSkipToFinish: boolean;
		advanceAllowed: boolean;
		submitting: boolean;
		onBack: () => void;
		onNext: () => void;
		onSkip: () => void;
		onSubmit: () => void;
	}

	const {
		currentStep,
		isLastStep,
		isClaimStep,
		canSkipToFinish,
		advanceAllowed,
		submitting,
		onBack,
		onNext,
		onSkip,
		onSubmit,
	}: Props = $props();
</script>

<div class="actions">
	{#if currentStep > 0}
		<button class="btn btn-secondary" onclick={onBack} disabled={submitting}>
			<ArrowLeft size={16} />
			Back
		</button>
	{:else}
		<div></div>
	{/if}

	{#if !isLastStep}
		<div class="action-group">
			{#if canSkipToFinish && advanceAllowed}
				<button class="btn btn-ghost" onclick={onSkip}>
					Skip optional steps
					<SkipForward size={14} />
				</button>
			{/if}
			<button class="btn btn-primary" onclick={onNext} disabled={!advanceAllowed}>
				{currentStep === 0 ? 'Get Started' : 'Next'}
				<ArrowRight size={16} />
			</button>
		</div>
	{:else}
		<button
			class="btn btn-primary"
			onclick={onSubmit}
			disabled={submitting || (isClaimStep && !advanceAllowed)}
		>
			{#if submitting}
				<span class="spinner"><Loader2 size={16} /></span>
				Creating...
			{:else}
				Start Tuitbot
				<Zap size={16} />
			{/if}
		</button>
	{/if}
</div>

<style>
	.actions {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.action-group {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 10px 20px;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--color-accent);
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.btn-primary:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.btn-secondary {
		background: var(--color-surface);
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.btn-secondary:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.btn-ghost {
		background: transparent;
		color: var(--color-text-muted);
		padding: 10px 14px;
		font-size: 13px;
	}

	.btn-ghost:hover:not(:disabled) {
		color: var(--color-text);
		background: var(--color-surface);
	}

	.spinner {
		display: inline-flex;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
