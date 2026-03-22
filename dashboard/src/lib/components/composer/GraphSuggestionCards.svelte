<script lang="ts">
	import type { NeighborItem, GraphState } from '$lib/api/types';
	import { X, Lightbulb, BookOpen, ArrowLeftRight, Link } from 'lucide-svelte';
	import { fly } from 'svelte/transition';
	import { trackSuggestionsShown, trackEmptyGraph } from '$lib/analytics/backlinkFunnel';

	let {
		neighbors,
		graphState,
		loading = false,
		sessionId = '',
		onaccept,
		ondismiss,
	}: {
		neighbors: NeighborItem[];
		graphState: GraphState;
		loading?: boolean;
		sessionId?: string;
		onaccept: (neighbor: NeighborItem, role: string) => void;
		ondismiss: (nodeId: number) => void;
	} = $props();

	const intentBadges: Record<string, string> = {
		pro_tip: 'pro tip',
		evidence: 'evidence',
		counterpoint: 'counterpoint',
		related: 'context',
	};

	function intentRoleFor(intent: string): string {
		return intent in intentBadges ? intent : 'related';
	}

	// Fire suggestions_shown once per session when neighbors appear
	let hasFiredShown = false;
	$effect(() => {
		if (neighbors.length > 0 && !hasFiredShown && !loading) {
			hasFiredShown = true;
			trackSuggestionsShown(neighbors.length, sessionId, graphState);
		}
	});

	// Fire empty_graph when relevant empty state renders
	let hasFiredEmpty = false;
	$effect(() => {
		const isEmpty = graphState === 'no_related_notes' || (graphState === 'available' && neighbors.length === 0) || graphState === 'node_not_indexed';
		if (isEmpty && !hasFiredEmpty && !loading) {
			hasFiredEmpty = true;
			trackEmptyGraph(graphState, sessionId);
		}
	});

	const reducedMotion = typeof window !== 'undefined' && typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

	function truncateSnippet(text: string, max: number = 120): string {
		if (text.length <= max) return text;
		return text.slice(0, max).trimEnd() + '\u2026';
	}

	function scoreOpacity(score: number): number {
		const clamped = Math.min(Math.max(score, 0), 10);
		return 0.3 + (clamped / 10) * 0.7;
	}

	function getReasonIcon(reason: string) {
		if (reason === 'mutual_link') return ArrowLeftRight;
		if (reason === 'backlink') return Link;
		if (reason === 'shared_tag') return BookOpen;
		return Lightbulb;
	}
</script>

{#if loading}
	<div class="graph-suggestions" role="status" aria-label="Finding related notes">
		<div class="graph-suggestions-header">
			<span class="graph-suggestions-label">Finding related notes...</span>
		</div>
		<div class="graph-shimmer-list">
			{#each [1, 2, 3] as _i}
				<div class="graph-shimmer-card">
					<div class="graph-shimmer-line short"></div>
					<div class="graph-shimmer-line long"></div>
					<div class="graph-shimmer-line medium"></div>
				</div>
			{/each}
		</div>
	</div>
{:else if graphState === 'fallback_active'}
	<!-- Silent fallback: render nothing -->
{:else if graphState === 'node_not_indexed'}
	<div class="graph-suggestions" role="status">
		<div class="graph-empty-state">
			<p class="graph-empty-message">This note hasn't been indexed yet. Generating from your selected text.</p>
		</div>
	</div>
{:else if graphState === 'no_related_notes' || (graphState === 'available' && neighbors.length === 0)}
	<div class="graph-suggestions" role="status">
		<div class="graph-empty-state">
			<p class="graph-empty-message">This note doesn't link to other indexed notes. You can still generate from this selection alone.</p>
		</div>
	</div>
{:else if graphState === 'available' && neighbors.length > 0}
	<div class="graph-suggestions" role="list" aria-label="Related note suggestions">
		<div class="graph-suggestions-header">
			<span class="graph-suggestions-label">Related notes from your vault</span>
			<span class="graph-suggestions-count">{neighbors.length}</span>
		</div>
		{#each neighbors as neighbor (neighbor.node_id)}
			{@const ReasonIcon = getReasonIcon(neighbor.reason)}
			<div
				class="graph-card"
				role="listitem"
				aria-label="Suggestion: {neighbor.node_title}"
				in:fly={{ y: 8, duration: reducedMotion ? 0 : 150 }}
				out:fly={{ y: -8, duration: reducedMotion ? 0 : 120 }}
			>
				<div class="graph-card-header">
					<span class="graph-card-title">{neighbor.node_title}</span>
					<button
						class="graph-card-dismiss"
						onclick={() => ondismiss(neighbor.node_id)}
						aria-label="Skip {neighbor.node_title}"
						title="Skip this note"
					>
						<X size={12} />
						<span class="dismiss-label">Skip</span>
					</button>
				</div>
				<div class="graph-card-snippet">{truncateSnippet(neighbor.snippet)}</div>
				<div class="graph-card-meta">
					<span class="graph-reason-badge" aria-label="Reason: {neighbor.reason_label}">
						<ReasonIcon size={10} />
						{neighbor.reason_label}
					</span>
					<span class="graph-intent-badge">{intentBadges[neighbor.intent] ?? 'context'}</span>
					<span class="graph-score-dot" style="opacity: {scoreOpacity(neighbor.score)}" title="Relevance: {neighbor.score.toFixed(1)}"></span>
				</div>
				<div class="graph-card-actions">
					<button
						class="graph-action-btn"
						onclick={() => onaccept(neighbor, intentRoleFor(neighbor.intent))}
						title="Include insights from this note in your draft"
					>
						Include
					</button>
				</div>
			</div>
		{/each}
	</div>
{/if}

<style>
	.graph-suggestions {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 8px;
	}

	.graph-suggestions-header {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.graph-suggestions-label {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.03em;
	}

	.graph-suggestions-count {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		padding: 0 5px;
		border-radius: 8px;
		line-height: 1.6;
	}

	/* Shimmer loading */
	.graph-shimmer-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.graph-shimmer-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
	}

	.graph-shimmer-line {
		height: 10px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-accent) 8%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: graph-shimmer 1.5s infinite;
	}

	.graph-shimmer-line.short { width: 40%; }
	.graph-shimmer-line.long { width: 90%; }
	.graph-shimmer-line.medium { width: 60%; }

	@keyframes graph-shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	/* Empty state */
	.graph-empty-state {
		padding: 8px 10px;
	}

	.graph-empty-message {
		margin: 0;
		font-size: 11px;
		color: var(--color-text-subtle);
		line-height: 1.5;
	}

	/* Card */
	.graph-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-surface);
		transition: border-color 0.12s ease;
	}

	.graph-card:hover {
		border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
	}

	.graph-card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
	}

	.graph-card-title {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.graph-card-dismiss {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 2px;
		height: 18px;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 0 4px;
		flex-shrink: 0;
		transition: all 0.1s ease;
		font-size: 10px;
		font-weight: 500;
	}

	.dismiss-label {
		font-size: 10px;
	}

	.graph-card-dismiss:hover {
		background: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.graph-card-snippet {
		font-size: 11px;
		color: var(--color-text-muted);
		line-height: 1.4;
	}

	.graph-card-meta {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.graph-reason-badge {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: 10px;
		padding: 1px 6px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-text-subtle) 8%, transparent);
		color: var(--color-text-subtle);
	}

	.graph-intent-badge {
		font-size: 9px;
		padding: 0 4px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-accent);
		font-weight: 500;
		text-transform: lowercase;
	}

	.graph-score-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-accent);
		flex-shrink: 0;
	}

	.graph-card-actions {
		display: flex;
		gap: 4px;
		margin-top: 2px;
	}

	.graph-action-btn {
		padding: 2px 8px;
		border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
		border-radius: 4px;
		background: transparent;
		color: var(--color-accent);
		font-size: 10px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.graph-action-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	@media (prefers-reduced-motion: reduce) {
		.graph-card, .graph-action-btn, .graph-card-dismiss, .graph-shimmer-line {
			transition: none;
			animation: none;
		}
	}

	@media (pointer: coarse) {
		.graph-card-dismiss {
			width: 24px;
			height: 24px;
		}

		.graph-action-btn {
			padding: 4px 10px;
			min-height: 28px;
		}
	}
</style>
