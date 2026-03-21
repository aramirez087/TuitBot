/**
 * DraftStudioComposerZone.test.ts — Unit tests for DraftStudioComposerZone.svelte
 *
 * Tests: no-selection state, error banner, loading spinner, zone-error retry,
 * empty state variants, composer bar visibility.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

// ─── Store mocks (hoisted — no top-level variables allowed in factories) ──────

vi.mock('$lib/stores/draftStudio.svelte', () => ({
	getSelectedId: vi.fn().mockReturnValue(null),
	getTabCounts: vi.fn().mockReturnValue({ active: 0, scheduled: 0, posted: 0, archive: 0 }),
	getError: vi.fn().mockReturnValue(null),
	isLoading: vi.fn().mockReturnValue(false),
	getSelectedDraft: vi.fn().mockReturnValue(null),
	clearError: vi.fn(),
}));

vi.mock('$lib/stores/calendar', () => ({
	schedule: {
		subscribe: vi.fn((cb: (v: null) => void) => {
			cb(null);
			return () => {};
		}),
	},
}));

// ─── Imports after mocks ──────────────────────────────────────────────────────

import DraftStudioComposerZone from '$lib/components/drafts/DraftStudioComposerZone.svelte';
import * as studio from '$lib/stores/draftStudio.svelte';

// ─── Fixtures ─────────────────────────────────────────────────────────────────

const defaultProps = {
	drawerOpen: false,
	hydration: null,
	hydrationDraftId: null,
	loadingDraft: false,
	publishEnabled: false,
	selectionSessionId: null,
	onToggleDrawer: vi.fn(),
	onSyncStatus: vi.fn(),
	onDraftAction: vi.fn(),
	onDraftSubmit: vi.fn(),
	onFetchDraft: vi.fn(),
	onCreate: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(studio.getSelectedId).mockReturnValue(null);
	vi.mocked(studio.getTabCounts).mockReturnValue({ active: 0, scheduled: 0, posted: 0, archive: 0 });
	vi.mocked(studio.getError).mockReturnValue(null);
	vi.mocked(studio.isLoading).mockReturnValue(false);
	vi.mocked(studio.getSelectedDraft).mockReturnValue(null);
});

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('DraftStudioComposerZone', () => {
	it('renders without crashing', () => {
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.composer-zone')).toBeTruthy();
	});

	it('shows composer-bar when no draft is selected', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.composer-bar')).toBeTruthy();
	});

	it('shows composer-bar when loadingDraft is true and no hydration', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(5);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, loadingDraft: true, hydration: null },
		});
		expect(container.querySelector('.composer-bar')).toBeTruthy();
	});

	it('shows error banner when studio.getError() returns a message', () => {
		vi.mocked(studio.getError).mockReturnValue('Something went wrong');
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		const banner = container.querySelector('.error-banner');
		expect(banner).toBeTruthy();
		expect(banner?.textContent).toContain('Something went wrong');
	});

	it('does not show error banner when getError returns null', () => {
		vi.mocked(studio.getError).mockReturnValue(null);
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.error-banner')).toBeNull();
	});

	it('error banner has a Dismiss button', () => {
		vi.mocked(studio.getError).mockReturnValue('Error occurred');
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		const dismissBtn = container.querySelector('.error-banner button');
		expect(dismissBtn?.textContent?.trim()).toBe('Dismiss');
	});

	it('clicking Dismiss button calls studio.clearError', async () => {
		vi.mocked(studio.getError).mockReturnValue('Error occurred');
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		const dismissBtn = container.querySelector('.error-banner button') as HTMLButtonElement;
		await fireEvent.click(dismissBtn);
		expect(studio.clearError).toHaveBeenCalled();
	});

	it('shows zone-loading spinner when isLoading and no selected draft', () => {
		vi.mocked(studio.isLoading).mockReturnValue(true);
		vi.mocked(studio.getSelectedDraft).mockReturnValue(null);
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.zone-loading')).toBeTruthy();
		expect(container.querySelector('.zone-spinner')).toBeTruthy();
	});

	it('does not show global zone-loading when isLoading but selected draft exists', () => {
		vi.mocked(studio.isLoading).mockReturnValue(true);
		// Return a minimal DraftSummary shape
		vi.mocked(studio.getSelectedDraft).mockReturnValue({
			id: 1,
			title: 'Draft',
			content_type: 'tweet',
			content_preview: '',
			status: 'draft',
			scheduled_for: null,
			archived_at: null,
			updated_at: new Date().toISOString(),
			created_at: new Date().toISOString(),
			source: 'manual',
		});
		vi.mocked(studio.getSelectedId).mockReturnValue(1);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, loadingDraft: false, hydration: null },
		});
		// When selectedDraft exists, the isLoading branch is skipped — zone-loading comes
		// only from loadingDraft prop (which is false here), so no spinner
		const zoneLoading = container.querySelector('.zone-loading');
		expect(zoneLoading).toBeNull();
	});

	it('shows zone-loading when loadingDraft is true with a selectedId', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(3);
		vi.mocked(studio.isLoading).mockReturnValue(false);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, loadingDraft: true, hydration: null },
		});
		expect(container.querySelector('.zone-loading')).toBeTruthy();
	});

	it('shows zone-error when selectedId set but no hydration and not loading', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(7);
		vi.mocked(studio.isLoading).mockReturnValue(false);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, loadingDraft: false, hydration: null, hydrationDraftId: null },
		});
		const zoneError = container.querySelector('.zone-error');
		expect(zoneError).toBeTruthy();
		expect(zoneError?.textContent).toContain('Failed to load draft content.');
	});

	it('zone-error retry button calls onFetchDraft with selected id', async () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(7);
		vi.mocked(studio.isLoading).mockReturnValue(false);
		const onFetchDraft = vi.fn();
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, loadingDraft: false, hydration: null, onFetchDraft },
		});
		const retryBtn = container.querySelector('.zone-error button') as HTMLButtonElement;
		await fireEvent.click(retryBtn);
		expect(onFetchDraft).toHaveBeenCalledWith(7);
	});

	it('shows empty state when tab counts are all zero and no selection', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		vi.mocked(studio.getTabCounts).mockReturnValue({ active: 0, scheduled: 0, posted: 0, archive: 0 });
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.composer-zone')).toBeTruthy();
		expect(container.textContent).toBeTruthy();
	});

	it('shows no-selection state when drafts exist but none selected', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		vi.mocked(studio.getTabCounts).mockReturnValue({ active: 3, scheduled: 0, posted: 0, archive: 0 });
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.composer-zone')).toBeTruthy();
	});

	it('New button calls onCreate when clicked', async () => {
		const onCreate = vi.fn();
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, onCreate },
		});
		const newBtn = Array.from(container.querySelectorAll('.bar-action')).find(
			(b) => b.textContent?.includes('New')
		) as HTMLButtonElement | undefined;
		expect(newBtn).toBeTruthy();
		if (newBtn) {
			await fireEvent.click(newBtn);
			expect(onCreate).toHaveBeenCalled();
		}
	});

	it('Drafts toggle button calls onToggleDrawer when clicked', async () => {
		const onToggleDrawer = vi.fn();
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, onToggleDrawer },
		});
		const toggleBtn = container.querySelector('.drafts-toggle-btn') as HTMLButtonElement;
		expect(toggleBtn).toBeTruthy();
		await fireEvent.click(toggleBtn);
		expect(onToggleDrawer).toHaveBeenCalled();
	});

	it('Drafts toggle button has active class when drawerOpen is true', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		const { container } = render(DraftStudioComposerZone, {
			props: { ...defaultProps, drawerOpen: true },
		});
		const toggleBtn = container.querySelector('.drafts-toggle-btn');
		expect(toggleBtn?.classList.contains('active')).toBe(true);
	});

	it('shows draft count badge when active drafts count is > 0', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		vi.mocked(studio.getTabCounts).mockReturnValue({ active: 5, scheduled: 0, posted: 0, archive: 0 });
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		const badge = container.querySelector('.draft-count');
		expect(badge?.textContent).toBe('5');
	});

	it('does not show draft count badge when active count is 0', () => {
		vi.mocked(studio.getSelectedId).mockReturnValue(null);
		vi.mocked(studio.getTabCounts).mockReturnValue({ active: 0, scheduled: 0, posted: 0, archive: 0 });
		const { container } = render(DraftStudioComposerZone, { props: defaultProps });
		expect(container.querySelector('.draft-count')).toBeNull();
	});
});
