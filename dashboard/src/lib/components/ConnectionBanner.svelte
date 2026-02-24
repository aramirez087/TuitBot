<script lang="ts">
	import { WifiOff } from 'lucide-svelte';
</script>

<div class="banner">
	<WifiOff size={14} />
	<span>Server disconnected — reconnecting...</span>
	<div class="spinner"></div>
</div>

<style>
	.banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		margin-bottom: 16px;
		background: color-mix(in srgb, var(--color-warning, #f59e0b) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 30%, transparent);
		border-radius: 8px;
		color: var(--color-warning, #f59e0b);
		font-size: 13px;
		font-weight: 500;
	}

	.spinner {
		width: 14px;
		height: 14px;
		border: 2px solid color-mix(in srgb, var(--color-warning, #f59e0b) 30%, transparent);
		border-top-color: var(--color-warning, #f59e0b);
		border-radius: 50%;
		animation: spin 1s linear infinite;
		margin-left: auto;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
