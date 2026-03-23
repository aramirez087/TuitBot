<script lang="ts">
	import { api } from '$lib/api';
	import type { EvidenceResult, ThreadBlock, IndexStatusResponse } from '$lib/api/types';
	import type { EvidenceState, PinnedEvidence } from '$lib/stores/evidenceStore';
	import {
		pinEvidence,
		unpinEvidence,
		dismissEvidence,
		toggleAutoQuery,
		filterResults,
		canPin,
		setLastManualQuery,
	} from '$lib/stores/evidenceStore';
	import {
		trackEvidenceRailOpened,
		trackEvidenceSearchExecuted,
		trackEvidencePinned,
		trackEvidenceDismissed,
		trackAutoQueryToggled,
		trackEvidenceAppliedToSlot,
	} from '$lib/analytics/evidenceFunnel';
	import EvidenceCard from './EvidenceCard.svelte';
	import IndexStatusBadge from './IndexStatusBadge.svelte';
	import { Search, ChevronDown, ChevronRight, Sparkles } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let {
		tweetText = '',
		threadBlocks = [],
		mode = 'tweet',
		focusedBlockIndex,
		hasExistingContent = false,
		selectionSessionId = null,
		graphNeighborChunkIds,
		evidenceState,
		onevidence,
		onapplytoSlot,
	}: {
		tweetText?: string;
		threadBlocks?: ThreadBlock[];
		mode?: 'tweet' | 'thread';
		focusedBlockIndex?: number;
		hasExistingContent?: boolean;
		selectionSessionId?: string | null;
		graphNeighborChunkIds?: Set<number>;
		evidenceState: EvidenceState;
		onevidence: (newState: EvidenceState) => void;
		onapplytoSlot?: (evidence: PinnedEvidence, slotIndex: number, slotLabel: string) => void;
	} = $props();

	let collapsed = $state(false);
	let searchQuery = $state('');
	let results = $state<EvidenceResult[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let indexStatus = $state<IndexStatusResponse | null>(null);
	let autoQueryController = $state<AbortController | null>(null);
	let isSuggested = $state(false);
	let manualSearchTimer: ReturnType<typeof setTimeout> | null = null;
	let autoQueryTimer: ReturnType<typeof setTimeout> | null = null;
	let hasFiredOpen = false;

	const providerConfigured = $derived(indexStatus?.provider_configured ?? false);
	const indexEmpty = $derived(indexStatus != null && indexStatus.total_chunks === 0 && !indexStatus.index_loaded);
	const indexStale = $derived(indexStatus != null && indexStatus.freshness_pct < 50 && indexStatus.total_chunks > 0);
	const buildingIndex = $derived(
		indexStatus != null &&
		indexStatus.total_chunks > 0 &&
		indexStatus.embedded_chunks < indexStatus.total_chunks &&
		indexStatus.freshness_pct < 95
	);

	const filteredResults = $derived(
		filterResults(results, evidenceState, graphNeighborChunkIds ?? new Set())
	);

	const focusedText = $derived.by(() => {
		if (mode === 'tweet') return tweetText;
		if (focusedBlockIndex != null && threadBlocks[focusedBlockIndex]) {
			return threadBlocks[focusedBlockIndex].text;
		}
		return threadBlocks[0]?.text ?? '';
	});

	onMount(async () => {
		try {
			indexStatus = await api.vault.indexStatus();
		} catch {
			indexStatus = null;
		}
	});

	$effect(() => {
		if (indexStatus && providerConfigured && !hasFiredOpen) {
			hasFiredOpen = true;
			trackEvidenceRailOpened(selectionSessionId ?? '', !!selectionSessionId);
		}
	});

	// Auto-query effect
	$effect(() => {
		if (!evidenceState.autoQueryEnabled || !providerConfigured) return;
		const text = focusedText;
		if (!text || text.trim().length < 10) return;

		if (autoQueryTimer) clearTimeout(autoQueryTimer);
		autoQueryTimer = setTimeout(() => {
			executeSearch(text.trim().slice(0, 200), 'semantic', 5, true);
		}, 800);
	});

	async function executeSearch(query: string, searchMode: string, limit: number, suggested: boolean) {
		if (autoQueryController) {
			autoQueryController.abort();
		}
		const controller = new AbortController();
		autoQueryController = controller;
		loading = true;
		error = null;
		isSuggested = suggested;

		try {
			const response = await api.vault.searchEvidence({ q: query, limit, mode: searchMode });
			if (controller.signal.aborted) return;
			results = response.results;
			trackEvidenceSearchExecuted(query.length, response.results.length, searchMode);
		} catch (e) {
			if (controller.signal.aborted) return;
			error = e instanceof Error ? e.message : 'Search failed';
			results = [];
		} finally {
			if (!controller.signal.aborted) {
				loading = false;
			}
		}
	}

	function handleManualSearch() {
		const q = searchQuery.trim();
		if (!q) { results = []; return; }
		onevidence(setLastManualQuery(evidenceState, q));
		if (manualSearchTimer) clearTimeout(manualSearchTimer);
		manualSearchTimer = setTimeout(() => {
			executeSearch(q, 'hybrid', 8, false);
		}, 300);
	}

	function handleSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			searchQuery = '';
			results = [];
			collapsed = true;
		}
	}

	function handlePin(result: EvidenceResult) {
		if (!canPin(evidenceState)) return;
		onevidence(pinEvidence(evidenceState, result));
		trackEvidencePinned(result.chunk_id, result.match_reason, result.score);
	}

	function handleUnpin(chunkId: number) {
		onevidence(unpinEvidence(evidenceState, chunkId));
	}

	function handleDismiss(result: EvidenceResult) {
		onevidence(dismissEvidence(evidenceState, result.chunk_id));
		trackEvidenceDismissed(result.chunk_id, result.match_reason);
	}

	function handleToggleAutoQuery() {
		const next = toggleAutoQuery(evidenceState);
		onevidence(next);
		trackAutoQueryToggled(next.autoQueryEnabled);
	}

	function handleApply(evidence: PinnedEvidence) {
		const slotIndex = mode === 'tweet' ? 0 : (focusedBlockIndex ?? 0);
		const total = mode === 'tweet' ? 1 : threadBlocks.length;
		let slotLabel = 'Tweet';
		if (total > 1) {
			if (slotIndex === 0) slotLabel = 'Opening hook';
			else if (slotIndex === total - 1) slotLabel = 'Closing takeaway';
			else slotLabel = `Tweet ${slotIndex + 1}`;
		}
		trackEvidenceAppliedToSlot(evidence.chunk_id, slotIndex, slotLabel, evidence.match_reason);
		onapplytoSlot?.(evidence, slotIndex, slotLabel);
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'e') {
			e.preventDefault();
			collapsed = !collapsed;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if providerConfigured}
	<div class="evidence-rail" class:collapsed>
		<button
			class="rail-header"
			onclick={() => { collapsed = !collapsed; }}
			aria-expanded={!collapsed}
			aria-controls="evidence-rail-content"
		>
			{#if collapsed}
				<ChevronRight size={14} />
			{:else}
				<ChevronDown size={14} />
			{/if}
			<span class="rail-label">Evidence</span>
			<kbd class="rail-kbd">{'\u2318\u21e7'}E</kbd>
			{#if evidenceState.pinned.length > 0}
				<span class="pinned-count">{evidenceState.pinned.length}</span>
			{/if}
			<IndexStatusBadge status={indexStatus} compact />
		</button>

		{#if !collapsed}
			<div id="evidence-rail-content" class="rail-content">
				{#if indexEmpty}
					<div class="rail-empty-state">
						<p class="empty-message">Building index...</p>
						{#if indexStatus}
							<div class="progress-bar">
								<div
									class="progress-fill"
									style="width: {indexStatus.total_chunks > 0 ? (indexStatus.embedded_chunks / indexStatus.total_chunks * 100) : 0}%"
								></div>
							</div>
							<p class="empty-detail">{indexStatus.embedded_chunks} / {indexStatus.total_chunks} chunks indexed</p>
						{/if}
					</div>
				{:else}
					{#if indexStale}
						<div class="stale-warning" role="alert">
							Index may show outdated results ({indexStatus?.freshness_pct}% fresh)
						</div>
					{/if}

					<div class="search-row">
						<div class="search-input-wrap">
							<Search size={13} />
							<input
								type="text"
								class="search-input"
								placeholder="Search your vault..."
								bind:value={searchQuery}
								oninput={handleManualSearch}
								onkeydown={handleSearchKeydown}
								aria-label="Search evidence"
							/>
						</div>
						<button
							class="auto-query-btn"
							class:active={evidenceState.autoQueryEnabled}
							onclick={handleToggleAutoQuery}
							title={evidenceState.autoQueryEnabled ? 'Disable auto-suggestions' : 'Enable auto-suggestions from draft text'}
							aria-pressed={evidenceState.autoQueryEnabled}
							aria-label="Toggle auto-query"
						>
							<Sparkles size={13} />
						</button>
					</div>

					{#if evidenceState.pinned.length > 0}
						<div class="pinned-section">
							<span class="section-label">Pinned ({evidenceState.pinned.length}/5)</span>
							<div class="card-list">
								{#each evidenceState.pinned as pin (pin.chunk_id)}
									<EvidenceCard
										result={{ ...pin, relative_path: pin.relative_path }}
										pinned
										{hasExistingContent}
										onunpin={() => handleUnpin(pin.chunk_id)}
										onapply={() => handleApply(pin)}
									/>
								{/each}
							</div>
						</div>
					{/if}

					{#if loading}
						<div class="shimmer-list" role="status" aria-label="Searching">
							{#each [1, 2, 3] as _i}
								<div class="shimmer-card"></div>
							{/each}
						</div>
					{:else if error}
						<div class="rail-error">{error}</div>
					{:else if filteredResults.length > 0}
						<div class="results-section">
							<span class="section-label">
								{isSuggested ? 'Suggestions' : 'Results'} ({filteredResults.length})
							</span>
							<div class="card-list">
								{#each filteredResults as result (result.chunk_id)}
									<EvidenceCard
										{result}
										suggested={isSuggested}
										{hasExistingContent}
										onpin={() => handlePin(result)}
										ondismiss={() => handleDismiss(result)}
										onapply={() => {
											const pinned = pinEvidence(evidenceState, result);
											onevidence(pinned);
											const pe = pinned.pinned.find(p => p.chunk_id === result.chunk_id);
											if (pe) handleApply(pe);
										}}
									/>
								{/each}
							</div>
						</div>
					{:else if searchQuery.trim() && !isSuggested}
						<p class="no-results">No matching evidence found</p>
					{/if}
				{/if}
			</div>
		{/if}
	</div>
{/if}

<style>
	.evidence-rail {
		padding: 0;
	}

	.rail-header {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 4px 0;
		border: none;
		background: none;
		cursor: pointer;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		text-align: left;
	}

	.rail-header:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
		border-radius: 4px;
	}

	.rail-label {
		flex: 0 0 auto;
	}

	.rail-kbd {
		display: inline-flex;
		align-items: center;
		padding: 0 4px;
		border-radius: 3px;
		background: var(--color-surface-hover);
		color: var(--color-text-subtle);
		font-size: 10px;
		font-family: var(--font-mono);
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
		line-height: 1.6;
	}

	.pinned-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 16px;
		height: 16px;
		padding: 0 4px;
		border-radius: 8px;
		background: var(--color-accent);
		color: #fff;
		font-size: 10px;
		font-weight: 600;
		line-height: 1;
	}

	.rail-content {
		padding: 8px 0 0;
	}

	.stale-warning {
		font-size: 11px;
		color: #f59e0b;
		padding: 6px 8px;
		border: 1px solid color-mix(in srgb, #f59e0b 30%, transparent);
		border-radius: 5px;
		background: color-mix(in srgb, #f59e0b 6%, transparent);
		margin-bottom: 8px;
	}

	.search-row {
		display: flex;
		gap: 6px;
		margin-bottom: 8px;
	}

	.search-input-wrap {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-surface);
		color: var(--color-text-subtle);
		transition: border-color 0.15s ease;
	}

	.search-input-wrap:focus-within {
		border-color: var(--color-accent);
	}

	.search-input {
		flex: 1;
		border: none;
		background: none;
		outline: none;
		color: var(--color-text);
		font-size: 12px;
		font-family: inherit;
	}

	.search-input::placeholder {
		color: var(--color-text-muted);
	}

	.auto-query-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 5px 8px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.auto-query-btn:hover {
		border-color: var(--color-border);
		color: var(--color-text);
	}

	.auto-query-btn.active {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.auto-query-btn:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 1px;
	}

	.section-label {
		display: block;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-muted);
		margin-bottom: 6px;
	}

	.pinned-section {
		margin-bottom: 10px;
	}

	.results-section {
		margin-bottom: 4px;
	}

	.card-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.shimmer-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.shimmer-card {
		height: 60px;
		border-radius: 6px;
		background: linear-gradient(90deg, var(--color-surface-hover) 25%, var(--color-surface) 50%, var(--color-surface-hover) 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	@keyframes shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	.rail-empty-state {
		padding: 12px 0;
		text-align: center;
	}

	.empty-message {
		font-size: 12px;
		color: var(--color-text-subtle);
		margin: 0 0 8px;
	}

	.empty-detail {
		font-size: 10px;
		color: var(--color-text-muted);
		margin: 4px 0 0;
	}

	.progress-bar {
		height: 4px;
		border-radius: 2px;
		background: var(--color-surface-hover);
		overflow: hidden;
		margin: 0 20px;
	}

	.progress-fill {
		height: 100%;
		border-radius: 2px;
		background: var(--color-accent);
		transition: width 0.3s ease;
	}

	.no-results {
		font-size: 12px;
		color: var(--color-text-subtle);
		text-align: center;
		padding: 12px 0;
		margin: 0;
	}

	.rail-error {
		font-size: 11px;
		color: #ef4444;
		padding: 6px 8px;
		border: 1px solid color-mix(in srgb, #ef4444 30%, transparent);
		border-radius: 5px;
		background: color-mix(in srgb, #ef4444 6%, transparent);
	}

	@media (prefers-reduced-motion: reduce) {
		.shimmer-card { animation: none; }
		.search-input-wrap { transition: none; }
		.auto-query-btn { transition: none; }
		.progress-fill { transition: none; }
	}

	@media (pointer: coarse) {
		.auto-query-btn { min-height: 44px; }
		.search-input-wrap { min-height: 44px; }
	}
</style>
