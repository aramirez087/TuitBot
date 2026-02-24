<script lang="ts">
	interface Props {
		value: number;
		label: string;
		min: number;
		max: number;
		step?: number;
		unit?: string;
		helpText?: string;
		error?: string;
		defaultValue?: number;
		onchange: (value: number) => void;
	}

	let {
		value,
		label,
		min,
		max,
		step = 1,
		unit = '',
		helpText = '',
		error = '',
		defaultValue,
		onchange
	}: Props = $props();

	function handleRangeInput(e: Event) {
		const target = e.target as HTMLInputElement;
		onchange(parseFloat(target.value));
	}

	function handleNumberInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const raw = parseFloat(target.value);
		if (!isNaN(raw)) {
			onchange(raw);
		}
	}

	function handleNumberBlur(e: Event) {
		const target = e.target as HTMLInputElement;
		const raw = parseFloat(target.value);
		if (isNaN(raw)) {
			onchange(min);
		} else {
			onchange(Math.min(max, Math.max(min, raw)));
		}
	}

	const displayValue = $derived(
		step < 1 ? value.toFixed(2) : value.toString()
	);

	const showDefault = $derived(
		defaultValue !== undefined && defaultValue !== value
	);

	const fillPercent = $derived(
		((value - min) / (max - min)) * 100
	);

	const inputId = $derived(label.toLowerCase().replace(/[^a-z0-9]+/g, '-'));
</script>

<div class="field">
	<div class="field-header">
		<label class="field-label" for="slider-{inputId}">{label}</label>
		<span class="field-value">
			{displayValue}{#if unit}<span class="unit">{unit}</span>{/if}
		</span>
	</div>
	<div class="slider-row">
		<input
			id="slider-{inputId}"
			type="range"
			{min}
			{max}
			{step}
			{value}
			class="slider"
			class:has-error={!!error}
			style="--fill: {fillPercent}%"
			oninput={handleRangeInput}
		/>
		<input
			type="number"
			{min}
			{max}
			{step}
			{value}
			class="number-input"
			class:has-error={!!error}
			oninput={handleNumberInput}
			onblur={handleNumberBlur}
		/>
	</div>
	{#if error}
		<span class="field-error">{error}</span>
	{:else if helpText}
		<span class="field-hint">{helpText}</span>
	{/if}
	{#if showDefault}
		<span class="field-default">Default: {defaultValue}{unit}</span>
	{/if}
</div>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.field-value {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-accent);
		font-family: var(--font-mono);
	}

	.unit {
		font-weight: 400;
		color: var(--color-text-muted);
		margin-left: 2px;
	}

	.slider-row {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.slider {
		flex: 1;
		height: 6px;
		-webkit-appearance: none;
		appearance: none;
		background: linear-gradient(
			to right,
			var(--color-accent) 0%,
			var(--color-accent) var(--fill),
			var(--color-border) var(--fill),
			var(--color-border) 100%
		);
		border-radius: 3px;
		outline: none;
		cursor: pointer;
	}

	.slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--color-accent);
		border: 2px solid var(--color-surface);
		cursor: pointer;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	.slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--color-accent);
		border: 2px solid var(--color-surface);
		cursor: pointer;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	.slider.has-error {
		background: linear-gradient(
			to right,
			var(--color-danger) 0%,
			var(--color-danger) var(--fill),
			var(--color-border) var(--fill),
			var(--color-border) 100%
		);
	}

	.number-input {
		width: 72px;
		padding: 6px 8px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-mono);
		text-align: right;
		outline: none;
		transition: border-color 0.15s;
	}

	.number-input:focus {
		border-color: var(--color-accent);
	}

	.number-input.has-error {
		border-color: var(--color-danger);
	}

	/* Hide spinners for number input */
	.number-input::-webkit-inner-spin-button,
	.number-input::-webkit-outer-spin-button {
		-webkit-appearance: none;
		appearance: none;
		margin: 0;
	}
	.number-input {
		-moz-appearance: textfield;
		appearance: textfield;
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
