<script lang="ts">
	interface Props {
		steps: string[];
		currentStep: number;
		skippedSteps: Set<string>;
		optionalSteps?: Set<string>;
	}

	const { steps, currentStep, skippedSteps, optionalSteps = new Set() }: Props = $props();
</script>

<div class="progress">
	{#each steps as step, displayIdx}
		{@const isSkipped = skippedSteps.has(step)}
		<div
			class="progress-step"
			class:active={displayIdx === currentStep}
			class:completed={displayIdx < currentStep && !isSkipped}
			class:skipped={isSkipped && displayIdx < currentStep}
		>
			<div class="progress-dot">
				{#if isSkipped && displayIdx < currentStep}
					<span class="skip-mark">&mdash;</span>
				{:else if displayIdx < currentStep}
					<span class="check-mark">&#10003;</span>
				{:else}
					{displayIdx + 1}
				{/if}
			</div>
			<span class="progress-label">
				{step}
				{#if optionalSteps.has(step)}
					<span class="optional-badge">[Optional]</span>
				{/if}
			</span>
		</div>
		{#if displayIdx < steps.length - 1}
			<div class="progress-line" class:filled={displayIdx < currentStep}></div>
		{/if}
	{/each}
</div>

<style>
	.progress {
		display: flex;
		align-items: center;
		gap: 0;
	}

	.progress-step {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.progress-dot {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 600;
		background: var(--color-surface);
		border: 2px solid var(--color-border);
		color: var(--color-text-muted);
		transition: all 0.2s;
	}

	.progress-step.active .progress-dot {
		background: var(--color-accent);
		border-color: var(--color-accent);
		color: white;
	}

	.progress-step.completed .progress-dot {
		background: var(--color-success);
		border-color: var(--color-success);
		color: white;
	}

	.progress-step.skipped .progress-dot {
		background: var(--color-surface);
		border-color: var(--color-border);
		color: var(--color-text-subtle);
		border-style: dashed;
	}

	.check-mark {
		font-size: 14px;
	}

	.skip-mark {
		font-size: 14px;
		font-weight: 700;
	}

	.progress-label {
		font-size: 11px;
		color: var(--color-text-subtle);
		white-space: nowrap;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}

	.optional-badge {
		font-size: 9px;
		font-weight: 500;
		color: var(--color-text-subtle);
		opacity: 0.75;
	}

	.progress-step.active .progress-label {
		color: var(--color-text);
		font-weight: 500;
	}

	.progress-step.skipped .progress-label {
		color: var(--color-text-subtle);
		font-style: italic;
	}

	.progress-line {
		flex: 1;
		height: 2px;
		background: var(--color-border);
		margin: 0 4px;
		margin-bottom: 20px;
		transition: background 0.2s;
	}

	.progress-line.filled {
		background: var(--color-success);
	}
</style>
