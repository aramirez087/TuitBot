import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$lib/analytics/funnel', () => ({
	trackFunnel: vi.fn()
}));

import { trackFunnel } from '$lib/analytics/funnel';
import {
	sanitizePathStem,
	trackAnglesShown,
	trackAngleSelected,
	trackFallbackOpened,
	trackForgePromptShown,
	trackForgeEnabled,
	trackForgeSyncSucceeded,
	trackForgeSyncFailed
} from '$lib/analytics/hookMinerFunnel';

const mockTrack = vi.mocked(trackFunnel);

beforeEach(() => {
	mockTrack.mockClear();
});

// ── sanitizePathStem ─────────────────────────────────────────

describe('sanitizePathStem', () => {
	it('strips Unix directory components', () => {
		expect(sanitizePathStem('/Users/alice/vault/my-note.md')).toBe('my-note');
	});

	it('strips Windows directory components', () => {
		expect(sanitizePathStem('C:\\Users\\alice\\vault\\my-note.md')).toBe('my-note');
	});

	it('removes file extension', () => {
		expect(sanitizePathStem('my-note.md')).toBe('my-note');
	});

	it('returns stem as-is when no extension or path', () => {
		expect(sanitizePathStem('my-note')).toBe('my-note');
	});

	it('handles multiple dots in filename', () => {
		expect(sanitizePathStem('my.note.draft.md')).toBe('my.note.draft');
	});

	it('handles empty string without crashing', () => {
		expect(sanitizePathStem('')).toBe('');
	});

	it('returns bare stem when no extension', () => {
		expect(sanitizePathStem('README')).toBe('README');
	});
});

// ── Hook Miner events ────────────────────────────────────────

describe('trackAnglesShown', () => {
	it('emits hook_miner.angles_shown with correct properties', () => {
		trackAnglesShown(3, 'sess-1', 'my-note', true);
		expect(mockTrack).toHaveBeenCalledWith('hook_miner.angles_shown', {
			angle_count: 3,
			session_id: 'sess-1',
			source_path_stem: 'my-note',
			local_eligible: true
		});
	});

	it('sanitizes full paths in sourcePathStem', () => {
		trackAnglesShown(1, 's', '/vault/secret/note.md', false);
		const props = mockTrack.mock.calls[0][1] as Record<string, unknown>;
		expect(props.source_path_stem).toBe('note');
	});
});

describe('trackAngleSelected', () => {
	it('emits hook_miner.angle_selected with angle_kind', () => {
		trackAngleSelected('contradiction', 'sess-2', 'draft', 5);
		expect(mockTrack).toHaveBeenCalledWith('hook_miner.angle_selected', {
			angle_kind: 'contradiction',
			session_id: 'sess-2',
			source_path_stem: 'draft',
			evidence_count: 5
		});
	});
});

describe('trackFallbackOpened', () => {
	it('emits hook_miner.fallback_opened with reason', () => {
		trackFallbackOpened('weak_signal', 'sess-3', 2);
		expect(mockTrack).toHaveBeenCalledWith('hook_miner.fallback_opened', {
			reason: 'weak_signal',
			session_id: 'sess-3',
			accepted_count: 2
		});
	});
});

// ── Forge events ─────────────────────────────────────────────

describe('trackForgePromptShown', () => {
	it('emits forge.prompt_shown with local_eligible', () => {
		trackForgePromptShown('my-vault-note', true);
		expect(mockTrack).toHaveBeenCalledWith('forge.prompt_shown', {
			source_path_stem: 'my-vault-note',
			local_eligible: true
		});
	});
});

describe('trackForgeEnabled', () => {
	it('emits forge.enabled with enabled_from', () => {
		trackForgeEnabled('project-note', 'settings');
		expect(mockTrack).toHaveBeenCalledWith('forge.enabled', {
			source_path_stem: 'project-note',
			enabled_from: 'settings'
		});
	});
});

describe('trackForgeSyncSucceeded', () => {
	it('emits forge.sync_succeeded with count properties', () => {
		trackForgeSyncSucceeded(10, 3, 1, 2);
		expect(mockTrack).toHaveBeenCalledWith('forge.sync_succeeded', {
			tweets_synced: 10,
			threads_synced: 3,
			entries_not_found: 1,
			files_not_found: 2
		});
	});
});

describe('trackForgeSyncFailed', () => {
	it('emits forge.sync_failed with reason and stage', () => {
		trackForgeSyncFailed('permission_denied', 'file_write');
		expect(mockTrack).toHaveBeenCalledWith('forge.sync_failed', {
			reason: 'permission_denied',
			stage: 'file_write'
		});
	});
});

// ── Privacy guarantees ───────────────────────────────────────

describe('privacy', () => {
	it('sourcePathStem never contains path separators', () => {
		trackAnglesShown(1, 's', '/a/b/c/note.md', true);
		trackAngleSelected('x', 's', 'C:\\Users\\note.md', 1);
		trackForgePromptShown('/vault/note.md', true);
		trackForgeEnabled('C:\\docs\\note.md', 'prompt');

		for (const call of mockTrack.mock.calls) {
			const props = call[1] as Record<string, unknown>;
			if (typeof props.source_path_stem === 'string') {
				expect(props.source_path_stem).not.toMatch(/[/\\]/);
			}
		}
	});

	it('no event leaks raw content properties', () => {
		trackAnglesShown(1, 's', 'n', true);
		trackAngleSelected('k', 's', 'n', 1);
		trackFallbackOpened('r', 's', 0);
		trackForgePromptShown('n', true);
		trackForgeEnabled('n', 'prompt');
		trackForgeSyncSucceeded(0, 0, 0, 0);
		trackForgeSyncFailed('r', 's');

		const forbidden = ['content', 'body', 'raw_text', 'frontmatter'];
		for (const call of mockTrack.mock.calls) {
			const props = call[1] as Record<string, unknown>;
			for (const key of forbidden) {
				expect(props).not.toHaveProperty(key);
			}
		}
	});
});
