<script lang="ts">
	import { trackForgePromptShown, trackForgeEnabled } from '$lib/analytics/hookMinerFunnel';

	let {
		sourcePathStem = 'unknown',
		localEligible = true,
		onEnable,
		onDismiss,
	}: {
		sourcePathStem?: string;
		localEligible?: boolean;
		onEnable: () => void;
		onDismiss: () => void;
	} = $props();

	$effect(() => {
		trackForgePromptShown(sourcePathStem, localEligible);
	});

	function handleEnable() {
		trackForgeEnabled(sourcePathStem, 'prompt');
		onEnable();
	}
</script>

<div class="sync-prompt" role="status">
	<div class="sync-prompt-body">
		<span class="sync-prompt-title">Enable Analytics Sync?</span>
		<p class="sync-prompt-text">
			Your note was published successfully. TuitBot can enrich your source note with
			engagement metrics (impressions, likes, performance score) as they arrive —
			typically 15–60 minutes after posting.
		</p>
		<p class="sync-prompt-text">
			All writes are local-only and stay in your vault. No data leaves your machine.
		</p>
	</div>
	<div class="sync-prompt-actions">
		<button type="button" class="btn-enable" onclick={handleEnable}>
			Enable in Settings
		</button>
		<button type="button" class="btn-dismiss" onclick={onDismiss}>
			Not now
		</button>
	</div>
</div>

<style>
	.sync-prompt {
		padding: 16px 20px;
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 8px;
		margin-bottom: 16px;
	}
	.sync-prompt-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		display: block;
		margin-bottom: 6px;
	}
	.sync-prompt-text {
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
		margin: 0 0 4px;
	}
	.sync-prompt-actions {
		display: flex;
		gap: 10px;
		margin-top: 12px;
	}
	.btn-enable {
		padding: 6px 14px;
		background: var(--color-accent);
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		color: white;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: opacity 0.15s;
	}
	.btn-enable:hover { opacity: 0.9; }
	.btn-dismiss {
		padding: 6px 14px;
		background: transparent;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}
	.btn-dismiss:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}
</style>
