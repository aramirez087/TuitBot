<script lang="ts">
	import {
		api,
		type ScheduleConfig,
		type ComposeRequest,
		type ThreadBlock,
		parseThreadContent,
		isBlocksPayload
	} from '$lib/api';
	import { tweetWeightedLen } from '$lib/utils/tweetLength';
	import { X, Send, Image, Film, Plus, Maximize2, Minimize2, FileText } from 'lucide-svelte';
	import { matchEvent } from '$lib/utils/shortcuts';
	import { focusTrap } from '$lib/actions/focusTrap';
	import TimePicker from './TimePicker.svelte';
	import ThreadComposer from './ThreadComposer.svelte';
	import TweetPreview from './TweetPreview.svelte';
	import CommandPalette from './CommandPalette.svelte';
	import FromNotesPanel from './FromNotesPanel.svelte';

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

	// Media attachments
	interface AttachedMedia {
		path: string;
		file: File;
		previewUrl: string;
		mediaType: string;
	}
	let attachedMedia = $state<AttachedMedia[]>([]);
	let uploading = $state(false);
	let fileInput: HTMLInputElement | undefined = $state();

	// Focus mode
	let focusMode = $state(false);
	// Command palette
	let paletteOpen = $state(false);
	// Thread composer ref
	let threadComposerRef: ThreadComposer | undefined = $state();
	// From notes
	let showFromNotes = $state(false);
	// Focus return
	let triggerElement: Element | null = null;

	const ACCEPTED_TYPES = 'image/jpeg,image/png,image/webp,image/gif,video/mp4';
	const MAX_IMAGES = 4;
	const MAX_IMAGE_SIZE = 5 * 1024 * 1024;
	const MAX_GIF_SIZE = 15 * 1024 * 1024;
	const MAX_VIDEO_SIZE = 512 * 1024 * 1024;

	const hasGifOrVideo = $derived(
		attachedMedia.some((m) => m.mediaType === 'image/gif' || m.mediaType === 'video/mp4')
	);
	const canAttachMore = $derived(!hasGifOrVideo && attachedMedia.length < MAX_IMAGES);

	const targetDate = $derived(prefillDate ?? new Date());
	const dateLabel = $derived(
		targetDate.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })
	);

	// Auto-save
	const AUTOSAVE_KEY = 'tuitbot:compose:draft';
	const AUTOSAVE_DEBOUNCE_MS = 500;
	const AUTOSAVE_TTL_MS = 7 * 24 * 60 * 60 * 1000;
	let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
	let showRecovery = $state(false);
	let recoveryData = $state<{
		mode: string;
		tweetText: string;
		blocks: ThreadBlock[];
		timestamp: number;
	} | null>(null);

	function autoSave() {
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
		autoSaveTimer = setTimeout(() => {
			const payload = { mode, tweetText, blocks: threadBlocks, timestamp: Date.now() };
			try {
				localStorage.setItem(AUTOSAVE_KEY, JSON.stringify(payload));
			} catch {
				/* quota exceeded — silent */
			}
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
			const hasContent =
				data.tweetText?.trim() || data.blocks?.some((b: ThreadBlock) => b.text.trim());
			if (hasContent) {
				recoveryData = data;
				showRecovery = true;
			}
		} catch {
			localStorage.removeItem(AUTOSAVE_KEY);
		}
	}

	function recoverDraft() {
		if (!recoveryData) return;
		mode = (recoveryData.mode as 'tweet' | 'thread') ?? 'tweet';
		tweetText = recoveryData.tweetText || '';
		threadBlocks = recoveryData.blocks || [];
		showRecovery = false;
		clearAutoSave();
	}

	function dismissRecovery() {
		showRecovery = false;
		clearAutoSave();
	}

	// Sync state when modal opens/closes
	$effect(() => {
		if (open) {
			triggerElement = document.activeElement;
			selectedTime = prefillTime ?? null;
			checkRecovery();
			if (!showRecovery) {
				tweetText = '';
				threadBlocks = [];
				mode = 'tweet';
			}
			submitting = false;
			submitError = null;
			for (const m of attachedMedia) {
				URL.revokeObjectURL(m.previewUrl);
			}
			attachedMedia = [];
			uploading = false;
			focusMode = false;
			paletteOpen = false;
			showFromNotes = false;
		}
	});

	// Auto-save on content changes
	$effect(() => {
		// Read reactive deps
		void mode;
		void tweetText;
		void threadBlocks;
		if (open) autoSave();
	});

	const TWEET_MAX = 280;
	const tweetChars = $derived(tweetWeightedLen(tweetText));
	const tweetOverLimit = $derived(tweetChars > TWEET_MAX);
	const canSubmitTweet = $derived(tweetText.trim().length > 0 && !tweetOverLimit);
	const canSubmit = $derived(mode === 'tweet' ? canSubmitTweet : threadValid);

	const sortedPreviewBlocks = $derived(
		[...threadBlocks]
			.sort((a, b) => a.order - b.order)
			.filter((b) => b.text.trim().length > 0)
	);

	function getMaxSize(type: string): number {
		if (type === 'video/mp4') return MAX_VIDEO_SIZE;
		if (type === 'image/gif') return MAX_GIF_SIZE;
		return MAX_IMAGE_SIZE;
	}

	async function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const files = input.files;
		if (!files || files.length === 0) return;
		submitError = null;

		for (const file of files) {
			if (!canAttachMore && !hasGifOrVideo) {
				submitError = `Maximum ${MAX_IMAGES} images allowed per tweet.`;
				break;
			}
			const isGifOrVideo = file.type === 'image/gif' || file.type === 'video/mp4';
			if (isGifOrVideo && attachedMedia.length > 0) {
				submitError = 'GIF/video cannot be combined with other media.';
				break;
			}
			if (!isGifOrVideo && hasGifOrVideo) {
				submitError = 'Cannot add images when GIF/video is attached.';
				break;
			}
			const maxSize = getMaxSize(file.type);
			if (file.size > maxSize) {
				submitError = `File "${file.name}" exceeds maximum size of ${Math.round(maxSize / 1024 / 1024)}MB.`;
				break;
			}
			uploading = true;
			try {
				const result = await api.media.upload(file);
				attachedMedia = [
					...attachedMedia,
					{
						path: result.path,
						file,
						previewUrl: URL.createObjectURL(file),
						mediaType: result.media_type
					}
				];
			} catch (err) {
				submitError = err instanceof Error ? err.message : 'Failed to upload media';
				break;
			} finally {
				uploading = false;
			}
		}
		input.value = '';
	}

	function removeMedia(index: number) {
		const removed = attachedMedia[index];
		if (removed) URL.revokeObjectURL(removed.previewUrl);
		attachedMedia = attachedMedia.filter((_, i) => i !== index);
	}

	async function handleSubmit() {
		if (!canSubmit || submitting) return;
		submitting = true;
		submitError = null;

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
				if (allMedia.length > 0) {
					data.media_paths = allMedia;
				}
			}

			if (selectedTime) {
				const scheduled = new Date(targetDate);
				const [h, m] = selectedTime.split(':').map(Number);
				scheduled.setHours(h, m, 0, 0);
				data.scheduled_for = scheduled.toISOString().replace('Z', '');
			}

			if (attachedMedia.length > 0) {
				data.media_paths = attachedMedia.map((m) => m.path);
			}

			clearAutoSave();
			onsubmit(data);
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'Failed to submit';
			submitting = false;
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) handleCloseModal();
	}

	function toggleFocusMode() {
		focusMode = !focusMode;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (paletteOpen) return;

		if (matchEvent(e, 'cmd+k')) {
			e.preventDefault();
			paletteOpen = true;
			return;
		}
		if (matchEvent(e, 'cmd+shift+f')) {
			e.preventDefault();
			toggleFocusMode();
			return;
		}
		if (matchEvent(e, 'cmd+enter')) {
			e.preventDefault();
			handleSubmit();
			return;
		}
		if (matchEvent(e, 'cmd+j')) {
			e.preventDefault();
			handleInlineAssist();
			return;
		}
		if (matchEvent(e, 'cmd+shift+n')) {
			e.preventDefault();
			mode = 'tweet';
			return;
		}
		if (matchEvent(e, 'cmd+shift+t')) {
			e.preventDefault();
			mode = 'thread';
			return;
		}
		if (e.key === 'Escape') {
			if (showFromNotes) {
				showFromNotes = false;
			} else if (focusMode) {
				focusMode = false;
			} else {
				handleCloseModal();
			}
			return;
		}
	}

	function handlePaletteAction(actionId: string) {
		paletteOpen = false;
		switch (actionId) {
			case 'focus-mode':
				toggleFocusMode();
				break;
			case 'mode-tweet':
				mode = 'tweet';
				break;
			case 'mode-thread':
				mode = 'thread';
				break;
			case 'submit':
				handleSubmit();
				break;
			case 'ai-improve':
				handleInlineAssist();
				break;
			case 'ai-from-notes':
				showFromNotes = true;
				break;
			case 'attach-media':
				fileInput?.click();
				break;
			case 'add-card':
			case 'duplicate':
			case 'split':
			case 'merge':
			case 'move-up':
			case 'move-down':
				threadComposerRef?.handlePaletteAction(actionId);
				break;
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

			assisting = true;
			submitError = null;
			try {
				const result = await api.assist.improve(selectedText);
				if (start !== end) {
					tweetText = tweetText.slice(0, start) + result.content + tweetText.slice(end);
				} else {
					tweetText = result.content;
				}
			} catch (e) {
				submitError = e instanceof Error ? e.message : 'AI assist failed';
			} finally {
				assisting = false;
			}
		} else {
			threadComposerRef?.handleInlineAssist();
		}
	}

	async function handleGenerateFromNotes(notesInput: string) {
		const hasContent =
			mode === 'tweet'
				? tweetText.trim().length > 0
				: threadBlocks.some((b) => b.text.trim().length > 0);

		if (hasContent && !confirm('This will replace your current content. Continue?')) {
			return;
		}

		submitError = null;
		if (mode === 'thread') {
			const result = await api.assist.thread(notesInput);
			threadBlocks = result.tweets.map((text, i) => ({
				id: crypto.randomUUID(),
				text,
				media_paths: [],
				order: i
			}));
		} else {
			const result = await api.assist.improve(
				notesInput,
				'Expand these rough notes into a polished tweet'
			);
			tweetText = result.content;
		}
		showFromNotes = false;
	}

	function handleCloseModal() {
		onclose();
		if (triggerElement instanceof HTMLElement) {
			triggerElement.focus();
		}
	}

	// AI Assist
	let assisting = $state(false);

	async function handleAiAssist() {
		assisting = true;
		submitError = null;
		try {
			if (mode === 'tweet') {
				if (tweetText.trim()) {
					const result = await api.assist.improve(tweetText);
					tweetText = result.content;
				} else {
					const result = await api.assist.tweet('general');
					tweetText = result.content;
				}
			} else {
				const result = await api.assist.thread('general');
				threadBlocks = result.tweets.map((text, i) => ({
					id: crypto.randomUUID(),
					text,
					media_paths: [],
					order: i
				}));
			}
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'AI assist failed';
		} finally {
			assisting = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
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
						<button class="recovery-btn" onclick={recoverDraft}>Recover</button>
						<button class="recovery-dismiss" onclick={dismissRecovery}>Discard</button>
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
						onclick={toggleFocusMode}
						aria-label={focusMode ? 'Exit focus mode' : 'Enter focus mode'}
						title={focusMode ? 'Exit focus mode (⌘⇧F)' : 'Focus mode (⌘⇧F)'}
					>
						{#if focusMode}
							<Minimize2 size={14} />
						{:else}
							<Maximize2 size={14} />
						{/if}
					</button>
					<button class="close-btn" onclick={handleCloseModal} aria-label="Close compose modal">
						<X size={16} />
					</button>
				</div>
			</div>

			<div class="mode-tabs" role="tablist" aria-label="Content type">
				<button
					class="tab"
					class:active={mode === 'tweet'}
					onclick={() => (mode = 'tweet')}
					role="tab"
					aria-selected={mode === 'tweet'}
				>
					Tweet
				</button>
				<button
					class="tab"
					class:active={mode === 'thread'}
					onclick={() => (mode = 'thread')}
					role="tab"
					aria-selected={mode === 'thread'}
				>
					Thread
				</button>
			</div>

			<div class="modal-body">
				{#if mode === 'tweet'}
					<div class="tweet-compose">
						<textarea
							class="compose-input"
							class:over-limit={tweetOverLimit}
							placeholder="What's on your mind?"
							bind:value={tweetText}
							rows={4}
							aria-label="Tweet content"
						></textarea>
						<div
							class="char-counter"
							class:over-limit={tweetOverLimit}
							aria-live="polite"
							aria-label="Character count"
						>
							{tweetChars}/{TWEET_MAX}
						</div>
					</div>
				{:else}
					<div class="thread-layout">
						<div class="thread-editor-pane">
							<ThreadComposer
								bind:this={threadComposerRef}
								initialBlocks={threadBlocks.length > 0 ? threadBlocks : undefined}
								onchange={(b) => {
									threadBlocks = b;
								}}
								onvalidchange={(v) => {
									threadValid = v;
								}}
							/>
						</div>
						<div class="thread-preview-pane">
							<div class="preview-header-label">Preview</div>
							<div class="preview-scroll">
								{#if sortedPreviewBlocks.length > 0}
									{#each sortedPreviewBlocks as block, i (block.id)}
										<TweetPreview
											text={block.text}
											mediaPaths={block.media_paths}
											index={i}
											total={sortedPreviewBlocks.length}
										/>
									{/each}
								{:else}
									<div class="preview-empty">Start typing to see preview...</div>
								{/if}
							</div>
						</div>
					</div>
				{/if}

				<!-- Media attachments -->
				{#if attachedMedia.length > 0}
					<div class="media-preview-grid">
						{#each attachedMedia as media, i}
							<div class="media-thumb">
								{#if media.mediaType === 'video/mp4'}
									<video src={media.previewUrl} class="thumb-img" muted></video>
									<span class="media-badge"><Film size={10} /> Video</span>
								{:else}
									<img src={media.previewUrl} alt="Attached media" class="thumb-img" />
									{#if media.mediaType === 'image/gif'}
										<span class="media-badge">GIF</span>
									{/if}
								{/if}
								<button class="remove-media-btn" onclick={() => removeMedia(i)} aria-label="Remove media">
									<X size={12} />
								</button>
							</div>
						{/each}
					</div>
				{/if}

				{#if mode === 'tweet' && canAttachMore}
					<div class="media-attach-section">
						<button class="attach-btn" onclick={() => fileInput?.click()} disabled={uploading}>
							<Image size={14} />
							{uploading ? 'Uploading...' : 'Attach media'}
						</button>
						<span class="attach-hint">
							JPEG, PNG, WebP, GIF, MP4 &middot; max 4 images or 1 GIF/video
						</span>
						<input
							bind:this={fileInput}
							type="file"
							accept={ACCEPTED_TYPES}
							multiple
							class="hidden-file-input"
							onchange={handleFileSelect}
						/>
					</div>
				{/if}

				{#if showFromNotes}
					<FromNotesPanel {mode} ongenerate={handleGenerateFromNotes} onclose={() => { showFromNotes = false; }} />
				{/if}

				<div class="schedule-section">
					<TimePicker
						{schedule}
						{selectedTime}
						targetDate={targetDate}
						onselect={(time) => (selectedTime = time || null)}
					/>
				</div>

				{#if submitError}
					<div class="error-msg" role="alert">{submitError}</div>
				{/if}
			</div>

			{#if paletteOpen}
				<CommandPalette
					open={paletteOpen}
					{mode}
					onclose={() => { paletteOpen = false; }}
					onaction={handlePaletteAction}
				/>
			{/if}

			<div class="modal-footer">
				<button class="assist-btn" onclick={handleAiAssist} disabled={assisting}>
					{assisting ? 'Generating...' : tweetText.trim() && mode === 'tweet' ? 'AI Improve' : 'AI Assist'}
				</button>
				<button
					class="notes-btn"
					onclick={() => { showFromNotes = !showFromNotes; }}
					title="Generate from notes"
					aria-label="Generate from notes"
				>
					<FileText size={14} />
				</button>
				<div class="footer-spacer"></div>
				<button class="cancel-btn" onclick={handleCloseModal}>Cancel</button>
				<button class="submit-btn" onclick={handleSubmit} disabled={!canSubmit || submitting}>
					<Send size={14} />
					{submitting ? 'Submitting...' : selectedTime ? 'Schedule' : 'Post now'}
				</button>
			</div>
		</div>
	</div>
{/if}

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

	/* Tweet mode */
	.compose-input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-base);
		color: var(--color-text);
		font-size: 14px;
		font-family: var(--font-sans);
		line-height: 1.5;
		resize: vertical;
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.compose-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.compose-input.over-limit {
		border-color: var(--color-danger);
	}

	.char-counter {
		text-align: right;
		font-size: 11px;
		color: var(--color-text-subtle);
		margin-top: 4px;
		font-family: var(--font-mono);
	}

	.char-counter.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	/* Thread two-pane layout */
	.thread-layout {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}

	.thread-editor-pane {
		min-width: 0;
	}

	.thread-preview-pane {
		min-width: 0;
		border-left: 1px solid var(--color-border-subtle);
		padding-left: 16px;
	}

	.preview-header-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		margin-bottom: 8px;
	}

	.preview-scroll {
		max-height: 400px;
		overflow-y: auto;
	}

	.preview-empty {
		padding: 24px 0;
		text-align: center;
		font-size: 13px;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	@media (max-width: 768px) {
		.modal.thread-mode {
			width: 100%;
			max-width: 100vw;
			border-radius: 0;
			max-height: 100vh;
		}

		.thread-layout {
			grid-template-columns: 1fr;
		}

		.thread-preview-pane {
			border-left: none;
			padding-left: 0;
			border-top: 1px solid var(--color-border-subtle);
			padding-top: 16px;
		}
	}

	/* Media */
	.media-preview-grid {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-top: 12px;
	}

	.media-thumb {
		position: relative;
		width: 80px;
		height: 80px;
		border-radius: 8px;
		overflow: hidden;
		border: 1px solid var(--color-border);
	}

	.thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.media-badge {
		position: absolute;
		bottom: 4px;
		left: 4px;
		display: flex;
		align-items: center;
		gap: 3px;
		font-size: 9px;
		font-weight: 600;
		padding: 1px 5px;
		border-radius: 3px;
		background: rgba(0, 0, 0, 0.7);
		color: #fff;
	}

	.remove-media-btn {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 50%;
		background: rgba(0, 0, 0, 0.6);
		color: #fff;
		cursor: pointer;
		transition: background 0.15s ease;
	}

	.remove-media-btn:hover {
		background: rgba(0, 0, 0, 0.85);
	}

	.media-attach-section {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 12px;
	}

	.attach-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.attach-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.attach-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.attach-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.hidden-file-input {
		display: none;
	}

	/* Schedule & footer */
	.schedule-section {
		margin-top: 16px;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
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

	.modal.focus-mode .thread-layout {
		height: 100%;
	}

	.modal.focus-mode .preview-scroll {
		max-height: none;
		flex: 1;
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

		.remove-media-btn {
			width: 32px;
			height: 32px;
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

		.media-attach-section {
			flex-direction: column;
			align-items: flex-start;
		}

		.compose-input {
			font-size: 16px;
		}
	}
</style>
