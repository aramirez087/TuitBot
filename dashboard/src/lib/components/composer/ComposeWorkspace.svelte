<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { api, type ScheduleConfig, type ComposeRequest, type ThreadBlock } from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import { buildComposeRequest } from '$lib/utils/composeHandlers';
	import { buildScheduledFor } from '$lib/utils/timezone';
	import { clearSessionFlag, markSessionActive } from '$lib/utils/composerAutosave';
	import ThreadFlowLane from './ThreadFlowLane.svelte';
	import ComposerShell from './ComposerShell.svelte';
	import ComposerHeaderBar from './ComposerHeaderBar.svelte';
	import HomeComposerHeader from './HomeComposerHeader.svelte';
	import ComposerCanvas from './ComposerCanvas.svelte';
	import ComposerInspector from './ComposerInspector.svelte';
	import ThreadPreviewRail from './ThreadPreviewRail.svelte';
	import ComposerToolbar from './ComposerToolbar.svelte';
	import { currentAccount } from '$lib/stores/accounts';

	import type { AttachedMedia } from './TweetEditor.svelte';
	import type TweetEditor from './TweetEditor.svelte';
	import type VoiceContextPanel from './VoiceContextPanel.svelte';

	let {
		schedule,
		onsubmit,
		onclose,
		prefillTime = null,
		prefillDate = null,
		embedded = false,
		canPublish = true,
		// DraftStudio integration props (optional, handled externally)
		draftId: _draftId = undefined,
		initialContent: _initialContent = undefined,
		onsyncstatus: _onsyncstatus = undefined,
		extraPaletteActions: _extraPaletteActions = undefined,
		ondraftaction: _ondraftaction = undefined,
		headerLeft: _headerLeft = undefined,
	}: {
		schedule: ScheduleConfig | null;
		onsubmit: (data: ComposeRequest) => void | Promise<void>;
		onclose?: () => void;
		prefillTime?: string | null;
		prefillDate?: Date | null;
		embedded?: boolean;
		canPublish?: boolean;
		draftId?: unknown;
		initialContent?: unknown;
		onsyncstatus?: unknown;
		extraPaletteActions?: unknown;
		ondraftaction?: unknown;
		headerLeft?: unknown;
	} = $props();

	// ── State ──────────────────────────────────────────────
	let mode = $state<'tweet' | 'thread'>('tweet');
	let tweetText = $state('');
	let threadBlocks = $state<ThreadBlock[]>([]);
	let threadValid = $state(false);
	let selectedTime = $state<string | null>(null);
	let scheduledDate = $state<string | null>(null);
	let submitting = $state(false);
	let submitError = $state<string | null>(null);
	let attachedMedia = $state<AttachedMedia[]>([]);
	let focusMode = $state(false);
	let previewMode = $state(false);
	let inspectorOpen = $state(loadInspectorState());
	let isMobile = $state(false);
	let statusAnnouncement = $state('');
	let showUndo = $state(false);
	let undoMessage = $state('Content replaced.');
	let notesPanelMode = $state<'notes' | 'vault' | null>(null);
	let assisting = $state(false);
	let voiceCue = $state('');
	let initialized = $state(false);
	let showRecovery = $state(false);
	let recoveryData = $state<{ mode: string; tweetText: string; blocks: ThreadBlock[]; timestamp: number; tweetMedia?: Array<{ path: string; mediaType: string; altText?: string }> } | null>(null);
	let undoSnapshot = $state<{ mode: 'tweet' | 'thread'; text: string; blocks: ThreadBlock[]; media?: AttachedMedia[]; selectedTime?: string | null; scheduledDate?: string | null } | null>(null);


	// Component refs
	let threadFlowRef = $state<ThreadFlowLane | undefined>();
	let tweetEditorRef = $state<TweetEditor | undefined>();
	let voicePanelRef = $state<VoiceContextPanel | undefined>();
	let canvasRef: ComposerCanvas | undefined = $state();
	let inspectorRef: ComposerInspector | undefined = $state();

	// ── Derived ────────────────────────────────────────────
	const accountTimezone = $derived(schedule?.timezone ?? 'UTC');
	const targetDate = $derived(prefillDate ?? new Date());
	const scheduledFor = $derived.by(() => {
		if (!selectedTime || !scheduledDate) return null;
		try { return buildScheduledFor(scheduledDate, selectedTime, accountTimezone); } catch { return null; }
	});
	const TWEET_MAX = 280;
	const tweetChars = $derived(tweetWeightedLen(tweetText));
	const canSubmitTweet = $derived(tweetText.trim().length > 0 && tweetChars <= TWEET_MAX);
	const canSubmit = $derived(mode === 'tweet' ? canSubmitTweet : threadValid);
	const hasExistingContent = $derived(
		mode === 'tweet' ? tweetText.trim().length > 0 : threadBlocks.some((b) => b.text.trim().length > 0)
	);
	const desktopInspectorOpen = $derived(inspectorOpen && !isMobile);

	const threadBlockCount = $derived(
		mode === 'thread' ? threadBlocks.filter((b) => b.text.trim().length > 0).length || threadBlocks.length : 1
	);
	const sortedPreviewBlocks = $derived(
		[...threadBlocks].sort((a, b) => a.order - b.order).filter((b) => b.text.trim().length > 0)
	);
	const tweetMediaPreviewMap = $derived(new Map(attachedMedia.map((m) => [m.path, m.previewUrl])));

	// ── Inspector persistence ──────────────────────────────
	function loadInspectorState(): boolean {
		try { return (localStorage.getItem('tuitbot:inspector:open') ?? 'true') === 'true'; } catch { return true; }
	}

	function toggleInspector() {
		inspectorOpen = !inspectorOpen;
		try { localStorage.setItem('tuitbot:inspector:open', String(inspectorOpen)); } catch { /* quota */ }
	}

	// ── Effects ────────────────────────────────────────────
	$effect(() => {
		const mql = window.matchMedia('(max-width: 768px)');
		isMobile = mql.matches;
		const handler = (e: MediaQueryListEvent) => { isMobile = e.matches; };
		mql.addEventListener('change', handler);
		return () => mql.removeEventListener('change', handler);
	});

	let modeInitialized = false;
	$effect(() => {
		if (!modeInitialized) { modeInitialized = true; return; }
		statusAnnouncement = mode === 'tweet' ? 'Switched to tweet mode' : 'Switched to thread mode';
	});

	// ── Lifecycle ──────────────────────────────────────────
	onMount(async () => {
		selectedTime = prefillTime ?? null;
		if (prefillDate) {
			const y = prefillDate.getFullYear();
			const mo = String(prefillDate.getMonth() + 1).padStart(2, '0');
			const d = String(prefillDate.getDate()).padStart(2, '0');
			scheduledDate = `${y}-${mo}-${d}`;
		}
		canvasRef?.checkRecovery();
		if (!showRecovery && !initialized) {
			tweetText = ''; threadBlocks = []; mode = 'tweet'; initialized = true;
		}
		submitting = false; submitError = null; focusMode = false;
		notesPanelMode = null; voiceCue = ''; previewMode = false;
		inspectorOpen = loadInspectorState();
		window.addEventListener('beforeunload', handleBeforeUnload);
		if (embedded) {
			window.addEventListener('tuitbot:compose', handleComposeEvent);
		}
	});

	onDestroy(() => {
		window.removeEventListener('beforeunload', handleBeforeUnload);
		canvasRef?.flushAutoSave();
		if (embedded) window.removeEventListener('tuitbot:compose', handleComposeEvent);
		markSessionActive();
	});

	function handleBeforeUnload() { canvasRef?.flushAutoSave(); markSessionActive(); }
	function handleComposeEvent() {
		(document.querySelector('.compose-input') as HTMLTextAreaElement | null)?.focus();
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

	// ── Submit ─────────────────────────────────────────────
	async function handleSubmit() {
		if (!canSubmit || submitting) return;
		submitting = true; submitError = null;
		try {
			const data = buildComposeRequest({ mode, tweetText, threadBlocks, selectedTime, targetDate, attachedMedia, timezone: accountTimezone, scheduledDate });
			canvasRef?.clearAutoSave();
			clearSessionFlag();
			await onsubmit(data);
			if (embedded) {
				undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks], media: [...attachedMedia], selectedTime, scheduledDate };
				undoMessage = canPublish && !selectedTime ? 'Published.' : 'Saved to calendar.';
				tweetText = ''; threadBlocks = []; mode = 'tweet'; selectedTime = null; scheduledDate = null; attachedMedia = [];
				submitting = false; submitError = null; focusMode = false; notesPanelMode = null; voiceCue = ''; previewMode = false;
				showUndo = true;
				setTimeout(() => {
					showUndo = false;
					if (undoSnapshot?.media) { for (const m of undoSnapshot.media) URL.revokeObjectURL(m.previewUrl); }
					undoSnapshot = null;
				}, 10000);
			}
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'Failed to submit';
			submitting = false;
		}
	}

	function handleUndo() {
		if (!undoSnapshot) return;
		mode = undoSnapshot.mode; tweetText = undoSnapshot.text; threadBlocks = undoSnapshot.blocks;
		if (undoSnapshot.media) attachedMedia = undoSnapshot.media;
		if (undoSnapshot.selectedTime !== undefined) selectedTime = undoSnapshot.selectedTime;
		if (undoSnapshot.scheduledDate !== undefined) scheduledDate = undoSnapshot.scheduledDate;
		undoSnapshot = null; showUndo = false;
	}

	function handleClose() { onclose?.(); }
	function toggleFocusMode() { if (embedded) return; focusMode = !focusMode; }
	function togglePreview() { previewMode = !previewMode; }

	// ── Action bus (from ComposerToolbar) ─────────────────
	function handleAction(action: string) {
		switch (action) {
			case 'focus-mode': toggleFocusMode(); break;
			case 'mode-tweet': switchMode('tweet'); break;
			case 'mode-thread': switchMode('thread'); break;
			case 'submit': handleSubmit(); break;
			case 'switch-to-thread': canvasRef?.switchToThread(); break;
			case 'ai-inline': inspectorRef?.handleInlineAssist(() => {
				undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };
				undoMessage = 'Content replaced.';
			}); break;
			case 'ai-from-notes': notesPanelMode = 'notes'; if (!inspectorOpen) toggleInspector(); break;
			case 'ai-generate': inspectorRef?.handleAiAssist(); break;
			case 'toggle-inspector': toggleInspector(); break;
			case 'toggle-preview': togglePreview(); break;
			case 'close-preview': previewMode = false; break;
			case 'close-notes': notesPanelMode = null; break;
			case 'close-mobile-inspector': if (isMobile) inspectorOpen = false; break;
			case 'exit-focus': if (!embedded) focusMode = false; break;
			case 'close': handleClose(); break;
		}
	}
</script>

<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{statusAnnouncement}</div>

{#snippet composeBody()}
	<ComposerCanvas
		bind:this={canvasRef}
		{canSubmit} {submitting} {selectedTime} {submitError} {canPublish}
		inspectorOpen={desktopInspectorOpen}
		{embedded}
		onsubmit={handleSubmit}
		bind:mode bind:tweetText bind:threadBlocks bind:threadValid
		bind:attachedMedia bind:initialized bind:showRecovery bind:recoveryData
		bind:showUndo bind:undoMessage
		bind:threadFlowRef bind:tweetEditorRef
		onsubmiterror={(msg) => { submitError = msg; }}
	>
		{#snippet inspector()}
			<ComposerInspector
				bind:this={inspectorRef}
				open={true}
				isMobile={false}
				bind:assisting bind:voiceCue bind:notesPanelMode bind:showUndo bind:undoMessage
				bind:tweetText bind:threadBlocks bind:selectedTime bind:scheduledDate bind:voicePanelRef
				{mode} {schedule} {targetDate} timezone={accountTimezone} {hasExistingContent} {threadFlowRef}
				onundo={handleUndo}
				onsubmiterror={(msg) => { submitError = msg; }}
			/>
		{/snippet}
	</ComposerCanvas>

	{#if isMobile && inspectorOpen}
		<ComposerInspector
			open={inspectorOpen}
			isMobile={true}
			bind:assisting bind:voiceCue bind:notesPanelMode bind:showUndo bind:undoMessage
			bind:tweetText bind:threadBlocks bind:selectedTime bind:scheduledDate bind:voicePanelRef
			{mode} {schedule} {targetDate} timezone={accountTimezone} {hasExistingContent} {threadFlowRef}
			onclose={() => { inspectorOpen = false; }}
			onundo={handleUndo}
			onsubmiterror={(msg) => { submitError = msg; }}
		/>
	{/if}
{/snippet}

{#if !embedded}
	<ComposerShell open={true} {focusMode} inspectorOpen={desktopInspectorOpen} onclose={handleClose}>
		{#snippet children()}
			<ComposerHeaderBar
				{focusMode} {inspectorOpen} previewVisible={previewMode}
				ontogglefocus={toggleFocusMode} ontoggleinspector={toggleInspector}
				ontogglepreview={togglePreview} onclose={handleClose}
			/>
			<ComposerToolbar
				{mode} {embedded} {canSubmit} {focusMode} {previewMode}
				{inspectorOpen} showFromNotes={notesPanelMode === 'notes'} {isMobile}
				{threadFlowRef} {tweetEditorRef} {attachedMedia}
				onaiassist={() => inspectorRef?.handleAiAssist()}
				onaction={handleAction}
				onmediachange={(m) => { attachedMedia = m; }}
			/>
			{@render composeBody()}
		{/snippet}
	</ComposerShell>
{:else}
	<div class="embedded-workspace">
		<HomeComposerHeader
			{canSubmit} {submitting} {selectedTime} {inspectorOpen} {canPublish}
			previewVisible={previewMode}
			handle={$currentAccount?.x_username ?? null}
			avatarUrl={$currentAccount?.x_avatar_url ?? null}
			displayName={$currentAccount?.x_display_name ?? null}
			{mode} blockCount={threadBlockCount}
			onsubmit={handleSubmit} ontoggleinspector={toggleInspector}
			ontogglepreview={togglePreview} onopenpalette={() => {}}
		/>
		<ComposerToolbar
			{mode} {embedded} {canSubmit} {focusMode} {previewMode}
			{inspectorOpen} showFromNotes={notesPanelMode === 'notes'} {isMobile}
			{threadFlowRef} {tweetEditorRef} {attachedMedia}
			onaiassist={() => inspectorRef?.handleAiAssist()}
			onaction={handleAction}
			onmediachange={(m) => { attachedMedia = m; }}
		/>
		{@render composeBody()}
	</div>
{/if}

<ThreadPreviewRail
	{mode} {tweetText}
	tweetMediaPaths={attachedMedia.map((m) => m.path)}
	tweetLocalPreviews={tweetMediaPreviewMap}
	blocks={sortedPreviewBlocks}
	handle={$currentAccount?.x_username ? `@${$currentAccount.x_username}` : '@you'}
	avatarUrl={$currentAccount?.x_avatar_url ?? null}
	{previewMode}
	onclosepreview={() => { previewMode = false; }}
/>

<style>
	.sr-only {
		position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
		overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0;
	}

	.embedded-workspace {
		display: flex; flex-direction: column;
		flex: 1; min-height: 0; position: relative;
	}
</style>
