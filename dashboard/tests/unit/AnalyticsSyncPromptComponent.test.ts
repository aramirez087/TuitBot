/**
 * AnalyticsSyncPromptComponent.test.ts — Unit tests for AnalyticsSyncPrompt.svelte
 *
 * Tests the component rendering, button callbacks, and analytics tracking.
 * Note: analyticsSyncPrompt.test.ts tests the STORE, this tests the COMPONENT.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import AnalyticsSyncPrompt from '$lib/components/settings/AnalyticsSyncPrompt.svelte';

vi.mock('$lib/analytics/hookMinerFunnel', () => ({
	trackForgePromptShown: vi.fn(),
	trackForgeEnabled: vi.fn(),
}));

import { trackForgePromptShown, trackForgeEnabled } from '$lib/analytics/hookMinerFunnel';

const defaultProps = {
	sourcePathStem: 'my-article',
	localEligible: true,
	onEnable: vi.fn(),
	onDismiss: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('AnalyticsSyncPrompt', () => {
	// --- Rendering ---

	it('renders without crashing', () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		expect(container.querySelector('.sync-prompt')).toBeTruthy();
	});

	it('has role="status" on container', () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		expect(container.querySelector('[role="status"]')).toBeTruthy();
	});

	it('shows "Enable Analytics Sync?" title', () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		const title = container.querySelector('.sync-prompt-title');
		expect(title?.textContent).toBe('Enable Analytics Sync?');
	});

	it('shows descriptive body text', () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		expect(container.textContent).toContain('engagement metrics');
		expect(container.textContent).toContain('local-only');
	});

	// --- Buttons ---

	it('shows "Enable in Settings" button', () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		const enableBtn = container.querySelector('.btn-enable');
		expect(enableBtn?.textContent?.trim()).toBe('Enable in Settings');
	});

	it('shows "Not now" button', () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		const dismissBtn = container.querySelector('.btn-dismiss');
		expect(dismissBtn?.textContent?.trim()).toBe('Not now');
	});

	// --- Button callbacks ---

	it('"Enable in Settings" button calls trackForgeEnabled then onEnable', async () => {
		const onEnable = vi.fn();
		const { container } = render(AnalyticsSyncPrompt, {
			props: { ...defaultProps, onEnable }
		});
		const enableBtn = container.querySelector('.btn-enable') as HTMLButtonElement;
		await fireEvent.click(enableBtn);
		expect(trackForgeEnabled).toHaveBeenCalledWith('my-article', 'prompt');
		expect(onEnable).toHaveBeenCalled();
	});

	it('"Not now" button calls onDismiss', async () => {
		const onDismiss = vi.fn();
		const { container } = render(AnalyticsSyncPrompt, {
			props: { ...defaultProps, onDismiss }
		});
		const dismissBtn = container.querySelector('.btn-dismiss') as HTMLButtonElement;
		await fireEvent.click(dismissBtn);
		expect(onDismiss).toHaveBeenCalled();
	});

	it('"Not now" does not call trackForgeEnabled', async () => {
		const { container } = render(AnalyticsSyncPrompt, { props: defaultProps });
		const dismissBtn = container.querySelector('.btn-dismiss') as HTMLButtonElement;
		await fireEvent.click(dismissBtn);
		expect(trackForgeEnabled).not.toHaveBeenCalled();
	});

	// --- Analytics tracking ---

	it('calls trackForgePromptShown on mount', () => {
		render(AnalyticsSyncPrompt, { props: defaultProps });
		expect(trackForgePromptShown).toHaveBeenCalledWith('my-article', true);
	});

	it('calls trackForgePromptShown with localEligible=false', () => {
		render(AnalyticsSyncPrompt, {
			props: { ...defaultProps, localEligible: false }
		});
		expect(trackForgePromptShown).toHaveBeenCalledWith('my-article', false);
	});

	it('uses default props when not provided', () => {
		render(AnalyticsSyncPrompt, {
			props: {
				onEnable: vi.fn(),
				onDismiss: vi.fn(),
			}
		});
		expect(trackForgePromptShown).toHaveBeenCalledWith('unknown', true);
	});
});
