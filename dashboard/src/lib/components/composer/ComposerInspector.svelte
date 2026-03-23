<script lang="ts">
	import { api, type ScheduleConfig, type ThreadBlock, type ProvenanceRef } from '$lib/api';
	import type { NeighborItem, DraftInsertState } from '$lib/api/types';
	import { topicWithCue } from '$lib/utils/composeHandlers';
	import { createInsertState, pushInsert, popInsert, undoInsertById, buildInsert, hasInserts, getSlotLabel, partitionInserts } from '$lib/stores/draftInsertStore';
	import { createEvidenceState, type EvidenceState, type PinnedEvidence } from '$lib/stores/evidenceStore';
	import { trackSlotTargeted, trackInsertUndone } from '$lib/analytics/backlinkFunnel';
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
		focusedBlockIndex = 0,
		onSelectionConsumed,
		oninsertstatechange,
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
		focusedBlockIndex?: number;
		selectionSessionId?: string | null;
		threadFlowRef?: ThreadFlowLane;
		voicePanelRef?: VoiceContextPanel;
		onclose?: () => void;
		onundo?: () => void;
		onsubmiterror?: (msg: string) => void;
		onSelectionConsumed?: () => void;
		oninsertstatechange?: (state: DraftInsertState) => void;
	} = $props();

	// ── Vault provenance tracking ────────────────────────────
	/** Provenance refs captured from the most recent vault generation. */
	let vaultProvenance: ProvenanceRef[] = $state([]);
	/** Hook style from the most recent hook selection. */
	let vaultHookStyle: string | null = $state(null);

	/** Get the current vault provenance refs (read by parent). */
	export function getVaultProvenance(): ProvenanceRef[] {
		return vaultProvenance;
	}

	/** Get the current hook style (read by parent). */
	export function getVaultHookStyle(): string | null {
		return vaultHookStyle;
	}

	// ── Evidence state ────────────────────────────────────
	let evidenceState = $state<EvidenceState>(createEvidenceState());

	/** Get the current pinned evidence (read by parent). */
	export function getPinnedEvidence(): PinnedEvidence[] {
		return evidenceState.pinned;
	}

	function handleEvidenceChange(newState: EvidenceState) {
		evidenceState = newState;
	}

	async function handleApplyEvidence(evidence: PinnedEvidence, slotIndex: number, slotLabel: string) {
		const blockId = mode === 'tweet' ? 'tweet' : threadBlocks[slotIndex]?.id;
		if (!blockId) return;
		const previousText = mode === 'tweet' ? tweetText : (threadBlocks[slotIndex]?.text ?? '');
		if (!previousText.trim()) return;

		assisting = true;
		try {
			const context = `This is the "${slotLabel}" of a ${mode}. Refine it using this evidence from "${evidence.node_title ?? 'vault'}": ${evidence.snippet}`;
			const result = await api.assist.improve(previousText, context);
			const insertedText = result.content;

			if (mode === 'tweet') {
				tweetText = insertedText;
			} else {
				threadBlocks = threadBlocks.map((b, i) =>
					i === slotIndex ? { ...b, text: insertedText } : b
				);
			}

			const insert = buildInsert({
				blockId,
				slotLabel,
				previousText,
				insertedText,
				sourceNodeId: evidence.node_id,
				sourceTitle: evidence.node_title ?? 'Evidence',
				matchReason: evidence.match_reason,
				similarityScore: evidence.score,
				chunkId: evidence.chunk_id,
				sourceRole: 'semantic_evidence',
				headingPath: evidence.heading_path,
				snippet: evidence.snippet,
			});
			draftInsertState = pushInsert(draftInsertState, insert);

			vaultProvenance = [
				...vaultProvenance,
				{
					node_id: evidence.node_id,
					chunk_id: evidence.chunk_id,
					heading_path: evidence.heading_path,
					snippet: evidence.snippet,
					match_reason: evidence.match_reason,
					similarity_score: evidence.score,
					source_role: 'semantic_evidence',
				},
			];

			undoMessage = `Applied evidence from "${evidence.node_title ?? 'vault'}" to ${slotLabel}.`;
			startUndoTimer();
		} catch (e) {
			onsubmiterror?.(e instanceof Error ? e.message : 'Evidence application failed');
		} finally {
			assisting = false;
		}
	}

	async function handleStrengthenDraft() {
		const pinned = evidenceState.pinned;
		if (pinned.length === 0) return;

		const evidenceContext = pinned
			.map((p) => `"${p.node_title ?? 'vault'}": ${p.snippet}`)
			.join('\n');

		assisting = true;
		try {
			if (mode === 'tweet') {
				if (!tweetText.trim()) return;
				const context = `Strengthen this tweet using these evidence points:\n${evidenceContext}`;
				const result = await api.assist.improve(tweetText, context);
				const insert = buildInsert({
					blockId: 'tweet',
					slotLabel: 'Tweet',
					previousText: tweetText,
					insertedText: result.content,
					sourceNodeId: pinned[0].node_id,
					sourceTitle: `${pinned.length} evidence items`,
					matchReason: 'semantic',
					sourceRole: 'semantic_evidence',
				});
				tweetText = result.content;
				draftInsertState = pushInsert(draftInsertState, insert);
			} else {
				for (let i = 0; i < threadBlocks.length; i++) {
					const block = threadBlocks[i];
					if (!block.text.trim()) continue;
					const slotLabel = getSlotLabel(i, threadBlocks.length);
					const context = `This is the "${slotLabel}" of a thread. Strengthen it using these evidence points:\n${evidenceContext}`;
					const result = await api.assist.improve(block.text, context);
					const insert = buildInsert({
						blockId: block.id,
						slotLabel,
						previousText: block.text,
						insertedText: result.content,
						sourceNodeId: pinned[0].node_id,
						sourceTitle: `${pinned.length} evidence items`,
						matchReason: 'semantic',
						sourceRole: 'semantic_evidence',
					});
					threadBlocks = threadBlocks.map((b, j) =>
						j === i ? { ...b, text: result.content } : b
					);
					draftInsertState = pushInsert(draftInsertState, insert);
				}
			}

			for (const pin of pinned) {
				vaultProvenance = [
					...vaultProvenance,
					{
						node_id: pin.node_id,
						chunk_id: pin.chunk_id,
						heading_path: pin.heading_path,
						snippet: pin.snippet,
						match_reason: pin.match_reason,
						similarity_score: pin.score,
						source_role: 'semantic_evidence',
					},
				];
			}

			undoMessage = mode === 'tweet'
				? 'Strengthened tweet with evidence.'
				: `Strengthened ${threadBlocks.filter((b) => b.text.trim()).length} blocks with evidence.`;
			startUndoTimer();
		} catch (e) {
			onsubmiterror?.(e instanceof Error ? e.message : 'Strengthen failed');
		} finally {
			assisting = false;
		}
	}

	// ── Draft insert state ────────────────────────────────
	let draftInsertState = $state<DraftInsertState>(createInsertState());

	// Notify parent whenever insert state changes
	$effect(() => {
		oninsertstatechange?.(draftInsertState);
	});

	/** Get the current insert state (read by parent). */
	export function getDraftInsertState(): DraftInsertState {
		return draftInsertState;
	}

	/** Check if there are pending insert undos. */
	export function hasPendingInsertUndo(): boolean {
		return hasInserts(draftInsertState);
	}

	/** Handle slot insert: refine a specific block using a neighbor note. */
	export async function handleSlotInsert(neighbor: NeighborItem, slotIndex: number, slotLabel: string) {
		const blockId = mode === 'tweet' ? 'tweet' : threadBlocks[slotIndex]?.id;
		if (!blockId) return;
		const previousText = mode === 'tweet' ? tweetText : (threadBlocks[slotIndex]?.text ?? '');
		if (!previousText.trim()) return;

		assisting = true;
		try {
			const context = `This is the "${slotLabel}" of a ${mode}. Refine it using insights from the note "${neighbor.node_title}": ${neighbor.snippet}`;
			const result = await api.assist.improve(previousText, context);
			const insertedText = result.content;

			if (mode === 'tweet') {
				tweetText = insertedText;
			} else {
				threadBlocks = threadBlocks.map((b, i) =>
					i === slotIndex ? { ...b, text: insertedText } : b
				);
			}

			const insert = buildInsert({
				blockId,
				slotLabel,
				previousText,
				insertedText,
				sourceNodeId: neighbor.node_id,
				sourceTitle: neighbor.node_title,
				edgeType: neighbor.reason,
				edgeLabel: neighbor.reason_label,
			});
			draftInsertState = pushInsert(draftInsertState, insert);

			// Add neighbor provenance
			vaultProvenance = [
				...vaultProvenance,
				{ node_id: neighbor.node_id, edge_type: neighbor.reason, edge_label: neighbor.reason_label, source_role: 'accepted_neighbor' },
			];

			undoMessage = `Applied "${neighbor.node_title}" to ${slotLabel}.`;
			startUndoTimer();
			trackSlotTargeted(slotLabel, neighbor.node_id, selectionSessionId ?? '');
		} catch (e) {
			onsubmiterror?.(e instanceof Error ? e.message : 'Slot refinement failed');
		} finally {
			assisting = false;
		}
	}

	/** Undo the most recent insert. Returns true if an insert was undone. */
	export function handleUndoInsert(): boolean {
		const result = popInsert(draftInsertState);
		if (!result) return false;
		const { newState, undone } = result;
		draftInsertState = newState;

		// Restore the previous text
		if (undone.blockId === 'tweet') {
			tweetText = undone.previousText;
		} else {
			threadBlocks = threadBlocks.map((b) =>
				b.id === undone.blockId ? { ...b, text: undone.previousText } : b
			);
		}

		undoMessage = `Reverted ${undone.slotLabel}.`;
		startUndoTimer();
		trackInsertUndone(undone.id, undone.slotLabel, selectionSessionId ?? '');
		return true;
	}

	/** Undo a specific insert by ID. */
	export function handleUndoInsertById(insertId: string): boolean {
		const result = undoInsertById(draftInsertState, insertId);
		if (!result) return false;
		const { newState, undone } = result;
		draftInsertState = newState;

		if (undone.blockId === 'tweet') {
			tweetText = undone.previousText;
		} else {
			threadBlocks = threadBlocks.map((b) =>
				b.id === undone.blockId ? { ...b, text: undone.previousText } : b
			);
		}

		undoMessage = `Reverted ${undone.slotLabel}.`;
		startUndoTimer();
		trackInsertUndone(undone.id, undone.slotLabel, selectionSessionId ?? '');
		return true;
	}

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

	export async function handleGenerateFromVault(selectedNodeIds: number[], outputFormat: 'tweet' | 'thread' = mode, highlights?: string[], hookStyle?: string, neighborProvenance?: Array<{ node_id: number; edge_type?: string; edge_label?: string; angle_kind?: string; signal_kind?: string; signal_text?: string; source_role?: string }>) {
		if (selectedNodeIds.length === 0) return;
		// Capture provenance from the vault node IDs used for generation.
		// For accepted neighbors, include edge_type, edge_label, and hook miner fields.
		// Primary selection gets source_role: 'primary_selection'.
		vaultProvenance = selectedNodeIds.map((id) => {
			const neighborInfo = neighborProvenance?.find((n) => n.node_id === id);
			if (neighborInfo) {
				return {
					node_id: id,
					edge_type: neighborInfo.edge_type,
					edge_label: neighborInfo.edge_label,
					angle_kind: neighborInfo.angle_kind,
					signal_kind: neighborInfo.signal_kind,
					signal_text: neighborInfo.signal_text,
					source_role: neighborInfo.source_role,
				};
			}
			return { node_id: id, source_role: 'primary_selection' as const, angle_kind: hookStyle };
		});
		vaultHookStyle = hookStyle ?? null;
		try {
			if (hookStyle && highlights && highlights.length > 0) {
				// Hook selected — use hook text directly for tweet, or as opening for thread.
				const hookText = highlights[0];
				if (outputFormat === 'thread') {
					const topic = topicWithCue(voiceCue, hookText);
					const result = await api.assist.thread(topic, selectedNodeIds, hookText);
					threadBlocks = result.tweets.map((text, i) => ({
						id: crypto.randomUUID(), text, media_paths: [], order: i
					}));
				} else {
					tweetText = hookText;
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
		tweetText,
		assisting,
		hasExistingContent,
		notesPanelMode,
		showUndo,
		mode,
		selectionSessionId,
		threadBlocks,
		insertState: draftInsertState,
		evidenceState,
		focusedBlockIndex,
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
					onslotinsert={handleSlotInsert}
					onundoinsert={handleUndoInsertById}
					onevidence={handleEvidenceChange}
					onapplyevidence={handleApplyEvidence}
					onstrengthen={handleStrengthenDraft}
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
		onslotinsert={handleSlotInsert}
		onundoinsert={handleUndoInsertById}
		onevidence={handleEvidenceChange}
		onapplyevidence={handleApplyEvidence}
		onstrengthen={handleStrengthenDraft}
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
