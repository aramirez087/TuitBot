<script lang="ts">
	import { api, type ScheduleConfig, type ThreadBlock } from '$lib/api';
	import { topicWithCue } from '$lib/utils/composeHandlers';
	import InspectorContent from './InspectorContent.svelte';
	import VoiceContextPanel from './VoiceContextPanel.svelte';
	import type ThreadFlowLane from './ThreadFlowLane.svelte';

	let {
		open = $bindable(true),
		isMobile = $bindable(false),
		assisting = $bindable(false),
		voiceCue = $bindable(''),
		notesPanelMode = $bindable<'notes' | 'vault' | null>(null),
		showUndo = $bindable(false),
		undoMessage = $bindable('Content replaced.'),
		tweetText = $bindable(''),
		threadBlocks = $bindable<ThreadBlock[]>([]),
		mode = $bindable<'tweet' | 'thread'>('tweet'),
		schedule,
		selectedTime = $bindable<string | null>(null),
		scheduledDate = $bindable<string | null>(null),
		targetDate,
		timezone = 'UTC',
		hasExistingContent,
		threadFlowRef,
		voicePanelRef = $bindable<VoiceContextPanel | undefined>(undefined),
		selectionSessionId = null,
		onclose,
		onundo,
		onsubmiterror,
		onSelectionConsumed,
	}: {
		open?: boolean;
		isMobile?: boolean;
		assisting?: boolean;
		voiceCue?: string;
		notesPanelMode?: 'notes' | 'vault' | null;
		showUndo?: boolean;
		undoMessage?: string;
		tweetText?: string;
		threadBlocks?: ThreadBlock[];
		mode?: 'tweet' | 'thread';
		schedule: ScheduleConfig | null;
		selectedTime?: string | null;
		scheduledDate?: string | null;
		targetDate: Date;
		timezone?: string;
		hasExistingContent: boolean;
		selectionSessionId?: string | null;
		threadFlowRef?: ThreadFlowLane;
		voicePanelRef?: VoiceContextPanel;
		onclose?: () => void;
		onundo?: () => void;
		onsubmiterror?: (msg: string) => void;
		onSelectionConsumed?: () => void;
	} = $props();

	// ── Undo timer (AI operations) ─────────────────────────
	let undoTimer: ReturnType<typeof setTimeout> | null = null;

	function startUndoTimer() {
		showUndo = true;
		if (undoTimer) clearTimeout(undoTimer);
		undoTimer = setTimeout(() => { showUndo = false; }, 10000);
	}

	// ── AI assist ──────────────────────────────────────────
	export async function handleInlineAssist(snapshotCallback?: () => void) {
		if (mode === 'tweet') {
			const textarea = document.querySelector('.compose-input') as HTMLTextAreaElement | null;
			if (!textarea) return;
			const start = textarea.selectionStart;
			const end = textarea.selectionEnd;
			const selectedText = start !== end ? tweetText.slice(start, end) : tweetText;
			if (!selectedText.trim()) return;

			snapshotCallback?.();
			assisting = true;
			try {
				const result = await api.assist.improve(selectedText, voiceCue || undefined);
				if (start !== end) {
					tweetText = tweetText.slice(0, start) + result.content + tweetText.slice(end);
				} else {
					tweetText = result.content;
				}
				voicePanelRef?.saveCueToHistory();
				startUndoTimer();
			} catch (e) {
				onsubmiterror?.(e instanceof Error ? e.message : 'AI assist failed');
			} finally { assisting = false; }
		} else {
			snapshotCallback?.();
			try {
				await threadFlowRef?.handleInlineAssist(voiceCue || undefined);
				voicePanelRef?.saveCueToHistory();
				startUndoTimer();
			} catch { /* silently ignore */ }
		}
	}

	export async function handleAiAssist() {
		assisting = true;
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
			onsubmiterror?.(e instanceof Error ? e.message : 'AI assist failed');
		} finally { assisting = false; }
	}

	export async function handleGenerateFromNotes(notesInput: string) {
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
		notesPanelMode = null;
		startUndoTimer();
	}

	export async function handleGenerateFromVault(selectedNodeIds: number[], outputFormat: 'tweet' | 'thread' = mode, highlights?: string[]) {
		if (selectedNodeIds.length === 0) return;
		try {
			if (highlights && highlights.length > 0) {
				const highlightContext = highlights.join('\n');
				if (outputFormat === 'thread') {
					const topic = topicWithCue(voiceCue, 'the key highlights provided');
					const result = await api.assist.thread(topic, selectedNodeIds);
					threadBlocks = result.tweets.map((text, i) => ({
						id: crypto.randomUUID(), text, media_paths: [], order: i
					}));
				} else {
					const context = voiceCue
						? `${voiceCue}. Expand these key highlights into a polished tweet`
						: 'Expand these key highlights into a polished tweet';
					const result = await api.assist.improve(highlightContext, context);
					tweetText = result.content;
				}
			} else {
				const topic = topicWithCue(voiceCue, 'the insights and ideas provided in the context above');
				if (outputFormat === 'thread') {
					const result = await api.assist.thread(topic, selectedNodeIds);
					threadBlocks = result.tweets.map((text, i) => ({
						id: crypto.randomUUID(), text, media_paths: [], order: i
					}));
				} else {
					const result = await api.assist.tweet(topic, selectedNodeIds);
					tweetText = result.content;
				}
			}
			mode = outputFormat;
			voicePanelRef?.saveCueToHistory();
			notesPanelMode = null;
			startUndoTimer();
		} catch (e) {
			onsubmiterror?.(e instanceof Error ? e.message : 'AI generate from vault failed');
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) onclose?.();
	}

	function handleBackdropKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); onclose?.(); }
	}

	const inspectorProps = $derived({
		schedule,
		selectedTime,
		scheduledDate,
		targetDate,
		timezone,
		voiceCue,
		assisting,
		hasExistingContent,
		notesPanelMode,
		showUndo,
		mode,
		selectionSessionId,
	});

	function handleScheduleSelect(date: string, time: string) {
		scheduledDate = date;
		selectedTime = time;
	}

	function handleUnschedule() {
		scheduledDate = null;
		selectedTime = null;
	}
</script>

{#if isMobile && open}
	<div
		class="inspector-backdrop"
		onclick={handleBackdropClick}
		onkeydown={handleBackdropKeydown}
		role="presentation"
	>
		<div class="inspector-drawer" role="complementary" aria-label="Composer inspector">
			<div class="drawer-handle-area"><div class="drawer-handle"></div></div>
			<div class="inspector-scroll">
				<InspectorContent
					{...inspectorProps}
					bind:voicePanelRef
					onscheduleselect={handleScheduleSelect}
					onunschedule={handleUnschedule}
					oncuechange={(c) => { voiceCue = c; }}
					onaiassist={handleAiAssist}
					onopenotes={() => { notesPanelMode = 'notes'; }}
					onopenvault={() => { notesPanelMode = 'vault'; }}
					ongenerate={handleGenerateFromNotes}
					ongeneratefromvault={handleGenerateFromVault}
					onclosenotes={() => { notesPanelMode = null; }}
					onundo={() => { onundo?.(); }}
					{onSelectionConsumed}
				/>
			</div>
		</div>
	</div>
{:else if !isMobile && open}
	<InspectorContent
		{...inspectorProps}
		bind:voicePanelRef
		onscheduleselect={handleScheduleSelect}
		onunschedule={handleUnschedule}
		oncuechange={(c) => { voiceCue = c; }}
		onaiassist={handleAiAssist}
		onopenotes={() => { notesPanelMode = 'notes'; }}
		onopenvault={() => { notesPanelMode = 'vault'; }}
		ongenerate={handleGenerateFromNotes}
		ongeneratefromvault={handleGenerateFromVault}
		onclosenotes={() => { notesPanelMode = null; }}
		onundo={() => { onundo?.(); }}
		{onSelectionConsumed}
	/>
{/if}

<style>
	.inspector-backdrop {
		position: fixed; inset: 0;
		background: rgba(0, 0, 0, 0.4);
		z-index: 1099;
		animation: fade-in 0.15s ease;
	}

	.inspector-drawer {
		position: fixed; bottom: 0; left: 0; right: 0;
		max-height: 60vh;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border);
		border-radius: 12px 12px 0 0;
		z-index: 1100;
		display: flex; flex-direction: column;
		box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.3);
		animation: slide-up 0.2s ease;
	}

	.drawer-handle-area {
		display: flex; justify-content: center;
		padding: 8px 0 4px; flex-shrink: 0; cursor: grab;
	}

	.drawer-handle {
		width: 36px; height: 4px;
		border-radius: 2px; background: var(--color-border);
	}

	.inspector-scroll {
		overflow-y: auto;
		padding: 4px 16px calc(16px + env(safe-area-inset-bottom, 0px));
		flex: 1; min-height: 0;
	}

	@keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
	@keyframes slide-up { from { transform: translateY(100%); } to { transform: translateY(0); } }

	@media (prefers-reduced-motion: reduce) {
		.inspector-backdrop, .inspector-drawer { animation: none; }
	}
</style>
