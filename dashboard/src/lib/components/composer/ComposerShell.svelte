<script lang="ts">
	import { X, Send, Maximize2, Minimize2, FileText } from 'lucide-svelte';
	import { focusTrap } from '$lib/actions/focusTrap';
	import type { Snippet } from 'svelte';

	let {
		open,
		mode,
		focusMode,
		dateLabel,
		canSubmit,
		submitting,
		assisting,
		tweetHasText,
		showRecovery,
		selectedTime,
		submitError,
		showFromNotes,
		onclose,
		ontogglefocus,
		onmodechange,
		onsubmit,
		onaiassist,
		ontogglefromnotes,
		onrecover,
		ondismissrecovery,
		children
	}: {
		open: boolean;
		mode: 'tweet' | 'thread';
		focusMode: boolean;
		dateLabel: string;
		canSubmit: boolean;
		submitting: boolean;
		assisting: boolean;
		tweetHasText: boolean;
		showRecovery: boolean;
		selectedTime: string | null;
		submitError: string | null;
		showFromNotes: boolean;
		onclose: () => void;
		ontogglefocus: () => void;
		onmodechange: (mode: 'tweet' | 'thread') => void;
		onsubmit: () => void;
		onaiassist: () => void;
		ontogglefromnotes: () => void;
		onrecover: () => void;
		ondismissrecovery: () => void;
		children: Snippet;
	} = $props();

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onclose();
	}
</script>

<div
	class="backdrop"
	onclick={handleBackdropClick}
	role="presentation"
>
	<div
		class="modal"
		class:thread-mode={mode === 'thread'}
		class:focus-mode={focusMode}
		role="dialog"
		aria-modal="true"
		aria-label="Compose content"
		use:focusTrap
	>
		{#if showRecovery}
			<div class="recovery-banner" role="alert">
				<span>Unsaved draft found. Recover it?</span>
				<div class="recovery-actions">
					<button class="recovery-btn" onclick={onrecover}>Recover</button>
					<button class="recovery-dismiss" onclick={ondismissrecovery}>Discard</button>
				</div>
			</div>
		{/if}

		<div class="modal-header">
			<div class="modal-title">
				<h2>Compose</h2>
				<span class="date-subtitle">{dateLabel}</span>
			</div>
			<div class="header-actions">
				<button
					class="focus-btn"
					onclick={ontogglefocus}
					aria-label={focusMode ? 'Exit focus mode' : 'Enter focus mode'}
					title={focusMode ? 'Exit focus mode (\u2318\u21E7F)' : 'Focus mode (\u2318\u21E7F)'}
				>
					{#if focusMode}
						<Minimize2 size={14} />
					{:else}
						<Maximize2 size={14} />
					{/if}
				</button>
				<button class="close-btn" onclick={onclose} aria-label="Close compose modal">
					<X size={16} />
				</button>
			</div>
		</div>

		<div class="mode-tabs" role="tablist" aria-label="Content type">
			<button
				class="tab"
				class:active={mode === 'tweet'}
				onclick={() => onmodechange('tweet')}
				role="tab"
				aria-selected={mode === 'tweet'}
			>
				Tweet
			</button>
			<button
				class="tab"
				class:active={mode === 'thread'}
				onclick={() => onmodechange('thread')}
				role="tab"
				aria-selected={mode === 'thread'}
			>
				Thread
			</button>
		</div>

		<div class="modal-body">
			{@render children()}

			{#if submitError}
				<div class="error-msg" role="alert">{submitError}</div>
			{/if}
		</div>

		<div class="modal-footer">
			<button class="assist-btn" onclick={onaiassist} disabled={assisting}>
				{assisting ? 'Generating...' : tweetHasText && mode === 'tweet' ? 'AI Improve' : 'AI Assist'}
			</button>
			<button
				class="notes-btn"
				onclick={ontogglefromnotes}
				title="Generate from notes"
				aria-label="Generate from notes"
			>
				<FileText size={14} />
			</button>
			<div class="footer-spacer"></div>
			<button class="cancel-btn" onclick={onclose}>Cancel</button>
			<button class="submit-btn" onclick={onsubmit} disabled={!canSubmit || submitting}>
				<Send size={14} />
				{submitting ? 'Submitting...' : selectedTime ? 'Schedule' : 'Post now'}
			</button>
		</div>
	</div>
</div>

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
		width: 520px;
		max-width: 90vw;
		max-height: 85vh;
		overflow-y: auto;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
	}

	.modal.thread-mode {
		width: 900px;
	}

	.recovery-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 20px;
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		border-bottom: 1px solid color-mix(in srgb, var(--color-warning) 25%, transparent);
		font-size: 13px;
		color: var(--color-warning);
	}

	.recovery-actions {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}

	.recovery-btn {
		padding: 4px 12px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: var(--color-warning);
		color: #000;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
	}

	.recovery-dismiss {
		padding: 4px 12px;
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		background: transparent;
		color: var(--color-warning);
		font-size: 12px;
		cursor: pointer;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.modal-title {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text);
	}

	.date-subtitle {
		font-size: 13px;
		font-weight: 400;
		color: var(--color-text-muted);
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

	.mode-tabs {
		display: flex;
		gap: 0;
		padding: 0 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.tab {
		padding: 10px 16px;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.tab:hover {
		color: var(--color-text);
	}

	.tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.modal-body {
		padding: 20px;
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

	.assist-btn {
		padding: 8px 16px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: transparent;
		color: var(--color-accent);
		font-size: 13px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.assist-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.assist-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.footer-spacer {
		flex: 1;
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

	/* Focus mode */
	.modal.focus-mode {
		width: 100vw;
		max-width: 100vw;
		height: 100vh;
		max-height: 100vh;
		border-radius: 0;
		display: flex;
		flex-direction: column;
	}

	.modal.focus-mode .modal-body {
		flex: 1;
		overflow-y: auto;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.focus-btn {
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

	.focus-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.notes-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.notes-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	/* Touch targets */
	@media (pointer: coarse) {
		.close-btn,
		.focus-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.notes-btn {
			min-width: 44px;
			min-height: 44px;
		}

		.tab {
			min-height: 44px;
		}

		.assist-btn,
		.cancel-btn,
		.submit-btn {
			min-height: 44px;
		}
	}

	/* Mobile responsive */
	@media (max-width: 768px) {
		.modal.thread-mode {
			width: 100%;
			max-width: 100vw;
			border-radius: 0;
			max-height: 100vh;
		}
	}

	@media (max-width: 640px) {
		.modal {
			width: 100vw;
			max-width: 100vw;
			height: 100vh;
			max-height: 100vh;
			border-radius: 0;
			display: flex;
			flex-direction: column;
		}

		.modal-body {
			flex: 1;
			overflow-y: auto;
			padding: 16px;
		}

		.modal-header {
			padding: 12px 16px;
		}

		.mode-tabs {
			padding: 0 16px;
		}

		.modal-footer {
			flex-wrap: wrap;
			gap: 8px;
			padding: 12px 16px;
		}

		.modal-footer .footer-spacer {
			display: none;
		}

		.modal-footer .submit-btn {
			width: 100%;
			justify-content: center;
			order: -1;
		}
	}
</style>
