<script lang="ts">
	import { Send, Undo2 } from 'lucide-svelte';
	import type { Snippet } from 'svelte';
	import type { ThreadBlock } from '$lib/api';
	import { saveAutoSave, clearAutoSave as clearAutoSaveStorage, readAutoSave, restoreMedia, AUTOSAVE_DEBOUNCE_MS } from '$lib/utils/composerAutosave';
	import type { RecoveryData } from '$lib/stores/composerAutosave';
	import TweetEditor from './TweetEditor.svelte';
	import ThreadFlowLane from './ThreadFlowLane.svelte';
	import AddTweetDivider from './AddTweetDivider.svelte';
	import ComposerInsertBar from './ComposerInsertBar.svelte';
	import { currentAccount } from '$lib/stores/accounts';
	import type { AttachedMedia } from './TweetEditor.svelte';

	let {
		canSubmit,
		submitting,
		selectedTime,
		submitError,
		canPublish = true,
		inspectorOpen = false,
		embedded = false,
		onsubmit,
		inspector,
		// Editor state (bindable)
		mode = $bindable<'tweet' | 'thread'>('tweet'),
		tweetText = $bindable(''),
		threadBlocks = $bindable<ThreadBlock[]>([]),
		threadValid = $bindable(false),
		attachedMedia = $bindable<AttachedMedia[]>([]),
		initialized = $bindable(false),
		showRecovery = $bindable(false),
		recoveryData = $bindable<RecoveryData | null>(null),
		showUndo = $bindable(false),
		undoMessage = $bindable('Content replaced.'),
		// Refs (bindable so parent can read them)
		threadFlowRef = $bindable<ThreadFlowLane | undefined>(undefined),
		tweetEditorRef = $bindable<TweetEditor | undefined>(undefined),
		// Callbacks
		onsubmiterror,
		onswitchtothread,
		onundo,
	}: {
		canSubmit: boolean;
		submitting: boolean;
		selectedTime: string | null;
		submitError: string | null;
		canPublish?: boolean;
		inspectorOpen?: boolean;
		embedded?: boolean;
		onsubmit: () => void;
		inspector?: Snippet;
		mode?: 'tweet' | 'thread';
		tweetText?: string;
		threadBlocks?: ThreadBlock[];
		threadValid?: boolean;
		attachedMedia?: AttachedMedia[];
		initialized?: boolean;
		showRecovery?: boolean;
		recoveryData?: RecoveryData | null;
		showUndo?: boolean;
		undoMessage?: string;
		threadFlowRef?: ThreadFlowLane;
		tweetEditorRef?: TweetEditor;
		onsubmiterror?: (msg: string) => void;
		onswitchtothread?: () => void;
		onundo?: () => void;
	} = $props();

	const avatarUrl = $derived($currentAccount?.x_avatar_url ?? null);
	const displayName = $derived($currentAccount?.x_display_name ?? null);
	const handle = $derived($currentAccount?.x_username ?? null);

	// ── Autosave ───────────────────────────────────────────
	let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;

	$effect(() => {
		void mode; void tweetText; void threadBlocks; void attachedMedia;
		if (initialized) autoSave();
	});

	function autoSave() {
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
		autoSaveTimer = setTimeout(() => {
			saveAutoSave(mode, tweetText, threadBlocks, attachedMedia);
		}, AUTOSAVE_DEBOUNCE_MS);
	}

	export function flushAutoSave() {
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
		autoSaveTimer = null;
		if (!initialized) return;
		saveAutoSave(mode, tweetText, threadBlocks, attachedMedia);
	}

	export function clearAutoSave() {
		clearAutoSaveStorage();
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
	}

	// ── Recovery ───────────────────────────────────────────
	function restoreDraft(data: NonNullable<typeof recoveryData>) {
		mode = (data.mode as 'tweet' | 'thread') ?? 'tweet';
		tweetText = data.tweetText || '';
		threadBlocks = data.blocks || [];
		attachedMedia = restoreMedia(data.tweetMedia);
	}

	export function checkRecovery() {
		const data = readAutoSave();
		if (!data) return;
		restoreDraft(data);
		initialized = true;
	}

	// ── Mode switching ─────────────────────────────────────
	export function switchToThread() {
		if (mode !== 'tweet') return;
		const mediaPaths = attachedMedia.map((m) => m.path);
		threadBlocks = [
			{ id: crypto.randomUUID(), text: tweetText, media_paths: mediaPaths, order: 0 },
			{ id: crypto.randomUUID(), text: '', media_paths: [], order: 1 }
		];
		const focusId = threadBlocks[1].id;
		tweetText = '';
		attachedMedia = [];
		mode = 'thread';
		requestAnimationFrame(() => {
			const textarea = document.querySelector(`[data-block-id="${focusId}"] textarea`) as HTMLTextAreaElement | null;
			textarea?.focus();
		});
	}

	// ── Insert text ────────────────────────────────────────
	export function handleInsertText(text: string) {
		const textarea = document.activeElement as HTMLTextAreaElement | null;
		if (!textarea || textarea.tagName !== 'TEXTAREA') {
			const fallback = document.querySelector('.compose-input, .flow-textarea') as HTMLTextAreaElement | null;
			if (fallback) {
				fallback.focus();
				const pos = fallback.selectionStart ?? fallback.value.length;
				const newVal = fallback.value.slice(0, pos) + text + fallback.value.slice(pos);
				if (mode === 'tweet') tweetText = newVal;
				fallback.value = newVal;
				fallback.dispatchEvent(new Event('input', { bubbles: true }));
				fallback.setSelectionRange(pos + text.length, pos + text.length);
			}
			return;
		}
		const pos = textarea.selectionStart ?? textarea.value.length;
		const newVal = textarea.value.slice(0, pos) + text + textarea.value.slice(pos);
		if (mode === 'tweet') tweetText = newVal;
		textarea.value = newVal;
		textarea.dispatchEvent(new Event('input', { bubbles: true }));
		textarea.setSelectionRange(pos + text.length, pos + text.length);
		textarea.focus();
	}

	// Auto-collapse: thread → tweet when only 1 block remains
	$effect(() => {
		if (mode === 'thread' && threadBlocks.length <= 1 && initialized) {
			const surviving = threadBlocks[0];
			tweetText = surviving?.text ?? '';
			threadBlocks = [];
			mode = 'tweet';
			requestAnimationFrame(() => {
				const textarea = document.querySelector('.compose-input') as HTMLTextAreaElement | null;
				textarea?.focus();
			});
		}
	});
</script>

<div class="canvas" class:with-inspector={inspectorOpen && inspector}>
	<div class="canvas-main">
		{#if mode === 'tweet'}
			<TweetEditor
				bind:this={tweetEditorRef}
				text={tweetText}
				onchange={(t) => { tweetText = t; }}
				{attachedMedia}
				onmediachange={(m) => { attachedMedia = m; }}
				onerror={(msg) => { onsubmiterror?.(msg); }}
				{avatarUrl}
				{displayName}
				{handle}
			/>
			<AddTweetDivider onclick={() => { onswitchtothread?.() ?? switchToThread(); }} disabled={!tweetText.trim()} />
		{:else}
			<ThreadFlowLane
				bind:this={threadFlowRef}
				blocks={threadBlocks}
				{avatarUrl}
				{displayName}
				{handle}
				onchange={(b) => { threadBlocks = b; }}
				onvalidchange={(v) => { threadValid = v; }}
			/>
		{/if}

		<ComposerInsertBar oninsert={handleInsertText} />

		{#if showUndo}
			<div class="undo-banner">
				<span>{undoMessage}</span>
				{#if onundo}
					<button class="undo-banner-btn" onclick={onundo} aria-label="Undo">
						<Undo2 size={12} />
						Undo
					</button>
				{/if}
			</div>
		{/if}

		{#if submitError}
			<div class="error-msg" role="alert">{submitError}</div>
		{/if}

		{#if !embedded}
			<div class="submit-anchor">
				<button
					class="submit-pill"
					onclick={onsubmit}
					disabled={!canSubmit || submitting}
					title={!canPublish && !selectedTime ? 'Connect X API to publish directly' : ''}
				>
					<Send size={14} />
					{submitting
						? 'Submitting...'
						: selectedTime
							? 'Schedule'
							: canPublish
								? 'Post now'
								: 'Save to Calendar'}
				</button>
			</div>
		{/if}
	</div>

	{#if inspectorOpen && inspector}
		<div class="canvas-inspector">
			{@render inspector()}
		</div>
	{/if}
</div>

<style>
	.canvas { display: flex; flex: 1; min-height: 0; position: relative; }
	.canvas.with-inspector { display: flex; }

	.canvas-main {
		display: flex; flex-direction: column;
		flex: 1; min-height: 0; min-width: 0;
		overflow-y: auto; max-width: 660px;
		margin-left: auto; margin-right: auto; width: 100%;
		padding: 0 24px 24px;
	}

	.canvas-main > :global(:first-child) { padding-top: 12px; }

	.canvas-inspector {
		width: 260px; flex-shrink: 0;
		border-left: 1px solid var(--color-border-subtle);
		overflow-y: auto; padding: 12px 16px;
		background: color-mix(in srgb, var(--color-base) 50%, var(--color-surface));
	}

	.error-msg {
		margin-top: 12px; padding: 8px 12px; border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger); font-size: 12px;
	}

	.undo-banner {
		display: flex; align-items: center; justify-content: space-between;
		margin-top: 8px; padding: 8px 12px; border-radius: 6px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		font-size: 12px; color: var(--color-accent);
	}

	.undo-banner-btn {
		display: inline-flex; align-items: center; gap: 4px;
		padding: 3px 10px; border: 1px solid var(--color-accent);
		border-radius: 4px; background: transparent;
		color: var(--color-accent); font-size: 11px; font-weight: 600;
		cursor: pointer; transition: background 0.1s ease;
	}

	.undo-banner-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
	}

	.submit-anchor {
		position: sticky; bottom: 0;
		display: flex; justify-content: flex-end;
		padding: 12px 0 0; pointer-events: none;
	}

	.submit-pill {
		display: flex; align-items: center; gap: 6px;
		height: 40px; padding: 0 24px; border: none; border-radius: 20px;
		background: var(--color-accent); color: #fff;
		font-size: 13px; font-weight: 500; cursor: pointer;
		pointer-events: auto; transition: all 0.15s ease;
		box-shadow: 0 2px 12px rgba(0, 0, 0, 0.3);
	}

	.submit-pill:hover:not(:disabled) {
		background: var(--color-accent-hover);
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
	}

	.submit-pill:disabled { opacity: 0.4; cursor: not-allowed; }

	@media (pointer: coarse) { .submit-pill { min-height: 44px; } }
	@media (max-width: 768px) { .canvas-inspector { display: none; } }
	@media (prefers-reduced-motion: reduce) { .submit-pill { transition: none; } }

	@media (max-width: 640px) {
		.canvas-main { padding: 0 16px 16px; }
		.submit-anchor { padding-bottom: env(safe-area-inset-bottom, 0px); }
		.submit-pill { width: 100%; justify-content: center; }
	}
</style>
