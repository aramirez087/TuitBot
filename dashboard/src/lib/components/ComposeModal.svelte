<script lang="ts">
	import {
		api,
		type ScheduleConfig,
		type ComposeRequest,
		type ThreadBlock
	} from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import { matchEvent } from '$lib/utils/shortcuts';
	import TimePicker from './TimePicker.svelte';
	import ThreadComposer from './ThreadComposer.svelte';
	import CommandPalette from './CommandPalette.svelte';
	import FromNotesPanel from './FromNotesPanel.svelte';
	import ComposerShell from './composer/ComposerShell.svelte';
	import ComposerHeaderBar from './composer/ComposerHeaderBar.svelte';
	import ComposerCanvas from './composer/ComposerCanvas.svelte';
	import ComposerInspector from './composer/ComposerInspector.svelte';
	import TweetEditor from './composer/TweetEditor.svelte';
	import VoiceContextPanel from './composer/VoiceContextPanel.svelte';
	import ThreadPreviewRail from './composer/ThreadPreviewRail.svelte';
	import type { AttachedMedia } from './composer/TweetEditor.svelte';

	let {
		open,
		prefillTime = null,
		prefillDate = null,
		schedule,
		onclose,
		onsubmit
	}: {
		open: boolean;
		prefillTime?: string | null;
		prefillDate?: Date | null;
		schedule: ScheduleConfig | null;
		onclose: () => void;
		onsubmit: (data: ComposeRequest) => void;
	} = $props();

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
	let threadComposerRef: ThreadComposer | undefined = $state();
	let tweetEditorRef: TweetEditor | undefined = $state();
	let voicePanelRef: VoiceContextPanel | undefined = $state();
	let showFromNotes = $state(false);
	let triggerElement: Element | null = null;
	let assisting = $state(false);
	let voiceCue = $state('');
	let previewCollapsed = $state(false);
	let inspectorOpen = $state(loadInspectorState());
	let isMobile = $state(false);

	// Undo state for notes generation
	let undoSnapshot = $state<{ mode: 'tweet' | 'thread'; text: string; blocks: ThreadBlock[] } | null>(null);
	let showUndo = $state(false);
	let undoTimer: ReturnType<typeof setTimeout> | null = null;

	const targetDate = $derived(prefillDate ?? new Date());

	// Auto-save
	const AUTOSAVE_KEY = 'tuitbot:compose:draft';
	const AUTOSAVE_DEBOUNCE_MS = 500;
	const AUTOSAVE_TTL_MS = 7 * 24 * 60 * 60 * 1000;
	let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
	let showRecovery = $state(false);
	let recoveryData = $state<{
		mode: string; tweetText: string; blocks: ThreadBlock[]; timestamp: number;
	} | null>(null);

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
		previewCollapsed = !previewCollapsed;
	}

	// Responsive detection
	$effect(() => {
		const mql = window.matchMedia('(max-width: 768px)');
		isMobile = mql.matches;
		const handler = (e: MediaQueryListEvent) => { isMobile = e.matches; };
		mql.addEventListener('change', handler);
		return () => mql.removeEventListener('change', handler);
	});

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

	$effect(() => {
		if (open) {
			triggerElement = document.activeElement;
			selectedTime = prefillTime ?? null;
			checkRecovery();
			if (!showRecovery) { tweetText = ''; threadBlocks = []; mode = 'tweet'; }
			submitting = false; submitError = null;
			for (const m of attachedMedia) URL.revokeObjectURL(m.previewUrl);
			attachedMedia = [];
			focusMode = false; paletteOpen = false; showFromNotes = false;
			voiceCue = '';
			undoSnapshot = null; showUndo = false;
			previewCollapsed = false;
			inspectorOpen = loadInspectorState();
			if (undoTimer) clearTimeout(undoTimer);
		}
	});

	$effect(() => { void mode; void tweetText; void threadBlocks; if (open) autoSave(); });

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

	async function handleSubmit() {
		if (!canSubmit || submitting) return;
		submitting = true; submitError = null;
		try {
			const data: ComposeRequest = { content_type: mode, content: '' };
			if (mode === 'tweet') {
				data.content = tweetText.trim();
			} else {
				const validBlocks = threadBlocks
					.filter((b) => b.text.trim().length > 0)
					.sort((a, b) => a.order - b.order)
					.map((b, i) => ({ ...b, text: b.text.trim(), order: i }));
				data.content = JSON.stringify(validBlocks.map((b) => b.text));
				data.blocks = validBlocks;
				const allMedia = validBlocks.flatMap((b) => b.media_paths);
				if (allMedia.length > 0) data.media_paths = allMedia;
			}
			if (selectedTime) {
				const scheduled = new Date(targetDate);
				const [h, m] = selectedTime.split(':').map(Number);
				scheduled.setHours(h, m, 0, 0);
				data.scheduled_for = scheduled.toISOString().replace('Z', '');
			}
			if (attachedMedia.length > 0) data.media_paths = attachedMedia.map((m) => m.path);
			clearAutoSave();
			onsubmit(data);
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'Failed to submit';
			submitting = false;
		}
	}

	function handleCloseModal() {
		onclose();
		if (triggerElement instanceof HTMLElement) triggerElement.focus();
	}

	function toggleFocusMode() { focusMode = !focusMode; }

	function handleKeydown(e: KeyboardEvent) {
		if (!open || paletteOpen) return;
		if (matchEvent(e, 'cmd+k')) { e.preventDefault(); paletteOpen = true; return; }
		if (matchEvent(e, 'cmd+shift+f')) { e.preventDefault(); toggleFocusMode(); return; }
		if (matchEvent(e, 'cmd+enter')) { e.preventDefault(); handleSubmit(); return; }
		if (matchEvent(e, 'cmd+j')) { e.preventDefault(); handleInlineAssist(); return; }
		if (matchEvent(e, 'cmd+shift+n')) { e.preventDefault(); mode = 'tweet'; return; }
		if (matchEvent(e, 'cmd+shift+t')) { e.preventDefault(); mode = 'thread'; return; }
		if (matchEvent(e, 'cmd+i')) { e.preventDefault(); toggleInspector(); return; }
		if (matchEvent(e, 'cmd+shift+p')) { e.preventDefault(); togglePreview(); return; }
		if (e.key === 'Escape') {
			if (showFromNotes) showFromNotes = false;
			else if (isMobile && inspectorOpen) inspectorOpen = false;
			else if (focusMode) focusMode = false;
			else handleCloseModal();
			return;
		}
	}

	function handlePaletteAction(actionId: string) {
		paletteOpen = false;
		switch (actionId) {
			case 'focus-mode': toggleFocusMode(); break;
			case 'mode-tweet': mode = 'tweet'; break;
			case 'mode-thread': mode = 'thread'; break;
			case 'submit': handleSubmit(); break;
			case 'ai-improve': handleInlineAssist(); break;
			case 'ai-from-notes': showFromNotes = true; if (!inspectorOpen) inspectorOpen = true; break;
			case 'ai-generate': handleAiAssist(); break;
			case 'toggle-inspector': toggleInspector(); break;
			case 'attach-media': tweetEditorRef?.triggerFileSelect(); break;
			case 'add-card': case 'duplicate': case 'split': case 'merge':
			case 'move-up': case 'move-down':
				threadComposerRef?.handlePaletteAction(actionId); break;
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
			assisting = true; submitError = null;
			try {
				const result = await api.assist.improve(selectedText, voiceCue || undefined);
				if (start !== end) {
					tweetText = tweetText.slice(0, start) + result.content + tweetText.slice(end);
				} else {
					tweetText = result.content;
				}
				voicePanelRef?.saveCueToHistory();
			} catch (e) {
				submitError = e instanceof Error ? e.message : 'AI assist failed';
			} finally { assisting = false; }
		} else {
			threadComposerRef?.handleInlineAssist(voiceCue || undefined);
		}
	}

	async function handleGenerateFromNotes(notesInput: string) {
		submitError = null;
		undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };

		if (mode === 'thread') {
			const topicWithCue = voiceCue ? `[Tone: ${voiceCue}] ${notesInput}` : notesInput;
			const result = await api.assist.thread(topicWithCue);
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
		tweetText = undoSnapshot.text;
		threadBlocks = undoSnapshot.blocks;
		undoSnapshot = null;
		showUndo = false;
		if (undoTimer) clearTimeout(undoTimer);
	}

	async function handleAiAssist() {
		assisting = true; submitError = null;
		try {
			if (mode === 'tweet') {
				if (tweetText.trim()) {
					const result = await api.assist.improve(tweetText, voiceCue || undefined);
					tweetText = result.content;
				} else {
					const topicWithCue = voiceCue ? `[Tone: ${voiceCue}] general` : 'general';
					const result = await api.assist.tweet(topicWithCue);
					tweetText = result.content;
				}
			} else {
				const topicWithCue = voiceCue ? `[Tone: ${voiceCue}] general` : 'general';
				const result = await api.assist.thread(topicWithCue);
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

{#if open}
	<ComposerShell
		{open} {focusMode} {showRecovery}
		inspectorOpen={desktopInspectorOpen}
		onclose={handleCloseModal}
		onrecover={recoverDraft}
		ondismissrecovery={dismissRecovery}
	>
		{#snippet children()}
			<ComposerHeaderBar
				{focusMode}
				inspectorOpen={inspectorOpen}
				previewVisible={hasPreviewContent && !previewCollapsed}
				ontogglefocus={toggleFocusMode}
				ontoggleinspector={toggleInspector}
				ontogglepreview={togglePreview}
				onclose={handleCloseModal}
			/>

			<ComposerCanvas
				{canSubmit} {submitting} {selectedTime} {submitError}
				inspectorOpen={desktopInspectorOpen}
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
						<ThreadComposer
							bind:this={threadComposerRef}
							initialBlocks={threadBlocks.length > 0 ? threadBlocks : undefined}
							onchange={(b) => { threadBlocks = b; }}
							onvalidchange={(v) => { threadValid = v; }}
						/>
					{/if}

					{#if hasPreviewContent && !previewCollapsed}
						<div class="preview-section">
							<button class="preview-toggle" onclick={() => { previewCollapsed = true; }}>
								<span class="preview-label">Preview</span>
								<span class="preview-collapse">Hide</span>
							</button>
							<ThreadPreviewRail
								{mode}
								tweetText={tweetText}
								tweetMediaPaths={attachedMedia.map((m) => m.path)}
								tweetLocalPreviews={tweetMediaPreviewMap}
								blocks={sortedPreviewBlocks}
							/>
						</div>
					{/if}

					{#if showUndo && !showFromNotes}
						<div class="undo-banner">
							<span>Content replaced from notes.</span>
							<button class="undo-btn" onclick={handleUndo}>Undo</button>
						</div>
					{/if}
				{/snippet}

				{#snippet inspector()}
					<div class="inspector-section">
						<div class="inspector-section-label">Schedule</div>
						<TimePicker
							{schedule} {selectedTime} targetDate={targetDate}
							onselect={(time) => (selectedTime = time || null)}
						/>
						{#if !selectedTime}
							<p class="inspector-hint">Posts immediately unless scheduled</p>
						{/if}
					</div>

					<div class="inspector-section">
						<div class="inspector-section-label">Voice</div>
						<VoiceContextPanel
							bind:this={voicePanelRef}
							cue={voiceCue}
							oncuechange={(c) => { voiceCue = c; }}
							inline={true}
						/>
					</div>

					<div class="inspector-section">
						<div class="inspector-section-label">AI</div>
						<div class="ai-actions-row">
							<button class="ai-action-btn" onclick={handleAiAssist} disabled={assisting}>
								{assisting ? 'Working...' : hasExistingContent ? 'AI Improve' : 'AI Generate'}
							</button>
							<button class="ai-action-btn secondary" onclick={() => { showFromNotes = true; }}>
								From Notes
							</button>
						</div>
						<p class="inspector-hint">{'\u2318'}J to improve selected text</p>
					</div>

					{#if showFromNotes}
						<div class="inspector-section">
							<FromNotesPanel
								{mode}
								{hasExistingContent}
								compact={true}
								ongenerate={handleGenerateFromNotes}
								onclose={() => { showFromNotes = false; }}
								onundo={handleUndo}
								{showUndo}
							/>
						</div>
					{/if}
				{/snippet}
			</ComposerCanvas>
		{/snippet}
	</ComposerShell>

	{#if isMobile && inspectorOpen}
		<ComposerInspector
			open={inspectorOpen}
			mobile={true}
			onclose={() => { inspectorOpen = false; }}
		>
			{#snippet children()}
				<div class="inspector-section">
					<div class="inspector-section-label">Schedule</div>
					<TimePicker
						{schedule} {selectedTime} targetDate={targetDate}
						onselect={(time) => (selectedTime = time || null)}
					/>
					{#if !selectedTime}
						<p class="inspector-hint">Posts immediately unless scheduled</p>
					{/if}
				</div>

				<div class="inspector-section">
					<div class="inspector-section-label">Voice</div>
					<VoiceContextPanel
						bind:this={voicePanelRef}
						cue={voiceCue}
						oncuechange={(c) => { voiceCue = c; }}
						inline={true}
					/>
				</div>

				<div class="inspector-section">
					<div class="inspector-section-label">AI</div>
					<div class="ai-actions-row">
						<button class="ai-action-btn" onclick={handleAiAssist} disabled={assisting}>
							{assisting ? 'Working...' : hasExistingContent ? 'AI Improve' : 'AI Generate'}
						</button>
						<button class="ai-action-btn secondary" onclick={() => { showFromNotes = true; }}>
							From Notes
						</button>
					</div>
					<p class="inspector-hint">{'\u2318'}J to improve selected text</p>
				</div>

				{#if showFromNotes}
					<div class="inspector-section">
						<FromNotesPanel
							{mode}
							{hasExistingContent}
							compact={true}
							ongenerate={handleGenerateFromNotes}
							onclose={() => { showFromNotes = false; }}
							onundo={handleUndo}
							{showUndo}
						/>
					</div>
				{/if}
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
{/if}

<style>
	.preview-section {
		margin-top: 16px;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.preview-toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 0 0 12px;
		border: none;
		background: transparent;
		cursor: pointer;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.preview-toggle:hover {
		color: var(--color-text);
	}

	.preview-collapse {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.preview-toggle:hover .preview-collapse {
		color: var(--color-accent);
	}

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

	/* Inspector sections */
	.inspector-section {
		padding: 12px 0;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.inspector-section:last-child {
		border-bottom: none;
	}

	.inspector-section-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-muted);
		margin-bottom: 8px;
	}

	.inspector-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
		margin: 6px 0 0;
	}

	.ai-actions-row {
		display: flex;
		gap: 6px;
	}

	.ai-action-btn {
		flex: 1;
		padding: 6px 10px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
		white-space: nowrap;
	}

	.ai-action-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
		border-color: var(--color-accent-hover);
	}

	.ai-action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.ai-action-btn.secondary {
		background: transparent;
		color: var(--color-accent);
	}

	.ai-action-btn.secondary:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	@media (pointer: coarse) {
		.ai-action-btn {
			min-height: 44px;
		}
	}
</style>
