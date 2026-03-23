/**
 * Hook Miner and Forge funnel events.
 *
 * Wraps `trackFunnel()` with typed helpers for the angle-mining
 * and analytics-sync features. All events use `hook_miner.` or
 * `forge.` namespace prefixes.
 *
 * Privacy: no helper ever sends raw note content, full file paths,
 * or frontmatter values. Source paths are reduced to filename stems.
 */

import { trackFunnel } from './funnel';

// ── Path sanitization ────────────────────────────────────────

/**
 * Strips directory components from a path, returning only the
 * filename stem (no extension). Ensures no full paths leak into
 * telemetry events.
 */
export function sanitizePathStem(raw: string): string {
	// Strip directory separators (Unix and Windows)
	const filename = raw.split(/[/\\]/).pop() ?? raw;
	// Remove extension
	const dot = filename.lastIndexOf('.');
	return dot > 0 ? filename.slice(0, dot) : filename;
}

// ── Hook Miner events ────────────────────────────────────────

/** Angle cards shown to the user after mining. */
export function trackAnglesShown(
	angleCount: number,
	sessionId: string,
	sourcePathStem: string,
	localEligible: boolean
): void {
	trackFunnel('hook_miner.angles_shown', {
		angle_count: angleCount,
		session_id: sessionId,
		source_path_stem: sanitizePathStem(sourcePathStem),
		local_eligible: localEligible
	});
}

/** User selects an angle card. */
export function trackAngleSelected(
	angleKind: string,
	sessionId: string,
	sourcePathStem: string,
	evidenceCount: number
): void {
	trackFunnel('hook_miner.angle_selected', {
		angle_kind: angleKind,
		session_id: sessionId,
		source_path_stem: sanitizePathStem(sourcePathStem),
		evidence_count: evidenceCount
	});
}

/** Fallback triggered (weak signal, no neighbors, etc.). */
export function trackFallbackOpened(
	reason: string,
	sessionId: string,
	acceptedCount: number
): void {
	trackFunnel('hook_miner.fallback_opened', {
		reason,
		session_id: sessionId,
		accepted_count: acceptedCount
	});
}

// ── Forge events ─────────────────────────────────────────────

/** Analytics sync consent prompt shown on Activity page. */
export function trackForgePromptShown(
	sourcePathStem: string,
	localEligible: boolean
): void {
	trackFunnel('forge.prompt_shown', {
		source_path_stem: sanitizePathStem(sourcePathStem),
		local_eligible: localEligible
	});
}

/** User enables analytics sync (from prompt banner or settings). */
export function trackForgeEnabled(
	sourcePathStem: string,
	enabledFrom: string
): void {
	trackFunnel('forge.enabled', {
		source_path_stem: sanitizePathStem(sourcePathStem),
		enabled_from: enabledFrom
	});
}

/** Forge sync completed successfully. */
export function trackForgeSyncSucceeded(
	tweetsSynced: number,
	threadsSynced: number,
	entriesNotFound: number,
	filesNotFound: number
): void {
	trackFunnel('forge.sync_succeeded', {
		tweets_synced: tweetsSynced,
		threads_synced: threadsSynced,
		entries_not_found: entriesNotFound,
		files_not_found: filesNotFound
	});
}

/** Forge sync failed. */
export function trackForgeSyncFailed(
	reason: string,
	stage: string
): void {
	trackFunnel('forge.sync_failed', {
		reason,
		stage
	});
}
