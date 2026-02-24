<script lang="ts">
	import { Wifi, Check, X, Loader2 } from 'lucide-svelte';
	import type { SettingsTestResult } from '$lib/api';

	interface Props {
		label: string;
		ontest: () => Promise<SettingsTestResult>;
	}

	let { label, ontest }: Props = $props();

	let status: 'idle' | 'testing' | 'success' | 'failure' = $state('idle');
	let result: SettingsTestResult | null = $state(null);
	let timeout: ReturnType<typeof setTimeout> | null = $state(null);

	async function runTest() {
		if (status === 'testing') return;
		status = 'testing';
		result = null;

		try {
			result = await ontest();
			status = result.success ? 'success' : 'failure';

			if (result.success) {
				if (timeout) clearTimeout(timeout);
				timeout = setTimeout(() => {
					status = 'idle';
					result = null;
				}, 5000);
			}
		} catch (e) {
			result = {
				success: false,
				error: e instanceof Error ? e.message : 'Connection test failed'
			};
			status = 'failure';
		}
	}
</script>

<div class="connection-test">
	{#if status === 'idle'}
		<button type="button" class="test-btn" onclick={runTest}>
			<Wifi size={14} />
			{label}
		</button>
	{:else if status === 'testing'}
		<button type="button" class="test-btn testing" disabled>
			<span class="spinner"><Loader2 size={14} /></span>
			Testing...
		</button>
	{:else if status === 'success'}
		<div class="test-result success">
			<Check size={14} />
			<span>
				Connected
				{#if result?.latency_ms !== undefined}
					— {result.latency_ms}ms
				{/if}
			</span>
		</div>
	{:else if status === 'failure'}
		<div class="test-result-wrap">
			<div class="test-result failure">
				<X size={14} />
				<span>{result?.error ?? 'Connection failed'}</span>
			</div>
			<button type="button" class="retry-btn" onclick={runTest}>
				Retry
			</button>
		</div>
	{/if}
</div>

<style>
	.connection-test {
		margin-top: 4px;
	}

	.test-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.test-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-color: var(--color-accent);
	}

	.test-btn:disabled {
		opacity: 0.7;
		cursor: wait;
	}

	.test-btn.testing {
		color: var(--color-text-muted);
		background: var(--color-surface-hover);
		border-color: var(--color-border);
	}

	.spinner {
		display: inline-flex;
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

	.test-result {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
	}

	.test-result.success {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		color: var(--color-success);
		border: 1px solid color-mix(in srgb, var(--color-success) 25%, transparent);
	}

	.test-result.failure {
		background: color-mix(in srgb, var(--color-danger) 12%, transparent);
		color: var(--color-danger);
		border: 1px solid color-mix(in srgb, var(--color-danger) 25%, transparent);
		max-width: 100%;
	}

	.test-result.failure span {
		word-break: break-word;
	}

	.test-result-wrap {
		display: flex;
		flex-direction: column;
		gap: 8px;
		align-items: flex-start;
	}

	.retry-btn {
		padding: 5px 12px;
		background: none;
		color: var(--color-accent);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 12px;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.retry-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}
</style>
