<script lang="ts">
	import { onMount, onDestroy, flushSync } from "svelte";
	import type { Snippet } from "svelte";
	import {
		api,
		type ScheduleConfig,
		type ComposeRequest,
		type ThreadBlock,
	} from "$lib/api";
	import type { VaultCitation } from "$lib/api/types";
	import { tweetWeightedLen } from "$lib/utils/tweetLength";
	import { matchEvent } from "$lib/utils/shortcuts";
	import {
		buildComposeRequest,
		topicWithCue,
	} from "$lib/utils/composeHandlers";
	import { buildScheduledFor, nowInAccountTz } from "$lib/utils/timezone";
	import { trackFunnel } from "$lib/analytics/funnel";
	import ThreadFlowLane from "./ThreadFlowLane.svelte";
	import CommandPalette from "../CommandPalette.svelte";
	import ComposerShell from "./ComposerShell.svelte";
	import ComposerHeaderBar from "./ComposerHeaderBar.svelte";
	import HomeComposerHeader from "./HomeComposerHeader.svelte";
	import ComposerCanvas from "./ComposerCanvas.svelte";
	import ComposerInspector from "./ComposerInspector.svelte";
	import InspectorContent from "./InspectorContent.svelte";
	import RecoveryBanner from "./RecoveryBanner.svelte";
	import TweetEditor from "./TweetEditor.svelte";
	import AddTweetDivider from "./AddTweetDivider.svelte";
	import ComposerPreviewSurface from "./ComposerPreviewSurface.svelte";
	import VoiceContextPanel from "./VoiceContextPanel.svelte";
	import ComposerToolbar from "./ComposerToolbar.svelte";
	import ComposerInsertBar from "./ComposerInsertBar.svelte";
	import CitationChips from "./CitationChips.svelte";
	import { currentAccount } from "$lib/stores/accounts";
	import { deploymentMode } from "$lib/stores/runtime";
	import { persistGet, persistSet } from "$lib/stores/persistence";
	import {
		saveAutoSave,
		clearAutoSave as clearAutoSaveStorage,
		readAutoSave,
		restoreMedia,
		wasNavigationExit,
		markSessionActive,
		clearSessionFlag,
		AUTOSAVE_DEBOUNCE_MS,
		DraftSaveManager,
		readDraftAutoSave,
		clearDraftAutoSave,
	} from "$lib/utils/composerAutosave";
	import type { AttachedMedia } from "./TweetEditor.svelte";

	let {
		schedule,
		onsubmit,
		onclose,
		prefillTime = null,
		prefillDate = null,
		embedded = false,
		canPublish = true,
		draftId = undefined,
		initialContent = undefined,
		onsyncstatus = undefined,
		extraPaletteActions = [],
		ondraftaction = undefined,
		headerLeft = undefined,
	}: {
		schedule: ScheduleConfig | null;
		onsubmit: (data: ComposeRequest) => void | Promise<void>;
		onclose?: () => void;
		prefillTime?: string | null;
		prefillDate?: Date | null;
		embedded?: boolean;
		canPublish?: boolean;
		draftId?: number;
		initialContent?: {
			mode: "tweet" | "thread";
			tweetText: string;
			threadBlocks: ThreadBlock[];
			attachedMedia: AttachedMedia[];
			updatedAt: string;
		};
		onsyncstatus?: (
			status: import("$lib/utils/composerAutosave").SyncStatus,
		) => void;
		extraPaletteActions?: import("../CommandPalette.svelte").PaletteAction[];
		ondraftaction?: (actionId: string) => void;
		headerLeft?: Snippet;
	} = $props();

	// ── State ──────────────────────────────────────────────
	let mode = $state<"tweet" | "thread">("tweet");
	let tweetText = $state("");
	let threadBlocks = $state<ThreadBlock[]>([]);
	let threadValid = $state(false);
	let selectedTime = $state<string | null>(null);
	let scheduledDate = $state<string | null>(null);
	let submitting = $state(false);
	let submitError = $state<string | null>(null);
	let attachedMedia = $state<AttachedMedia[]>([]);
	let focusMode = $state(false);
	let paletteOpen = $state(false);
	let threadFlowRef: ThreadFlowLane | undefined = $state();
	let tweetEditorRef: TweetEditor | undefined = $state();
	let voicePanelRef: VoiceContextPanel | undefined = $state();
	let notesPanelMode = $state<"notes" | "vault" | null>(null);
	let vaultCitations = $state<VaultCitation[]>([]);
	let assisting = $state(false);
	let voiceCue = $state("");
	let previewMode = $state(false);
	let inspectorOpen = $state(loadInspectorState());
	let isMobile = $state(false);
	let statusAnnouncement = $state("");

	// Desktop vault path for Obsidian deep-links
	let localVaultPath = $state<string | null>(null);
	const isDesktop = $derived($deploymentMode === "desktop");

	// Home-surface state (only active when embedded)
	let tipsVisible = $state(false);
	let promptDismissed = $state(false);

	// Undo state for notes generation
	let undoSnapshot = $state<{
		mode: "tweet" | "thread";
		text: string;
		blocks: ThreadBlock[];
		media?: AttachedMedia[];
		selectedTime?: string | null;
		scheduledDate?: string | null;
		citations?: VaultCitation[];
	} | null>(null);
	let showUndo = $state(false);
	let undoMessage = $state("Content replaced.");
	let undoTimer: ReturnType<typeof setTimeout> | null = null;

	// Auto-save (logic extracted to composerAutosave.ts)
	let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
	let draftSaveManager: DraftSaveManager | null = null;
	let initialized = $state(false);
	let showRecovery = $state(false);
	let recoveryData = $state<{
		mode: string;
		tweetText: string;
		blocks: ThreadBlock[];
		timestamp: number;
		tweetMedia?: Array<{
			path: string;
			mediaType: string;
			altText?: string;
		}>;
	} | null>(null);

	// Toolbar state

	// ── Derived ────────────────────────────────────────────
	const targetDate = $derived(prefillDate ?? new Date());
	const TWEET_MAX = 280;
	const tweetChars = $derived(tweetWeightedLen(tweetText));
	const tweetOverLimit = $derived(tweetChars > TWEET_MAX);
	const canSubmitTweet = $derived(
		tweetText.trim().length > 0 && !tweetOverLimit,
	);
	const canSubmit = $derived(mode === "tweet" ? canSubmitTweet : threadValid);

	const sortedPreviewBlocks = $derived(
		[...threadBlocks]
			.sort((a, b) => a.order - b.order)
			.filter((b) => b.text.trim().length > 0),
	);

	const hasExistingContent = $derived(
		mode === "tweet"
			? tweetText.trim().length > 0
			: threadBlocks.some((b) => b.text.trim().length > 0),
	);

	const tweetMediaPreviewMap = $derived(
		new Map(attachedMedia.map((m) => [m.path, m.previewUrl])),
	);

	const hasPreviewContent = $derived(
		mode === "thread"
			? sortedPreviewBlocks.length > 0
			: tweetText.trim().length > 0,
	);

	const accountTimezone = $derived(schedule?.timezone ?? "UTC");

	const scheduledFor = $derived(() => {
		if (!selectedTime || !scheduledDate) return null;
		try {
			return buildScheduledFor(scheduledDate, selectedTime, accountTimezone);
		} catch {
			return null;
		}
	});

	const desktopInspectorOpen = $derived(inspectorOpen && !isMobile);

	const threadBlockCount = $derived(
		mode === "thread"
			? threadBlocks.filter((b) => b.text.trim().length > 0).length ||
					threadBlocks.length
			: 1,
	);

	// ── Inspector persistence ──────────────────────────────
	function loadInspectorState(): boolean {
		try {
			const saved = localStorage.getItem("tuitbot:inspector:open");
			return saved === null ? true : saved === "true";
		} catch {
			return true;
		}
	}

	function persistInspectorState(v: boolean) {
		try {
			localStorage.setItem("tuitbot:inspector:open", String(v));
		} catch {
			/* quota */
		}
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
		const mql = window.matchMedia("(max-width: 768px)");
		isMobile = mql.matches;
		const handler = (e: MediaQueryListEvent) => {
			isMobile = e.matches;
		};
		mql.addEventListener("change", handler);
		return () => mql.removeEventListener("change", handler);
	});

	$effect(() => {
		void mode;
		void tweetText;
		void threadBlocks;
		void attachedMedia;
		if (initialized) autoSave();
	});

	// Auto-collapse: thread → tweet when only 1 block remains
	$effect(() => {
		if (mode === "thread" && threadBlocks.length <= 1 && initialized) {
			const surviving = threadBlocks[0];
			tweetText = surviving?.text ?? "";
			threadBlocks = [];
			mode = "tweet";
			requestAnimationFrame(() => {
				const textarea = document.querySelector(
					".compose-input",
				) as HTMLTextAreaElement | null;
				textarea?.focus();
			});
		}
	});

	// Announce mode switches to screen readers (skip initial render)
	let modeInitialized = false;
	$effect(() => {
		if (!modeInitialized) {
			modeInitialized = true;
			return;
		}
		statusAnnouncement =
			mode === "tweet"
				? "Switched to tweet mode"
				: "Switched to thread mode";
	});

	function flushAutoSave() {
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
		autoSaveTimer = null;
		if (!initialized) return;
		saveAutoSave(mode, tweetText, threadBlocks, attachedMedia);
	}

	function handleBeforeUnload() {
		if (draftSaveManager) {
			draftSaveManager.flush();
		} else {
			flushAutoSave();
			markSessionActive();
		}
	}

	onMount(async () => {
		selectedTime = prefillTime ?? null;
		if (prefillDate) {
			const y = prefillDate.getFullYear();
			const mo = String(prefillDate.getMonth() + 1).padStart(2, "0");
			const d = String(prefillDate.getDate()).padStart(2, "0");
			scheduledDate = `${y}-${mo}-${d}`;
		}

		if (draftId !== undefined && initialContent) {
			// Draft Studio mode: hydrate from server data
			mode = initialContent.mode;
			tweetText = initialContent.tweetText;
			threadBlocks = initialContent.threadBlocks;
			attachedMedia = initialContent.attachedMedia;

			// Check for crash recovery data
			const localData = readDraftAutoSave(draftId);
			if (
				localData &&
				localData.timestamp >
					new Date(initialContent.updatedAt).getTime()
			) {
				recoveryData = localData;
				showRecovery = true;
			} else {
				clearDraftAutoSave(draftId);
			}

			// Create save manager
			const syncCallback = onsyncstatus ?? (() => {});
			draftSaveManager = new DraftSaveManager(
				draftId,
				initialContent.updatedAt,
				syncCallback,
			);
			initialized = true;
		} else {
			checkRecovery();
			if (!showRecovery && !initialized) {
				tweetText = "";
				threadBlocks = [];
				mode = "tweet";
				initialized = true;
			}
		}

		submitting = false;
		submitError = null;
		focusMode = false;
		paletteOpen = false;
		notesPanelMode = null;
		vaultCitations = [];
		voiceCue = "";
		undoSnapshot = null;
		showUndo = false;
		previewMode = false;
		inspectorOpen = loadInspectorState();

		window.addEventListener("beforeunload", handleBeforeUnload);

		// Load vault path for Obsidian deep-links (desktop only)
		if ($deploymentMode === "desktop") {
			try {
				const res = await api.vault.sources();
				const localSrc = res.sources.find(
					(s) => s.source_type === "local_fs",
				);
				if (localSrc?.path) localVaultPath = localSrc.path;
			} catch {
				/* vault path is best-effort */
			}
		}

		if (embedded) {
			const tipsDismissed = await persistGet(
				"home_tips_dismissed",
				false,
			);
			tipsVisible = !tipsDismissed;
			window.addEventListener("tuitbot:compose", handleComposeEvent);
		}
	});

	onDestroy(() => {
		window.removeEventListener("beforeunload", handleBeforeUnload);
		if (embedded)
			window.removeEventListener("tuitbot:compose", handleComposeEvent);
		if (draftSaveManager) {
			draftSaveManager.destroy();
			draftSaveManager = null;
		} else {
			flushAutoSave();
			markSessionActive();
		}
		if (undoTimer) clearTimeout(undoTimer);
	});

	function handleComposeEvent() {
		const textarea = document.querySelector(
			".compose-input",
		) as HTMLTextAreaElement | null;
		textarea?.focus();
	}

	function switchMode(newMode: "tweet" | "thread") {
		if (newMode === mode) return;
		if (newMode === "thread" && tweetText.trim()) {
			const hasThreadContent = threadBlocks.some((b) => b.text.trim());
			if (!hasThreadContent) {
				threadBlocks = [
					{
						id: crypto.randomUUID(),
						text: tweetText,
						media_paths: [],
						order: 0,
					},
					{
						id: crypto.randomUUID(),
						text: "",
						media_paths: [],
						order: 1,
					},
				];
				tweetText = "";
			}
		}
		mode = newMode;
	}

	function switchToThread() {
		if (mode !== "tweet") return;
		const mediaPaths = attachedMedia.map((m) => m.path);
		threadBlocks = [
			{
				id: crypto.randomUUID(),
				text: tweetText,
				media_paths: mediaPaths,
				order: 0,
			},
			{ id: crypto.randomUUID(), text: "", media_paths: [], order: 1 },
		];
		const focusId = threadBlocks[1].id;
		tweetText = "";
		attachedMedia = [];
		mode = "thread";
		requestAnimationFrame(() => {
			const textarea = document.querySelector(
				`[data-block-id="${focusId}"] textarea`,
			) as HTMLTextAreaElement | null;
			textarea?.focus();
		});
	}

	// ── Autosave / Recovery ────────────────────────────────
	function autoSave() {
		if (draftSaveManager) {
			draftSaveManager.save(mode, tweetText, threadBlocks, attachedMedia);
		} else {
			if (autoSaveTimer) clearTimeout(autoSaveTimer);
			autoSaveTimer = setTimeout(() => {
				saveAutoSave(mode, tweetText, threadBlocks, attachedMedia);
			}, AUTOSAVE_DEBOUNCE_MS);
		}
	}

	function clearAutoSave() {
		clearAutoSaveStorage();
		if (autoSaveTimer) clearTimeout(autoSaveTimer);
	}

	function restoreDraft(data: NonNullable<typeof recoveryData>) {
		mode = (data.mode as "tweet" | "thread") ?? "tweet";
		tweetText = data.tweetText || "";
		threadBlocks = data.blocks || [];
		attachedMedia = restoreMedia(data.tweetMedia);
	}

	function checkRecovery() {
		const data = readAutoSave();
		if (!data) return;

		if (wasNavigationExit()) {
			restoreDraft(data);
			initialized = true;
		} else {
			recoveryData = data;
			showRecovery = true;
		}
	}

	function recoverDraft() {
		if (!recoveryData) return;
		restoreDraft(recoveryData);
		showRecovery = false;
		initialized = true;
	}

	function dismissRecovery() {
		showRecovery = false;
		if (draftId !== undefined) {
			clearDraftAutoSave(draftId);
		} else {
			clearAutoSave();
		}
		initialized = true;
	}

	// ── Handlers ───────────────────────────────────────────
	async function handleSubmit() {
		if (!canSubmit || submitting) return;
		submitting = true;
		submitError = null;
		try {
			const data = buildComposeRequest({
				mode,
				tweetText,
				threadBlocks,
				selectedTime,
				targetDate,
				attachedMedia,
				timezone: accountTimezone,
				scheduledDate,
			});
			if (vaultCitations.length > 0) {
				data.provenance = vaultCitations.map((c) => ({
					node_id: c.node_id,
					chunk_id: c.chunk_id,
					source_path: c.source_path,
					heading_path: c.heading_path,
					snippet: c.snippet,
				}));
			}
			if (data.scheduled_for) {
				trackFunnel('schedule:created', { mode, timezone: accountTimezone });
			} else if (canPublish) {
				trackFunnel('compose:publish-now', { mode });
			} else {
				trackFunnel('compose:save-draft', { mode });
			}
			if (draftSaveManager) {
				await draftSaveManager.flush();
			} else {
				clearAutoSave();
				clearSessionFlag();
			}
			await onsubmit(data);

			// In embedded mode (full-page), reset state after submit since the component doesn't unmount
			if (embedded) {
				// Snapshot current state so user can undo the clear
				undoSnapshot = {
					mode,
					text: tweetText,
					blocks: [...threadBlocks],
					media: [...attachedMedia],
					selectedTime,
					scheduledDate,
				};
				if (selectedTime) {
					undoMessage = 'Scheduled.';
					statusAnnouncement = 'Post scheduled for ' + (scheduledFor() ?? selectedTime);
				} else if (canPublish) {
					undoMessage = 'Published.';
					statusAnnouncement = 'Post published';
				} else {
					undoMessage = 'Saved to calendar.';
					statusAnnouncement = 'Post saved to calendar';
				}

				tweetText = "";
				threadBlocks = [];
				mode = "tweet";
				selectedTime = null;
				scheduledDate = null;
				// Don't revoke media URLs yet — undo may need them
				attachedMedia = [];
				submitting = false;
				submitError = null;
				focusMode = false;
				notesPanelMode = null;
				vaultCitations = [];
				voiceCue = "";
				previewMode = false;

				showUndo = true;
				if (undoTimer) clearTimeout(undoTimer);
				undoTimer = setTimeout(() => {
					showUndo = false;
					// Revoke media URLs now that undo window has closed
					if (undoSnapshot?.media) {
						for (const m of undoSnapshot.media)
							URL.revokeObjectURL(m.previewUrl);
					}
					undoSnapshot = null;
				}, 10000);
			}
		} catch (e) {
			const rawMsg = e instanceof Error ? e.message : 'Failed to submit';
			const errorCtx = selectedTime ? "Couldn't schedule post" : canPublish ? "Couldn't publish" : "Couldn't save draft";
			submitError = errorCtx + ': ' + rawMsg;
			trackFunnel('compose:submit-error', { error_type: rawMsg, mode });
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

	async function handleImagePaste() {
		try {
			const { readImage } = await import(
				"@tauri-apps/plugin-clipboard-manager"
			);
			const img = await readImage();
			const { width: w, height: h } = await img.size();
			const rgba = await img.rgba();

			const canvas = document.createElement("canvas");
			canvas.width = w;
			canvas.height = h;
			const ctx = canvas.getContext("2d");
			if (!ctx) return;
			ctx.putImageData(
				new ImageData(new Uint8ClampedArray(rgba), w, h),
				0,
				0,
			);
			const blob: Blob | null = await new Promise((resolve) =>
				canvas.toBlob(resolve, "image/png"),
			);
			if (!blob) return;

			const file = new File([blob], "pasted-image.png", {
				type: "image/png",
			});
			const result = await api.media.upload(file);

			if (mode === "thread") {
				threadFlowRef?.addMediaToFocusedBlock(result.path);
			} else {
				flushSync(() => {
					attachedMedia = [
						...attachedMedia,
						{
							path: result.path,
							file,
							previewUrl: api.media.fileUrl(result.path),
							mediaType: result.media_type,
						},
					];
				});
			}
			return;
		} catch {
			// Not an image in clipboard — fall through to text paste
		}

		try {
			const { readText } = await import(
				"@tauri-apps/plugin-clipboard-manager"
			);
			const text = await readText();
			if (text) {
				document.execCommand("insertText", false, text);
			}
		} catch {
			// No text in clipboard either
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (paletteOpen) return;

		// When preview overlay is open, only allow Escape and toggle
		if (previewMode) {
			if (e.key === "Escape") {
				e.preventDefault();
				previewMode = false;
				return;
			}
			if (matchEvent(e, "cmd+shift+p")) {
				e.preventDefault();
				togglePreview();
				return;
			}
			return;
		}

		if (matchEvent(e, "cmd+k")) {
			e.preventDefault();
			paletteOpen = true;
			return;
		}
		if (matchEvent(e, "cmd+shift+f")) {
			e.preventDefault();
			if (!embedded) toggleFocusMode();
			return;
		}
		if (matchEvent(e, "cmd+shift+enter")) {
			e.preventDefault();
			handleSubmit();
			return;
		}
		if (matchEvent(e, "cmd+enter")) {
			if (mode === "tweet") {
				e.preventDefault();
				if (tweetText.trim()) switchToThread();
			}
			// In thread mode: let event propagate to ThreadFlowLane's card handler for split
			return;
		}
		if (matchEvent(e, "cmd+v")) {
			e.preventDefault();
			handleImagePaste();
			return;
		}
		if (matchEvent(e, "cmd+shift+j")) {
			e.preventDefault();
			handleInlineAssist();
			return;
		}
		if (matchEvent(e, "cmd+shift+n")) {
			e.preventDefault();
			switchMode("tweet");
			return;
		}
		if (matchEvent(e, "cmd+shift+t")) {
			e.preventDefault();
			switchMode("thread");
			return;
		}
		if (matchEvent(e, "cmd+i")) {
			e.preventDefault();
			toggleInspector();
			return;
		}
		if (matchEvent(e, "cmd+shift+p")) {
			e.preventDefault();
			togglePreview();
			return;
		}
		if (e.key === "Escape") {
			if (notesPanelMode) notesPanelMode = null;
			else if (isMobile && inspectorOpen) inspectorOpen = false;
			else if (!embedded && focusMode) focusMode = false;
			else if (!embedded) handleClose();
			return;
		}
	}

	function handlePaletteAction(actionId: string) {
		paletteOpen = false;
		switch (actionId) {
			case "focus-mode":
				toggleFocusMode();
				break;
			case "mode-tweet":
				switchMode("tweet");
				break;
			case "mode-thread":
				switchMode("thread");
				break;
			case "submit":
				handleSubmit();
				break;
			case "ai-improve":
				handleInlineAssist();
				break;
			case "ai-from-notes":
				notesPanelMode = "notes";
				if (!inspectorOpen) inspectorOpen = true;
				break;
			case "ai-from-vault":
				notesPanelMode = "vault";
				if (!inspectorOpen) inspectorOpen = true;
				break;
			case "ai-generate":
				handleAiAssist();
				break;
			case "toggle-inspector":
				toggleInspector();
				break;
			case "toggle-preview":
				togglePreview();
				break;
			case "attach-media":
				tweetEditorRef?.triggerFileSelect();
				break;
			case "add-card":
			case "duplicate":
			case "split":
			case "merge":
			case "move-up":
			case "move-down":
				threadFlowRef?.handlePaletteAction(actionId);
				break;
			default:
				ondraftaction?.(actionId);
				break;
		}
	}

	async function handleInlineAssist() {
		if (mode === "tweet") {
			const textarea = document.querySelector(
				".compose-input",
			) as HTMLTextAreaElement | null;
			if (!textarea) return;
			const start = textarea.selectionStart;
			const end = textarea.selectionEnd;
			const selectedText =
				start !== end ? tweetText.slice(start, end) : tweetText;
			if (!selectedText.trim()) return;

			// Snapshot before replacement for undo
			undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };
			undoMessage = "Content replaced.";

			assisting = true;
			submitError = null;
			try {
				const result = await api.assist.improve(
					selectedText,
					voiceCue || undefined,
				);
				if (start !== end) {
					tweetText =
						tweetText.slice(0, start) +
						result.content +
						tweetText.slice(end);
				} else {
					tweetText = result.content;
				}
				voicePanelRef?.saveCueToHistory();
				showUndo = true;
				if (undoTimer) clearTimeout(undoTimer);
				undoTimer = setTimeout(() => {
					showUndo = false;
				}, 10000);
			} catch (e) {
				submitError =
					e instanceof Error ? e.message : "AI assist failed";
				undoSnapshot = null;
			} finally {
				assisting = false;
			}
		} else {
			// Thread mode: snapshot all blocks before delegating
			undoSnapshot = { mode, text: tweetText, blocks: [...threadBlocks] };
			undoMessage = "Content replaced.";
			try {
				await threadFlowRef?.handleInlineAssist(voiceCue || undefined);
				showUndo = true;
				if (undoTimer) clearTimeout(undoTimer);
				undoTimer = setTimeout(() => {
					showUndo = false;
				}, 10000);
			} catch {
				undoSnapshot = null;
			}
		}
	}

	async function handleGenerateFromNotes(notesInput: string) {
		submitError = null;
		undoSnapshot = {
			mode,
			text: tweetText,
			blocks: [...threadBlocks],
			citations: [...vaultCitations],
		};
		undoMessage = "Content replaced.";

		if (mode === "thread") {
			const result = await api.assist.thread(
				topicWithCue(voiceCue, notesInput),
			);
			threadBlocks = result.tweets.map((text, i) => ({
				id: crypto.randomUUID(),
				text,
				media_paths: [],
				order: i,
			}));
		} else {
			const context = voiceCue
				? `${voiceCue}. Expand these rough notes into a polished tweet`
				: "Expand these rough notes into a polished tweet";
			const result = await api.assist.improve(notesInput, context);
			tweetText = result.content;
		}
		vaultCitations = [];
		voicePanelRef?.saveCueToHistory();
		notesPanelMode = null;
		showUndo = true;
		if (undoTimer) clearTimeout(undoTimer);
		undoTimer = setTimeout(() => {
			showUndo = false;
		}, 10000);
	}

	async function handleGenerateFromVault(selectedNodeIds: number[]) {
		submitError = null;
		undoSnapshot = {
			mode,
			text: tweetText,
			blocks: [...threadBlocks],
			citations: [...vaultCitations],
		};
		undoMessage = "Content replaced.";

		if (mode === "thread") {
			const result = await api.assist.thread(
				topicWithCue(
					voiceCue,
					tweetText || "Generate thread from vault",
				),
				selectedNodeIds,
			);
			threadBlocks = result.tweets.map((text, i) => ({
				id: crypto.randomUUID(),
				text,
				media_paths: [],
				order: i,
			}));
			if (result.vault_citations) vaultCitations = result.vault_citations;
		} else {
			const result = await api.assist.tweet(
				topicWithCue(
					voiceCue,
					tweetText || "Generate tweet from vault",
				),
				selectedNodeIds,
			);
			tweetText = result.content;
			if (result.vault_citations) vaultCitations = result.vault_citations;
		}
		voicePanelRef?.saveCueToHistory();
		notesPanelMode = null;
		showUndo = true;
		if (undoTimer) clearTimeout(undoTimer);
		undoTimer = setTimeout(() => {
			showUndo = false;
		}, 10000);
	}

	function handleUndo() {
		if (!undoSnapshot) return;
		mode = undoSnapshot.mode;
		tweetText = undoSnapshot.text;
		threadBlocks = undoSnapshot.blocks;
		if (undoSnapshot.media) attachedMedia = undoSnapshot.media;
		if (undoSnapshot.selectedTime !== undefined)
			selectedTime = undoSnapshot.selectedTime;
		if (undoSnapshot.scheduledDate !== undefined)
			scheduledDate = undoSnapshot.scheduledDate;
		vaultCitations = undoSnapshot.citations ?? [];
		undoSnapshot = null;
		showUndo = false;
		if (undoTimer) clearTimeout(undoTimer);
	}

	function openScheduleInInspector() {
		if (!inspectorOpen) {
			inspectorOpen = true;
			persistInspectorState(true);
		}
	}

	function handleScheduleSelect(date: string, time: string) {
		scheduledDate = date;
		selectedTime = time;
	}

	function handleUnschedule() {
		scheduledDate = null;
		selectedTime = null;
	}

	function handleInsertText(text: string) {
		// Find the currently focused textarea and insert text at cursor
		const textarea = document.activeElement as HTMLTextAreaElement | null;
		if (!textarea || textarea.tagName !== "TEXTAREA") {
			// Fallback: try the main compose input or first thread textarea
			const fallback = document.querySelector(
				".compose-input, .flow-textarea",
			) as HTMLTextAreaElement | null;
			if (fallback) {
				fallback.focus();
				const pos = fallback.selectionStart ?? fallback.value.length;
				const before = fallback.value.slice(0, pos);
				const after = fallback.value.slice(pos);
				const newVal = before + text + after;
				if (mode === "tweet") {
					tweetText = newVal;
				}
				// For thread mode the textarea dispatches its own input event
				fallback.value = newVal;
				fallback.dispatchEvent(new Event("input", { bubbles: true }));
				const newPos = pos + text.length;
				fallback.setSelectionRange(newPos, newPos);
			}
			return;
		}
		const pos = textarea.selectionStart ?? textarea.value.length;
		const before = textarea.value.slice(0, pos);
		const after = textarea.value.slice(pos);
		const newVal = before + text + after;
		if (mode === "tweet") {
			tweetText = newVal;
		}
		textarea.value = newVal;
		textarea.dispatchEvent(new Event("input", { bubbles: true }));
		const newPos = pos + text.length;
		textarea.setSelectionRange(newPos, newPos);
		textarea.focus();
	}

	async function handleAiAssist() {
		assisting = true;
		submitError = null;
		try {
			if (mode === "tweet") {
				if (tweetText.trim()) {
					const result = await api.assist.improve(
						tweetText,
						voiceCue || undefined,
					);
					tweetText = result.content;
				} else {
					const result = await api.assist.tweet(
						topicWithCue(voiceCue, "general"),
					);
					tweetText = result.content;
				}
			} else {
				const result = await api.assist.thread(
					topicWithCue(voiceCue, "general"),
				);
				threadBlocks = result.tweets.map((text, i) => ({
					id: crypto.randomUUID(),
					text,
					media_paths: [],
					order: i,
				}));
			}
			voicePanelRef?.saveCueToHistory();
		} catch (e) {
			submitError = e instanceof Error ? e.message : "AI assist failed";
		} finally {
			assisting = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
	{statusAnnouncement}
</div>

{#snippet composeBody()}
	{#if showRecovery}
		<RecoveryBanner onrecover={recoverDraft} ondismiss={dismissRecovery} />
	{/if}

	<ComposerCanvas
		{canSubmit}
		{submitting}
		{selectedTime}
		{submitError}
		{canPublish}
		inspectorOpen={desktopInspectorOpen}
		{embedded}
		timezone={accountTimezone}
		{scheduledDate}
		{schedule}
		scheduledFor={scheduledFor()}
		onsubmit={handleSubmit}
		onscheduleselect={handleScheduleSelect}
		onunschedule={handleUnschedule}
	>
		{#snippet children()}
			{#if mode === "tweet"}
				<TweetEditor
					bind:this={tweetEditorRef}
					text={tweetText}
					onchange={(t) => {
						tweetText = t;
					}}
					{attachedMedia}
					onmediachange={(m) => {
						attachedMedia = m;
					}}
					onerror={(msg) => {
						submitError = msg;
					}}
					avatarUrl={$currentAccount?.x_avatar_url ?? null}
					displayName={$currentAccount?.x_display_name ?? null}
					handle={$currentAccount?.x_username ?? null}
				/>
				<AddTweetDivider
					onclick={switchToThread}
					disabled={!tweetText.trim()}
				/>
			{:else}
				<ThreadFlowLane
					bind:this={threadFlowRef}
					blocks={threadBlocks}
					avatarUrl={$currentAccount?.x_avatar_url ?? null}
					displayName={$currentAccount?.x_display_name ?? null}
					handle={$currentAccount?.x_username ?? null}
					onchange={(b) => {
						threadBlocks = b;
					}}
					onvalidchange={(v) => {
						threadValid = v;
					}}
				/>
			{/if}

			<ComposerInsertBar oninsert={handleInsertText} />

			{#if vaultCitations.length > 0 && notesPanelMode !== "vault"}
				<CitationChips
					citations={vaultCitations}
					vaultPath={localVaultPath}
					{isDesktop}
					onremove={(chunkId) => {
						vaultCitations = vaultCitations.filter(
							(c) => c.chunk_id !== chunkId,
						);
					}}
				/>
			{/if}

			{#if showUndo && !notesPanelMode}
				<div class="undo-banner">
					<span>{undoMessage}</span>
					<button class="undo-btn" onclick={handleUndo}>Undo</button>
				</div>
			{/if}
		{/snippet}

		{#snippet inspector()}
			<InspectorContent
				{schedule}
				{selectedTime}
				{scheduledDate}
				{targetDate}
				timezone={accountTimezone}
				{voiceCue}
				{assisting}
				{hasExistingContent}
				{notesPanelMode}
				{showUndo}
				{mode}
				bind:voicePanelRef
				onscheduleselect={handleScheduleSelect}
				onunschedule={handleUnschedule}
				oncuechange={(c) => {
					voiceCue = c;
				}}
				onaiassist={handleAiAssist}
				onopenotes={() => {
					notesPanelMode = "notes";
				}}
				onopenvault={() => {
					notesPanelMode = "vault";
				}}
				ongenerate={handleGenerateFromNotes}
				ongeneratefromvault={handleGenerateFromVault}
				onclosenotes={() => {
					notesPanelMode = null;
				}}
				onundo={handleUndo}
			/>
		{/snippet}
	</ComposerCanvas>

	{#if isMobile && inspectorOpen}
		<ComposerInspector
			open={inspectorOpen}
			mobile={true}
			onclose={() => {
				inspectorOpen = false;
			}}
		>
			{#snippet children()}
				<InspectorContent
					{schedule}
					{selectedTime}
					{scheduledDate}
					{targetDate}
					timezone={accountTimezone}
					{voiceCue}
					{assisting}
					{hasExistingContent}
					{notesPanelMode}
					{showUndo}
					{mode}
					bind:voicePanelRef
					onscheduleselect={handleScheduleSelect}
					onunschedule={handleUnschedule}
					oncuechange={(c) => {
						voiceCue = c;
					}}
					onaiassist={handleAiAssist}
					onopenotes={() => {
						notesPanelMode = "notes";
					}}
					onopenvault={() => {
						notesPanelMode = "vault";
					}}
					ongenerate={handleGenerateFromNotes}
					ongeneratefromvault={handleGenerateFromVault}
					onclosenotes={() => {
						notesPanelMode = null;
					}}
					onundo={handleUndo}
				/>
			{/snippet}
		</ComposerInspector>
	{/if}

	{#if paletteOpen}
		<CommandPalette
			open={paletteOpen}
			{mode}
			onclose={() => {
				paletteOpen = false;
			}}
			onaction={handlePaletteAction}
			extraActions={extraPaletteActions}
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
				{inspectorOpen}
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
			{canPublish}
			previewVisible={previewMode}
			{mode}
			blockCount={threadBlockCount}
			{headerLeft}
			timezone={accountTimezone}
			{scheduledDate}
			{schedule}
			scheduledFor={scheduledFor()}
			onsubmit={handleSubmit}
			onscheduleselect={handleScheduleSelect}
			onunschedule={handleUnschedule}
			ontoggleinspector={toggleInspector}
			ontogglepreview={togglePreview}
			onopenpalette={() => {
				paletteOpen = true;
			}}
		/>
		{@render composeBody()}
	</div>
{/if}

{#if previewMode}
	<ComposerPreviewSurface
		{mode}
		{tweetText}
		blocks={sortedPreviewBlocks}
		tweetMediaPaths={attachedMedia.map((m) => m.path)}
		tweetLocalPreviews={tweetMediaPreviewMap}
		handle={$currentAccount?.x_username
			? `@${$currentAccount.x_username}`
			: "@you"}
		avatarUrl={$currentAccount?.x_avatar_url ?? null}
		onclose={() => {
			previewMode = false;
		}}
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
