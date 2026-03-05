<script lang="ts">
	import {
		Send,
		Clock,
		Eye,
		EyeOff,
		PanelRight,
		Sparkles,
		Search,
		Loader2
	} from 'lucide-svelte';

	let {
		canSubmit,
		submitting,
		selectedTime,
		inspectorOpen,
		previewVisible,
		handle = null,
		avatarUrl = null,
		displayName = null,
		mode = 'tweet',
		blockCount = 1,
		onsubmit,
		onpublishnow,
		onschedule,
		ontoggleinspector,
		ontogglepreview,
		onopenpalette,
		onaiassist
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		inspectorOpen: boolean;
		previewVisible: boolean;
		handle?: string | null;
		avatarUrl?: string | null;
		displayName?: string | null;
		mode?: 'tweet' | 'thread';
		blockCount?: number;
		onsubmit: () => void;
		onpublishnow?: () => void;
		onschedule?: () => void;
		ontoggleinspector: () => void;
		ontogglepreview: () => void;
		onopenpalette: () => void;
		onaiassist?: () => void;
	} = $props();
</script>

<header class="home-header">
	<div class="header-left">
		{#if avatarUrl}
			<img src={avatarUrl} alt="" class="header-avatar" />
		{/if}
		{#if displayName}
			<span class="header-name">{displayName}</span>
		{/if}
		{#if handle}
			<span class="header-handle">@{handle}</span>
		{/if}
	</div>

	<div class="header-right">
		{#if selectedTime}
			<div class="button-group">
				<button
					class="cta-pill schedule-pill"
					onclick={onsubmit}
					disabled={!canSubmit || submitting}
					title="Schedule post"
					aria-label={submitting ? 'Scheduling' : 'Schedule post'}
				>
					{#if submitting}
						<Loader2 size={14} class="spin-icon" />
					{:else}
						<Clock size={14} />
					{/if}
					<span>{submitting ? 'Scheduling\u2026' : 'Schedule'}</span>
				</button>
				<button
					class="cta-pill publish-now-btn"
					onclick={onpublishnow ?? onsubmit}
					disabled={!canSubmit || submitting}
					title="Publish immediately"
					aria-label="Publish now"
				>
					<Send size={14} />
				</button>
			</div>
		{:else}
			<button
				class="cta-pill publish-pill"
				onclick={onsubmit}
				disabled={!canSubmit || submitting}
				title="Publish now"
				aria-label={submitting ? 'Posting' : 'Publish now'}
			>
				{#if submitting}
					<Loader2 size={14} class="spin-icon" />
				{:else}
					<Send size={14} />
				{/if}
				<span>{submitting ? 'Posting\u2026' : 'Publish'}</span>
			</button>
		{/if}

		<div class="icon-tools">
			<button
				class="icon-btn"
				class:active={previewVisible}
				onclick={ontogglepreview}
				aria-label={previewVisible ? 'Close preview' : 'Open preview'}
				title={previewVisible ? 'Close preview (\u2318\u21E7P)' : 'Open preview (\u2318\u21E7P)'}
			>
				{#if previewVisible}
					<Eye size={15} />
				{:else}
					<EyeOff size={15} />
				{/if}
			</button>

			{#if onaiassist}
				<button
					class="icon-btn"
					onclick={onaiassist}
					aria-label="AI improve selection or post"
					title="AI improve selection or post (\u2318\u21e7J)"
				>
					<Sparkles size={15} />
				</button>
			{/if}

			<button
				class="icon-btn"
				class:active={inspectorOpen}
				onclick={ontoggleinspector}
				aria-label={inspectorOpen ? 'Close inspector' : 'Open inspector'}
				title={inspectorOpen ? 'Close inspector (\u2318I)' : 'Open inspector (\u2318I)'}
			>
				<PanelRight size={15} />
			</button>

			<button
				class="icon-btn"
				onclick={onopenpalette}
				aria-label="Command palette"
				title="Commands (\u2318K)"
			>
				<Search size={15} />
			</button>
		</div>
	</div>
</header>

<style>
	.home-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 20px;
		flex-shrink: 0;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 50%, transparent);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.header-avatar {
		width: 24px;
		height: 24px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.header-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 120px;
	}

	.header-handle {
		font-size: 13px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 140px;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	.button-group {
		display: flex;
		align-items: center;
		gap: 3px;
	}

	.cta-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 38px;
		padding: 0 20px;
		border-radius: 20px;
		font-size: 13.5px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
		white-space: nowrap;
		border: none;
	}

	.publish-pill,
	.schedule-pill {
		background: var(--color-accent);
		color: #fff;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.publish-pill:hover:not(:disabled),
	.schedule-pill:hover:not(:disabled) {
		background: var(--color-accent-hover);
		box-shadow: 0 3px 12px rgba(0, 0, 0, 0.25);
		transform: translateY(-1px);
	}

	.publish-pill:disabled,
	.schedule-pill:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.publish-now-btn {
		background: transparent;
		border: 1.5px solid color-mix(in srgb, var(--color-accent) 50%, transparent);
		color: var(--color-accent);
		padding: 0 10px;
	}

	.publish-now-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-color: var(--color-accent);
	}

	.publish-now-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.icon-tools {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-left: 8px;
		opacity: 0.7;
		transition: opacity 0.15s ease;
	}

	.icon-tools:hover {
		opacity: 1;
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.icon-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.icon-btn.active {
		color: var(--color-accent);
	}

	.icon-btn.active:hover {
		color: var(--color-accent-hover);
	}

	:global(.spin-icon) {
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	@media (prefers-reduced-motion: reduce) {
		:global(.spin-icon) {
			animation: none;
		}

		.icon-tools {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.icon-btn {
			min-width: 44px;
			min-height: 44px;
		}
		.cta-pill {
			min-height: 44px;
		}
	}

	@media (max-width: 640px) {
		.home-header {
			gap: 8px;
		}

		.header-avatar,
		.header-name,
		.header-handle {
			display: none;
		}

		.icon-tools {
			display: none;
		}

		.cta-pill {
			padding: 0 12px;
			font-size: 12px;
			height: 34px;
		}
	}
</style>
