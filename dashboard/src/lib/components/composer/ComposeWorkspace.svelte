<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		api,
		type ScheduleConfig,
		type ComposeRequest,
		type ThreadBlock
	} from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import { matchEvent } from '$lib/utils/shortcuts';
	import { buildComposeRequest, topicWithCue } from '$lib/utils/composeHandlers';
	import ThreadFlowLane from './ThreadFlowLane.svelte';
	import CommandPalette from '../CommandPalette.svelte';
	import ComposerShell from './ComposerShell.svelte';
	import ComposerHeaderBar from './ComposerHeaderBar.svelte';
	import HomeComposerHeader from './HomeComposerHeader.svelte';
	import ComposerCanvas from './ComposerCanvas.svelte';
	import ComposerInspector from './ComposerInspector.svelte';
	import InspectorContent from './InspectorContent.svelte';
	import RecoveryBanner from './RecoveryBanner.svelte';
	import TweetEditor from './TweetEditor.svelte';
	import ComposerPreviewSurface from './ComposerPreviewSurface.svelte';
	import VoiceContextPanel from './VoiceContextPanel.svelte';
	import ComposerPromptCard from '../home/ComposerPromptCard.svelte';
	import ComposerTipsTray from '../home/ComposerTipsTray.svelte';
	import ComposerShortcutBar from '../home/ComposerShortcutBar.svelte';
	import { currentAccount } from '$lib/stores/accounts';
	import { persistGet, persistSet } from '$lib/stores/persistence';
	import type { AttachedMedia } from './TweetEditor.svelte';

	let {
		schedule,
		onsubmit,
		onclose,
		prefillTime = null,
		prefillDate = null,
		embedded = false
	}: {
		schedule: ScheduleConfig | null;
		onsubmit: (data: ComposeRequest) => void;
		onclose?: () => void;
		prefillTime?: string | null;
		prefillDate?: Date | null;
		embedded?: boolean;
	} = $props();

	// ── State ──────────────────────────────────────────────
	let mode = $state<'tweet' | 'thread'>('tweet');
	let tweetText = $state('');
	let threadBlocks = $state<ThreadBlock[]>([]);
	let threadValid = $state(false);
	let selectedTime = $state<string | null>(null);
	let submitting = $state(false);
	let submitError = $state<string | null>(null);
	let attachedMedia = $state<AttachedMedia[]>([]);
	let focusMode = $state(false);
	let paletteOpen = $state(false);
	let threadFlowRef: ThreadFlowLane | undefined = $state();
	let tweetEditorRef: TweetEditor | undefined = $state();
	let voicePanelRef: VoiceContextPanel | undefined = $state();
	let showFromNotes = $state(false);
	let assisting = $state(false);
	let voiceCue = $state('');
	let previewMode = $state(false);
	let inspectorOpen = $state(loadInspectorState());
	let isMobile = $state(false);
	let statusAnnouncement = $state('');

	// Home-surface state (only active when embedded)
	let tipsVisible = $state(false);
	let promptDismissed = $state(false);

	// Undo state for notes generation
	let undoSnapshot = $state<{
		mode: 'tweet' | 'thread'; text: string; blocks: ThreadBlock[];
		media?: AttachedMedia[]; selectedTime?: string | null;
	} | null>(null);
	let showUndo = $state(false);
	let undoMessage = $state('Content replaced.');
	let undoTimer: ReturnType<typeof setTimeout> | null = null;

	// Auto-save
	const AUTOSAVE_KEY = 'tuitbot:compose:draft';
	const AUTOSAVE_DEBOUNCE_MS = 500;
	const AUTOSAVE_TTL_MS = 7 * 24 * 60 * 60 * 1000;
	let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
	let showRecovery = $state(false);
	let recoveryData = $state<{
		mode: string; tweetText: string; blocks: ThreadBlock[]; timestamp: number;
	} | null>(null);

	// ── Derived ────────────────────────────────────────────
	const targetDate = $derived(prefillDate ?? new Date());
	const TWEET_MAX = 280;
	const tweetChars = $derived(tweetWeightedLen(tweetText));
	const tweetOverLimit = $derived(tweetChars > TWEET_MAX);
	const canSubmitTweet = $derived(tweetText.trim().length > 0 && !tweetOverLimit);
	const canSubmit = $derived(mode === 'tweet' ? canSubmitTweet : threadValid);

	const sortedPreviewBlocks = $derived(
		[...threadBlocks].sort((a, b) => a.order - b.order).filter((b) => b.text.trim().length > 0)
	);

	const hasExistingContent = $derived(
		mode === 'tweet' ? tweetText.trim().length > 0 : threadBlocks.some((b) => b.text.trim().length > 0)
	);

	const tweetMediaPreviewMap = $derived(
		new Map(attachedMedia.map((m) => [m.path, m.previewUrl]))
	);

	const hasPreviewContent = $derived(
		mode === 'thread'
			? sortedPreviewBlocks.length > 0
			: tweetText.trim().length > 0
	);

	const desktopInspectorOpen = $derived(inspectorOpen && !isMobile);

	const showPromptCard = $derived(
		embedded && !hasExistingContent && !promptDismissed
	);

	const threadBlockCount = $derived(
		mode === 'thread' ? threadBlocks.filter((b) => b.text.trim().length > 0).length || threadBlocks.length : 1
	);

	// ── Inspector persistence ──────────────────────────────
	function loadInspectorState(): boolean {
		try {
			const saved = localStorage.getItem('tuitbot:inspector:open');
			return saved === null ? true : saved === 'true';
		} catch {
			return true;
		}
	}

	function persistInspectorState(v: boolean) {
		try { localStorage.setItem('tuitbot:inspector:open', String(v)); } catch { /* quota */ }
	}

	function toggleInspector() {
		inspectorOpen = !inspectorOpen;
		persistInspectorState(inspectorOpen);
	}

	function togglePreview() {
		previewMode = !previewMode;
	}

	// ── Lifecycle ──────────────────────────────────────────
	$effect(() => {
		const mql = window.matchMedia('(max-width: 768px)');
		isMobile = mql.matches;
		const handler = (e: MediaQueryListEvent) => { isMobile = e.matches; };
		mql.addEventListener('change', handler);
		return () => mql.removeEventListener('change', handler);
	});

	$effect(() => { void mode; void tweetText; void threadBlocks; autoSave(); });

	// Announce mode switches to screen readers (skip initial render)
	let modeInitialized = false;
	$effect(() => {
		if (!modeInitialized) { modeInitialized = true; return; }
		statusAnnouncement = mode === 'tweet' ? 'Switched to tweet mode' : 'Switched to thread mode';
	});

	onMount(async () => {
		selectedTime = prefillTime ?? null;
		checkRecovery();
		if (!showRecovery) {
			tweetText = '';
			threadBlocks = [];
			mode = 'tweet';
		}
		submitting = false;
		submitError = null;
		focusMode = false;
		paletteOpen = false;
		showFromNotes = false;
		voiceCue = '';
		undoSnapshot = null;
		showUndo = false;
		previewMode = false;
		inspectorOpen = loadInspectorState();

		if (embedded) {
			const tipsDismissed = await persistGet('home_tips_dismissed', false);
			tipsVisible = !tipsDismissed;
			window.addEventListener('tuitbot:compose', handleComposeEvent);
		}
	});

	onDestroy(() => {
		for (const m of attachedMedia) URL.revokeObjectURL(m.previewUrl);
		if (undoSnapshot?.media) {
			for (const m of undoSnapshot.media) URL.revokeObjectURL(m.previewUrl);
		}
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
		if (undoTimer) clearTimeout(undoTimer);
		if (embedded) window.removeEventListener('tuitbot:compose', handleComposeEvent);
	});

	function handleComposeEvent() {
		const textarea = document.querySelector('.compose-input') as HTMLTextAreaElement | null;
		textarea?.focus();
	}

	function switchMode(newMode: 'tweet' | 'thread') {
		if (newMode === mode) return;
		if (newMode === 'thread' && tweetText.trim()) {
			const hasThreadContent = threadBlocks.some((b) => b.text.trim());
			if (!hasThreadContent) {
				threadBlocks = [
					{ id: crypto.randomUUID(), text: tweetText, media_paths: [], order: 0 },
					{ id: crypto.randomUUID(), text: '', media_paths: [], order: 1 }
				];
				tweetText = '';
			}
		}
		mode = newMode;
	}

	// ── Autosave / Recovery ────────────────────────────────
	function autoSave() {
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
		autoSaveTimer = setTimeout(() => {
			const payload = { mode, tweetText, blocks: threadBlocks, timestamp: Date.now() };
			try { localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(payload)); } catch { /* quota */ }
		}, AUTOSAVE_DEBOUNCE_MS);
	}

	function clearAutoSave() {
		localStorage.removeItem(AUTOSAVE_KEY);
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
	}

	function checkRecovery() {
		try {
			const raw = localStorage.getItem(AUTOSAVE_KEY);
			if (!raw) return;
			const data = JSON.parse(raw);
			if (Date.now() - data.timestamp > AUTOSAVE_TTL_MS) {
				localStorage.removeItem(AUTOSAVE_KEY);
				return;
			}
			const hasContent = data.tweetText?.trim() || data.blocks?.some((b: ThreadBlock) => b.text.trim());
			if (hasContent) { recoveryData = data; showRecovery = true; }
		} catch { localStorage.removeItem(AUTOSAVE_KEY); }
	}

	function recoverDraft() {
		if (!recoveryData) return;
		mode = (recoveryData.mode as 'tweet' | 'thread') ?? 'tweet';
		tweetText = recoveryData.tweetText || '';
		threadBlocks = recoveryData.blocks || [];
		showRecovery = false;
		clearAutoSave();
	}

	function dismissRecovery() { showRecovery = false; clearAutoSave(); }

	// ── Handlers ───────────────────────────────────────────
	async function handleSubmit() {
		if (!canSubmit || submitting) return;
		submitting = true; submitError = null;
		try {
			const data = buildComposeRequest({
				mode, tweetText, threadBlocks, selectedTime, targetDate, attachedMedia
			});
			clearAutoSave();
			onsubmit(data);

			// In embedded mode (full-page), reset state after submit since the component doesn't unmount
			if (embedded) {
				// Snapshot current state so user can undo the clear
				undoSnapshot = {
					mode, text: tweetText, blocks: [...threadBlocks],
					media: [...attachedMedia], selectedTime
				};
				undoMessage = 'Published.';

				tweetText = '';
				threadBlocks = [];
				mode = 'tweet';
				selectedTime = null;
				// Don't revoke media URLs yet — undo may need them
				attachedMedia = [];
				submitting = false;
				submitError = null;
				focusMode = false;
				showFromNotes = false;
				voiceCue = '';
				previewMode = false;

				showUndo = true;
				if (undoTimer) clearTimeout(undoTimer);
				undoTimer = setTimeout(() => {
					showUndo = false;
					// Revoke media URLs now that undo window has closed
					if (undoSnapshot?.media) {
						for (const m of undoSnapshot.media) URL.revokeObjectURL(m.previewUrl);
					}
					undoSnapshot = null;
				}, 10000);
			}
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'Failed to submit';
			submitting = false;
		}
	}

	function handleClose() {
		onclose?.();
	}

	function toggleFocusMode() {
		if (embedded) return; // Already full-page
		focusMode = !focusMode;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (paletteOpen) return;

		// When preview overlay is open, only allow Escape and toggle
		if (previewMode) {
			if (e.key === 'Escape') { e.preventDefault(); previewMode = false; return; }
			if (matchEvent(e, 'cmd+shift+p')) { e.preventDefault(); togglePreview(); return; }
			return;
		}

		if (matchEvent(e, 'cmd+k')) { e.preventDefault(); paletteOpen = true; return; }
		if (matchEvent(e, 'cmd+shift+f')) {
			e.preventDefault();
			if (!embedded) toggleFocusMode();
			return;
		}
		if (matchEvent(e, 'cmd+shift+enter')) { e.preventDefault(); handleSubmit(); return; }
		if (matchEvent(e, 'cmd+enter')) {
			if (mode === 'tweet') { e.preventDefault(); handleSubmit(); }
			// In thread mode: let event propagate to ThreadFlowLane's card handler for split
			return;
		}
		if (matchEvent(e, 'cmd+j')) { e.preventDefault(); handleInlineAssist(); return; }
		if (matchEvent(e, 'cmd+shift+n')) { e.preventDefault(); switchMode('tweet'); return; }
		if (matchEvent(e, 'cmd+shift+t')) { e.preventDefault(); switchMode('thread'); return; }
		if (matchEvent(e, 'cmd+i')) { e.preventDefault(); toggleInspector(); return; }
		if (matchEvent(e, 'cmd+shift+p')) { e.preventDefault(); togglePreview(); return; }
		if (e.key === 'Escape') {
			if (showFromNotes) showFromNotes = false;
			else if (isMobile && inspectorOpen) inspectorOpen = false;
			else if (!embedded && focusMode) focusMode = false;
			else if (!embedded) handleClose();
			return;
		}
	}

	function handlePaletteAction(actionId: string) {
		paletteOpen = false;
		switch (actionId) {
			case 'focus-mode': toggleFocusMode(); break;
			case 'mode-tweet': switchMode('tweet'); break;
			case 'mode-thread': switchMode('thread'); break;
			case 'submit': handleSubmit(); break;
			case 'ai-improve': handleInlineAssist(); break;
			case 'ai-from-notes': showFromNotes = true; if (!inspectorOpen) inspectorOpen = true; break;
			case 'ai-generate': handleAiAssist(); break;
			case 'toggle-inspector': toggleInspector(); break;
			case 'toggle-preview': togglePreview(); break;
			case 'attach-media': tweetEditorRef?.triggerFileSelect(); break;
			case 'add-card': case 'duplicate': case 'split': case 'merge':
			case 'move-up': case 'move-down':
				threadFlowRef?.handlePaletteAction(actionId); break;
		}
	}

	async function handleInlineAssist() {
		if (mode === 'tweet') {
			const textarea = document.querySelector('.compose-input') as HTMLTextAreaElement | null;
			if (!textarea) return;
			const start = textarea.selectionStart;
			const end = textarea.selectionEnd;
			const selectedText = start !== end ? tweetText.slice(start, end) : tweetText;
			if (!selectedText.trim()) return;

			// Snapshot before replacement for undo
			undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };
			undoMessage = 'Content replaced.';

			assisting = true; submitError = null;
			try {
				const result = await api.assist.improve(selectedText, voiceCue || undefined);
				if (start !== end) {
					tweetText = tweetText.slice(0, start) + result.content + tweetText.slice(end);
				} else {
					tweetText = result.content;
				}
				voicePanelRef?.saveCueToHistory();
				showUndo = true;
				if (undoTimer) clearTimeout(undoTimer);
				undoTimer = setTimeout(() => { showUndo = false; }, 10000);
			} catch (e) {
				submitError = e instanceof Error ? e.message : 'AI assist failed';
				undoSnapshot = null;
			} finally { assisting = false; }
		} else {
			// Thread mode: snapshot all blocks before delegating
			undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };
			undoMessage = 'Content replaced.';
			try {
				await threadFlowRef?.handleInlineAssist(voiceCue || undefined);
				showUndo = true;
				if (undoTimer) clearTimeout(undoTimer);
				undoTimer = setTimeout(() => { showUndo = false; }, 10000);
			} catch {
				undoSnapshot = null;
			}
		}
	}

	async function handleGenerateFromNotes(notesInput: string) {
		submitError = null;
		undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };
		undoMessage = 'Content replaced.';

		if (mode === 'thread') {
			const result = await api.assist.thread(topicWithCue(voiceCue, notesInput));
			threadBlocks = result.tweets.map((text, i) => ({
				id: crypto.randomUUID(), text, media_paths: [], order: i
			}));
		} else {
			const context = voiceCue
				? `${voiceCue}. Expand these rough notes into a polished tweet`
				: 'Expand these rough notes into a polished tweet';
			const result = await api.assist.improve(notesInput, context);
			tweetText = result.content;
		}
		voicePanelRef?.saveCueToHistory();
		showFromNotes = false;
		showUndo = true;
		if (undoTimer) clearTimeout(undoTimer);
		undoTimer = setTimeout(() => { showUndo = false; }, 10000);
	}

	function handleUndo() {
		if (!undoSnapshot) return;
		mode = undoSnapshot.mode;
		tweetText = undoSnapshot.text;
		threadBlocks = undoSnapshot.blocks;
		if (undoSnapshot.media) attachedMedia = undoSnapshot.media;
		if (undoSnapshot.selectedTime !== undefined) selectedTime = undoSnapshot.selectedTime;
		undoSnapshot = null;
		showUndo = false;
		if (undoTimer) clearTimeout(undoTimer);
	}

	async function dismissTips() {
		tipsVisible = false;
		await persistSet('home_tips_dismissed', true);
	}

	function handleUseExample(text: string) {
		if (mode === 'tweet') {
			tweetText = text;
		} else {
			const sorted = [...threadBlocks].sort((a, b) => a.order - b.order);
			if (sorted.length > 0 && sorted[0].text.trim() === '') {
				threadBlocks = threadBlocks.map((b) =>
					b.id === sorted[0].id ? { ...b, text } : b
				);
			}
		}
		promptDismissed = true;
	}

	function openScheduleInInspector() {
		if (!inspectorOpen) {
			inspectorOpen = true;
			persistInspectorState(true);
		}
	}

	async function handleAiAssist() {
		assisting = true; submitError = null;
		try {
			if (mode === 'tweet') {
				if (tweetText.trim()) {
					const result = await api.assist.improve(tweetText, voiceCue || undefined);
					tweetText = result.content;
				} else {
					const result = await api.assist.tweet(topicWithCue(voiceCue, 'general'));
					tweetText = result.content;
				}
			} else {
				const result = await api.assist.thread(topicWithCue(voiceCue, 'general'));
				threadBlocks = result.tweets.map((text, i) => ({
					id: crypto.randomUUID(), text, media_paths: [], order: i
				}));
			}
			voicePanelRef?.saveCueToHistory();
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'AI assist failed';
		} finally { assisting = false; }
	}

</script>

<svelte:window onkeydown={handleKeydown} />

<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{statusAnnouncement}</div>

{#snippet composeBody()}
	{#if showRecovery}
		<RecoveryBanner onrecover={recoverDraft} ondismiss={dismissRecovery} />
	{/if}

	<ComposerCanvas
		{canSubmit} {submitting} {selectedTime} {submitError}
		inspectorOpen={desktopInspectorOpen}
		{embedded}
		onsubmit={handleSubmit}
	>
		{#snippet children()}
			{#if mode === 'tweet'}
				<TweetEditor
					bind:this={tweetEditorRef}
					text={tweetText}
					onchange={(t) => { tweetText = t; }}
					{attachedMedia}
					onmediachange={(m) => { attachedMedia = m; }}
					onerror={(msg) => { submitError = msg; }}
				/>
			{:else}
				<ThreadFlowLane
					bind:this={threadFlowRef}
					initialBlocks={threadBlocks.length > 0 ? threadBlocks : undefined}
					onchange={(b) => { threadBlocks = b; }}
					onvalidchange={(v) => { threadValid = v; }}
				/>
			{/if}

			{#if showUndo && !showFromNotes}
				<div class="undo-banner">
					<span>{undoMessage}</span>
					<button class="undo-btn" onclick={handleUndo}>Undo</button>
				</div>
			{/if}
		{/snippet}

		{#snippet inspector()}
			<InspectorContent
				{schedule} {selectedTime} {targetDate} {voiceCue}
				{assisting} {hasExistingContent} {showFromNotes} {showUndo} {mode}
				bind:voicePanelRef={voicePanelRef}
				onselect={(time) => { selectedTime = time; }}
				oncuechange={(c) => { voiceCue = c; }}
				onaiassist={handleAiAssist}
				onopenotes={() => { showFromNotes = true; }}
				ongenerate={handleGenerateFromNotes}
				onclosenotes={() => { showFromNotes = false; }}
				onundo={handleUndo}
			/>
		{/snippet}
	</ComposerCanvas>

	{#if isMobile && inspectorOpen}
		<ComposerInspector
			open={inspectorOpen}
			mobile={true}
			onclose={() => { inspectorOpen = false; }}
		>
			{#snippet children()}
				<InspectorContent
					{schedule} {selectedTime} {targetDate} {voiceCue}
					{assisting} {hasExistingContent} {showFromNotes} {showUndo} {mode}
					bind:voicePanelRef={voicePanelRef}
					onselect={(time) => { selectedTime = time; }}
					oncuechange={(c) => { voiceCue = c; }}
					onaiassist={handleAiAssist}
					onopenotes={() => { showFromNotes = true; }}
					ongenerate={handleGenerateFromNotes}
					onclosenotes={() => { showFromNotes = false; }}
					onundo={handleUndo}
				/>
			{/snippet}
		</ComposerInspector>
	{/if}

	{#if paletteOpen}
		<CommandPalette
			open={paletteOpen}
			{mode}
			onclose={() => { paletteOpen = false; }}
			onaction={handlePaletteAction}
		/>
	{/if}
{/snippet}

{#if !embedded}
	<ComposerShell
		open={true}
		{focusMode}
		inspectorOpen={desktopInspectorOpen}
		onclose={handleClose}
	>
		{#snippet children()}
			<ComposerHeaderBar
				{focusMode}
				inspectorOpen={inspectorOpen}
				previewVisible={previewMode}
				ontogglefocus={toggleFocusMode}
				ontoggleinspector={toggleInspector}
				ontogglepreview={togglePreview}
				onclose={handleClose}
			/>
			{@render composeBody()}
		{/snippet}
	</ComposerShell>
{:else}
	<div class="embedded-workspace">
		<HomeComposerHeader
			{canSubmit}
			{submitting}
			{selectedTime}
			{inspectorOpen}
			previewVisible={previewMode}
			handle={$currentAccount?.x_username ?? null}
			{mode}
			blockCount={threadBlockCount}
			hasContent={hasExistingContent}
			onsubmit={handleSubmit}
			ontoggleinspector={toggleInspector}
			ontogglepreview={togglePreview}
			onopenpalette={() => { paletteOpen = true; }}
			onaiassist={handleInlineAssist}
		/>
		{#if tipsVisible}
			<ComposerTipsTray
				visible={tipsVisible}
				{mode}
				ondismiss={dismissTips}
			/>
		{:else}
			<ComposerShortcutBar
				{mode}
				onopenpalette={() => { paletteOpen = true; }}
				onswitchmode={() => { switchMode(mode === 'tweet' ? 'thread' : 'tweet'); }}
			/>
		{/if}
		{@render composeBody()}
		{#if showPromptCard}
			<ComposerPromptCard
				visible={showPromptCard}
				{mode}
				ondismiss={() => { promptDismissed = true; }}
				onuseexample={handleUseExample}
			/>
		{/if}
	</div>
{/if}

{#if previewMode}
	<ComposerPreviewSurface
		{mode}
		{tweetText}
		blocks={sortedPreviewBlocks}
		tweetMediaPaths={attachedMedia.map((m) => m.path)}
		tweetLocalPreviews={tweetMediaPreviewMap}
		handle={$currentAccount?.x_username ? `@${$currentAccount.x_username}` : '@you'}
		onclose={() => { previewMode = false; }}
	/>
{/if}

<style>
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}

	.embedded-workspace {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		position: relative;
	}

	/* Undo banner */
	.undo-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: 8px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		font-size: 12px;
		color: var(--color-accent);
	}

	.undo-btn {
		padding: 4px 10px;
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		background: transparent;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.undo-btn:hover {
		background: var(--color-accent);
		color: #fff;
	}

	@media (prefers-reduced-motion: reduce) {
		.undo-btn {
			transition: none;
		}
	}
</style>
