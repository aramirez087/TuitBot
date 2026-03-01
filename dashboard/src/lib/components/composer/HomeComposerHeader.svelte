<script lang="ts">
	import {
		Calendar,
		Send,
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
		mode,
		blockCount = 1,
		hasContent,
		onsubmit,
		ontoggleinspector,
		ontogglepreview,
		onschedule,
		onopenpalette,
		onaiassist
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		inspectorOpen: boolean;
		previewVisible: boolean;
		handle?: string | null;
		mode: 'tweet' | 'thread';
		blockCount?: number;
		hasContent: boolean;
		onsubmit: () => void;
		ontoggleinspector: () => void;
		ontogglepreview: () => void;
		onschedule: () => void;
		onopenpalette: () => void;
		onaiassist?: () => void;
	} = $props();

	const postLabel = $derived(
		mode === 'tweet' ? '1 tweet' : `${blockCount} post${blockCount !== 1 ? 's' : ''}`
	);

	const publishLabel = $derived(
		submitting ? 'Posting\u2026' : selectedTime ? 'Schedule' : 'Publish'
	);

	function formatTime(time: string): string {
		const [h, m] = time.split(':').map(Number);
		const period = h >= 12 ? 'PM' : 'AM';
		const h12 = h % 12 || 12;
		return `${h12}:${String(m).padStart(2, '0')} ${period}`;
	}
</script>

<header class="home-header">
	<div class="header-left">
		{#if handle}
			<span class="header-handle">@{handle}</span>
			<span class="header-sep" aria-hidden="true">&middot;</span>
		{/if}
		<span class="header-meta">{postLabel}</span>
		<span class="header-dot" class:active={hasContent} aria-label={hasContent ? 'Has content' : 'Empty draft'}></span>
	</div>

	<div class="header-right">
		<button
			class="cta-pill schedule-pill"
			onclick={onschedule}
			title="Schedule post"
			aria-label={selectedTime ? `Scheduled for ${formatTime(selectedTime)}` : 'Schedule post'}
		>
			<Calendar size={14} />
			{#if selectedTime}
				<span>{formatTime(selectedTime)}</span>
			{:else}
				<span>Schedule</span>
			{/if}
		</button>

		<button
			class="cta-pill publish-pill"
			onclick={onsubmit}
			disabled={!canSubmit || submitting}
			title={selectedTime ? 'Schedule post' : 'Publish now'}
			aria-label={submitting ? 'Posting' : selectedTime ? 'Schedule post' : 'Publish now'}
		>
			{#if submitting}
				<Loader2 size={14} class="spin-icon" />
			{:else}
				<Send size={14} />
			{/if}
			<span>{publishLabel}</span>
		</button>

		<div class="icon-tools">
			<button
				class="icon-btn"
				class:active={previewVisible}
				onclick={ontogglepreview}
				aria-label={previewVisible ? 'Hide preview' : 'Show preview'}
				title={previewVisible ? 'Hide preview (\u2318\u21E7P)' : 'Show preview (\u2318\u21E7P)'}
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
					aria-label="Improve with AI"
					title="Improve with AI (\u2318J)"
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

	.header-handle {
		font-size: 13px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 140px;
	}

	.header-sep {
		color: var(--color-text-subtle);
		font-size: 12px;
	}

	.header-meta {
		font-size: 12px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	.header-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-border-subtle);
		flex-shrink: 0;
		transition: background 0.2s ease;
	}

	.header-dot.active {
		background: var(--color-success, #22c55e);
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	.cta-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 36px;
		padding: 0 16px;
		border-radius: 20px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
		white-space: nowrap;
		border: none;
	}

	.schedule-pill {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
		border: 1px solid color-mix(in srgb, var(--color-warning) 25%, transparent);
	}

	.schedule-pill:hover {
		background: color-mix(in srgb, var(--color-warning) 20%, transparent);
		border-color: color-mix(in srgb, var(--color-warning) 40%, transparent);
	}

	.publish-pill {
		background: var(--color-accent);
		color: #fff;
	}

	.publish-pill:hover:not(:disabled) {
		background: var(--color-accent-hover);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.publish-pill:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.icon-tools {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-left: 4px;
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

		.header-dot {
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
			padding: 8px 16px;
			gap: 8px;
		}

		.header-handle {
			display: none;
		}

		.header-sep {
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
