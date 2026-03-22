<script lang="ts">
	import type { VaultCitation, DraftInsert } from "$lib/api/types";
	import {
		X,
		FileText,
		Link,
		ChevronDown,
		ChevronUp,
		ExternalLink,
		Undo2,
	} from "lucide-svelte";
	import { buildObsidianUri, openExternalUrl } from "$lib/utils/obsidianUri";
	import { trackCitationClicked } from "$lib/analytics/backlinkFunnel";

	let {
		citations,
		onremove,
		vaultPath = null,
		isDesktop = false,
		graphInserts = [],
		onundoinsert,
	}: {
		citations: VaultCitation[];
		onremove?: (chunkId: number) => void;
		vaultPath?: string | null;
		isDesktop?: boolean;
		graphInserts?: DraftInsert[];
		onundoinsert?: (insertId: string) => void;
	} = $props();

	let expandedId = $state<number | null>(null);

	function toggleExpand(chunkId: number) {
		const wasExpanded = expandedId === chunkId;
		expandedId = wasExpanded ? null : chunkId;
		if (!wasExpanded) {
			const cit = citations.find((c) => c.chunk_id === chunkId);
			if (cit) trackCitationClicked(chipLabel(cit), false, isDesktop);
		}
	}

	function handleKeydown(e: KeyboardEvent, chunkId: number) {
		if (e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			toggleExpand(chunkId);
		}
	}

	function chipLabel(c: VaultCitation): string {
		const title =
			c.source_title || c.source_path.split("/").pop() || "Note";
		const heading = c.heading_path.split(" > ").pop() || "";
		return heading && heading !== title ? `${title} › ${heading}` : title;
	}

	function obsidianUriFor(cit: VaultCitation): string | null {
		if (!isDesktop || !vaultPath) return null;
		return buildObsidianUri(vaultPath, cit.source_path, cit.heading_path);
	}

	async function handleOpenInObsidian(e: Event, cit: VaultCitation) {
		e.stopPropagation();
		trackCitationClicked(chipLabel(cit), false, isDesktop);
		const uri = obsidianUriFor(cit);
		if (uri) await openExternalUrl(uri);
	}
</script>

{#if citations.length > 0}
	<div class="citation-strip" role="list" aria-label="Source citations">
		<span class="citation-label">Based on:</span>
		<div class="citation-chips">
			{#each citations as cit (cit.chunk_id)}
				<div class="chip-wrapper" role="listitem">
					<button
						class="citation-chip"
						class:expanded={expandedId === cit.chunk_id}
						onclick={() => toggleExpand(cit.chunk_id)}
						onkeydown={(e) => handleKeydown(e, cit.chunk_id)}
						aria-expanded={expandedId === cit.chunk_id}
						aria-label="Citation: {chipLabel(cit)}"
						title={cit.heading_path}
					>
						<FileText size={11} />
						<span class="chip-text">{chipLabel(cit)}</span>
						{#if expandedId === cit.chunk_id}
							<ChevronUp size={10} />
						{:else}
							<ChevronDown size={10} />
						{/if}
					</button>
					{#if obsidianUriFor(cit) || onremove}
						<div class="chip-actions">
							{#if obsidianUriFor(cit)}
								<button
									class="chip-action chip-open"
									onclick={(e) =>
										handleOpenInObsidian(e, cit)}
									aria-label="Open in Obsidian"
									title="Open in Obsidian"
								>
									<ExternalLink size={10} />
								</button>
							{/if}
							{#if onremove}
								<button
									class="chip-action chip-remove"
									onclick={() => onremove?.(cit.chunk_id)}
									aria-label="Remove citation from {chipLabel(
										cit,
									)}"
								>
									<X size={10} />
								</button>
							{/if}
						</div>
					{/if}
					{#if expandedId === cit.chunk_id}
						<div class="chip-detail">
							<div class="chip-detail-path">
								{cit.heading_path}
							</div>
							<div class="chip-detail-snippet">{cit.snippet}</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
{/if}

{#if graphInserts.length > 0}
	<div class="citation-strip graph-strip" role="list" aria-label="Related notes used">
		<span class="citation-label">Related notes:</span>
		<div class="citation-chips">
			{#each graphInserts as ins (ins.id)}
				<div class="chip-wrapper" role="listitem">
					<span class="citation-chip graph-chip">
						<Link size={11} />
						<span class="chip-text">{ins.sourceTitle}</span>
						<span class="chip-slot-label">&rarr; {ins.slotLabel}</span>
					</span>
					{#if onundoinsert}
						<div class="chip-actions">
							<button
								class="chip-action chip-remove"
								onclick={() => onundoinsert?.(ins.id)}
								aria-label="Undo insert from {ins.sourceTitle}"
								title="Undo"
							>
								<Undo2 size={10} />
							</button>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.citation-strip {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		margin-top: 8px;
		padding: 6px 8px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
		border: 1px solid
			color-mix(in srgb, var(--color-accent) 12%, transparent);
	}

	.citation-label {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.03em;
		white-space: nowrap;
		padding-top: 3px;
		flex-shrink: 0;
	}

	.citation-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		min-width: 0;
	}

	.chip-wrapper {
		display: flex;
		align-items: flex-start;
		gap: 1px;
		flex-direction: column;
		position: relative;
		padding-right: 20px;
	}

	.chip-wrapper > :first-child {
		display: flex;
		flex-direction: row;
	}

	.chip-actions {
		position: absolute;
		top: 0;
		right: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.citation-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 6px;
		border: 1px solid
			color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		color: var(--color-text-muted);
		font-size: 11px;
		font-family: var(--font-sans);
		cursor: pointer;
		transition: all 0.12s ease;
		max-width: 220px;
		line-height: 1.4;
	}

	.citation-chip:hover {
		background: color-mix(in srgb, var(--color-accent) 14%, transparent);
		border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
	}

	.citation-chip.expanded {
		border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
	}

	.chip-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.chip-action {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		border: none;
		border-radius: 3px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 0;
		transition: all 0.1s ease;
	}

	.chip-open:hover {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
	}

	.chip-remove:hover {
		background: color-mix(in srgb, var(--color-danger) 15%, transparent);
		color: var(--color-danger);
	}

	.chip-detail {
		margin-top: 2px;
		padding: 4px 6px;
		border-radius: 4px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		font-size: 10px;
		line-height: 1.4;
		max-width: 260px;
	}

	.chip-detail-path {
		color: var(--color-text-subtle);
		margin-bottom: 2px;
		font-weight: 500;
	}

	.chip-detail-snippet {
		color: var(--color-text-muted);
		font-style: italic;
		overflow: hidden;
		display: -webkit-box;
		display: box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		box-orient: vertical;
	}

	.graph-strip {
		background: color-mix(in srgb, #9b59b6 5%, transparent);
		border-color: color-mix(in srgb, #9b59b6 12%, transparent);
	}

	.graph-chip {
		border-color: color-mix(in srgb, #9b59b6 20%, transparent);
		background: color-mix(in srgb, #9b59b6 8%, transparent);
		cursor: default;
	}

	.chip-slot-label {
		font-size: 9px;
		color: var(--color-text-subtle);
		white-space: nowrap;
	}

	@media (prefers-reduced-motion: reduce) {
		.citation-chip,
		.chip-action {
			transition: none;
		}
	}

	@media (pointer: coarse) {
		.citation-chip {
			padding: 4px 8px;
			min-height: 32px;
		}

		.chip-action {
			width: 24px;
			height: 24px;
		}

		.chip-wrapper {
			padding-right: 28px;
		}
	}
</style>
