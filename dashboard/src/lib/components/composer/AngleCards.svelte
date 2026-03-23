<script lang="ts">
	import { ArrowLeft, RefreshCw } from 'lucide-svelte';
	import type { MinedAngle } from '$lib/api/types';
	import { getAngleTypeLabel, getEvidenceTypeConfig, truncateCitation } from '$lib/utils/angleStyles';
	import { getConfidenceBadge } from '$lib/utils/hookStyles';
	import { trackAnglesShown, trackAngleSelected } from '$lib/analytics/hookMinerFunnel';

	let {
		angles,
		outputFormat = 'tweet',
		loading = false,
		error = null,
		sessionId = 'unknown',
		sourcePathStem = 'unknown',
		localEligible = true,
		onselect,
		onremine,
		onback,
		onfallback,
		onformatchange,
	}: {
		angles: MinedAngle[];
		outputFormat: 'tweet' | 'thread';
		loading?: boolean;
		error?: string | null;
		sessionId?: string;
		sourcePathStem?: string;
		localEligible?: boolean;
		onselect: (angle: MinedAngle, format: 'tweet' | 'thread') => void;
		onremine: () => void;
		onback: () => void;
		onfallback: () => void;
		onformatchange: (format: 'tweet' | 'thread') => void;
	} = $props();

	let trackedAnglesCount = $state(0);

	$effect(() => {
		if (angles.length > 0 && angles.length !== trackedAnglesCount) {
			trackedAnglesCount = angles.length;
			trackAnglesShown(angles.length, sessionId, sourcePathStem, localEligible);
		}
	});

	let selectedIndex = $state<number | null>(null);

	function handleCardClick(index: number) {
		selectedIndex = index;
	}

	function handleCardKeydown(e: KeyboardEvent, index: number) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			selectedIndex = index;
		}
	}

	function handleConfirm() {
		if (selectedIndex !== null && angles[selectedIndex]) {
			const angle = angles[selectedIndex];
			trackAngleSelected(angle.angle_type, sessionId, sourcePathStem, angle.evidence.length);
			onselect(angle, outputFormat);
		}
	}

	function handleRemine() {
		selectedIndex = null;
		onremine();
	}
</script>

<div class="angle-picker">
	<div class="angle-header">
		<button class="angle-back" onclick={onback} aria-label="Back to related notes">
			<ArrowLeft size={14} />
		</button>
		<span class="angle-label">Mined Angles</span>
	</div>

	{#if error}
		<div class="angle-error" role="alert">
			<span>{error}</span>
			<button class="angle-retry-btn" onclick={handleRemine}>Retry</button>
		</div>
	{/if}

	{#if loading}
		<p class="angle-loading-label" role="status" aria-label="Mining angles">Mining angles from your notes...</p>
	{/if}

	<div class="angle-card-list" role="listbox" aria-label="Mined angle options">
		{#if loading}
			{#each Array(3) as _}
				<div class="angle-card-shimmer" aria-hidden="true">
					<div class="shimmer-pill"></div>
					<div class="shimmer-line"></div>
					<div class="shimmer-line short"></div>
					<div class="shimmer-evidence-block"></div>
					<div class="shimmer-footer"></div>
				</div>
			{/each}
		{:else}
			{#each angles as angle, i}
				<div
					class="angle-card"
					class:selected={selectedIndex === i}
					role="option"
					aria-selected={selectedIndex === i}
					tabindex="0"
					onclick={() => handleCardClick(i)}
					onkeydown={(e) => handleCardKeydown(e, i)}
				>
					<span class="angle-type-pill" title={angle.rationale}>{getAngleTypeLabel(angle.angle_type)}</span>
					<p class="angle-seed-text">{angle.seed_text}</p>
					{#if angle.evidence.length > 0}
						<div class="angle-evidence">
							{#each angle.evidence as ev}
								<div class="angle-evidence-item" aria-label="Evidence from {ev.source_note_title}">
									<span
										class="angle-evidence-pill"
										style="background: color-mix(in srgb, var({getEvidenceTypeConfig(ev.evidence_type).cssVar}) 12%, transparent); color: var({getEvidenceTypeConfig(ev.evidence_type).cssVar});"
									>{getEvidenceTypeConfig(ev.evidence_type).label}</span>
									<span class="angle-evidence-citation">{truncateCitation(ev.citation_text)}</span>
									<span class="angle-evidence-source">from &ldquo;{ev.source_note_title}&rdquo;</span>
								</div>
							{/each}
						</div>
					{/if}
					<div class="angle-card-footer">
						<span class="angle-char-count">{angle.char_count} chars</span>
						<span class="angle-confidence {getConfidenceBadge(angle.confidence).cssClass}">{getConfidenceBadge(angle.confidence).label}</span>
					</div>
				</div>
			{/each}
		{/if}
	</div>

	<div class="angle-footer">
		<div class="angle-footer-row">
			<div class="angle-footer-left">
				<button
					class="angle-remine-btn"
					onclick={handleRemine}
					disabled={loading}
				>
					<RefreshCw size={12} />
					Mine again
				</button>
				<button
					class="angle-fallback-btn"
					onclick={onfallback}
					disabled={loading}
				>
					More hook styles
				</button>
			</div>
			<div class="angle-format-toggle" role="radiogroup" aria-label="Output format">
				<button
					class="angle-format-opt"
					class:active={outputFormat === 'tweet'}
					role="radio"
					aria-checked={outputFormat === 'tweet'}
					onclick={() => onformatchange('tweet')}
				>Tweet</button>
				<button
					class="angle-format-opt"
					class:active={outputFormat === 'thread'}
					role="radio"
					aria-checked={outputFormat === 'thread'}
					onclick={() => onformatchange('thread')}
				>Thread</button>
			</div>
		</div>

		<button
			class="angle-confirm-btn"
			onclick={handleConfirm}
			disabled={selectedIndex === null || loading}
		>
			Use this angle
		</button>
	</div>
</div>

<style>
	.angle-picker {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.angle-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 10px;
	}

	.angle-back {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.angle-back:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.angle-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.angle-loading-label {
		margin: 0 0 6px;
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.angle-error {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 8px 10px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
		font-size: 12px;
		color: var(--color-danger);
		margin-bottom: 8px;
	}

	.angle-retry-btn {
		padding: 3px 10px;
		border: 1px solid var(--color-danger);
		border-radius: 4px;
		background: transparent;
		color: var(--color-danger);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		flex-shrink: 0;
	}

	.angle-retry-btn:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.angle-card-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 340px;
		overflow-y: auto;
		overscroll-behavior: contain;
		margin-bottom: 8px;
	}

	.angle-card {
		padding: 10px 12px;
		border-radius: 6px;
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.angle-card:hover {
		border-color: var(--color-accent);
	}

	.angle-card:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: -2px;
	}

	.angle-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface-active));
	}

	.angle-type-pill {
		display: inline-block;
		padding: 1px 8px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.03em;
		margin-bottom: 6px;
	}

	.angle-seed-text {
		font-size: 13px;
		line-height: 1.45;
		color: var(--color-text);
		margin: 0 0 6px;
		word-break: break-word;
	}

	.angle-evidence {
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		padding: 6px 8px;
		margin-bottom: 6px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 120px;
		overflow-y: auto;
	}

	.angle-evidence-item {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px;
	}

	.angle-evidence-pill {
		display: inline-block;
		padding: 0 5px;
		border-radius: 2px;
		font-size: 9px;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.02em;
		flex-shrink: 0;
	}

	.angle-evidence-citation {
		font-size: 11px;
		color: var(--color-text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.angle-evidence-source {
		font-size: 10px;
		color: var(--color-text-subtle);
		width: 100%;
	}

	.angle-card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.angle-char-count {
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	.angle-confidence {
		font-size: 10px;
		font-weight: 500;
		padding: 1px 6px;
		border-radius: 3px;
	}

	.angle-confidence.confidence-high {
		background: color-mix(in srgb, var(--color-success, #22c55e) 12%, transparent);
		color: var(--color-success, #22c55e);
	}

	.angle-confidence.confidence-medium {
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
		color: var(--color-warning);
	}

	/* Shimmer loading cards */
	.angle-card-shimmer {
		padding: 10px 12px;
		border-radius: 6px;
		background: var(--color-surface-active);
		border: 1px solid var(--color-border-subtle);
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.shimmer-pill {
		width: 60px;
		height: 16px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-accent) 8%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	.shimmer-line {
		width: 100%;
		height: 14px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-text-subtle) 8%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	.shimmer-line.short {
		width: 75%;
	}

	.shimmer-evidence-block {
		width: 90%;
		height: 40px;
		border-radius: 4px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-text-subtle) 6%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	.shimmer-footer {
		width: 40%;
		height: 12px;
		border-radius: 3px;
		background: linear-gradient(90deg, transparent 25%, color-mix(in srgb, var(--color-text-subtle) 6%, transparent) 50%, transparent 75%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}

	@keyframes shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	.angle-footer {
		margin-top: 2px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.angle-footer-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.angle-footer-left {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.angle-remine-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 3px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.angle-remine-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.angle-remine-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.angle-fallback-btn {
		padding: 3px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.angle-fallback-btn:hover:not(:disabled) {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.angle-fallback-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.angle-format-toggle {
		display: flex;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		overflow: hidden;
	}

	.angle-format-opt {
		padding: 3px 10px;
		border: none;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.angle-format-opt:first-child {
		border-right: 1px solid var(--color-border);
	}

	.angle-format-opt.active {
		background: var(--color-accent);
		color: #fff;
	}

	.angle-format-opt:hover:not(.active) {
		background: var(--color-surface-hover);
	}

	.angle-confirm-btn {
		padding: 6px 14px;
		border: 1px solid var(--color-accent);
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.angle-confirm-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.angle-confirm-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	@media (pointer: coarse) {
		.angle-back {
			min-width: 44px;
			min-height: 44px;
		}

		.angle-confirm-btn {
			min-height: 44px;
			padding: 10px 14px;
		}

		.angle-card {
			padding: 12px 14px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.angle-back,
		.angle-card,
		.angle-confirm-btn,
		.angle-remine-btn,
		.angle-fallback-btn {
			transition: none;
		}

		.shimmer-pill,
		.shimmer-line,
		.shimmer-evidence-block,
		.shimmer-footer {
			animation: none;
		}
	}
</style>
