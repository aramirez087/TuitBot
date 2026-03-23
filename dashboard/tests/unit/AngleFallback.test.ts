/**
 * AngleFallback.test.ts — Unit tests for AngleFallback.svelte
 *
 * Tests: heading text for each reason variant, body visibility,
 * button labels, button callbacks, and analytics tracking.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import AngleFallback from '$lib/components/composer/AngleFallback.svelte';

vi.mock('$lib/analytics/hookMinerFunnel', () => ({
	trackFallbackOpened: vi.fn(),
}));

import { trackFallbackOpened } from '$lib/analytics/hookMinerFunnel';

const defaultProps = {
	reason: 'weak_signal',
	sessionId: 'sess-abc',
	acceptedCount: 3,
	onusegenerichooks: vi.fn(),
	onbacktoneighbors: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('AngleFallback', () => {
	// --- Rendering ---

	it('renders without crashing', () => {
		const { container } = render(AngleFallback, { props: defaultProps });
		expect(container.querySelector('.angle-fallback')).toBeTruthy();
	});

	it('has role="status" on container', () => {
		const { container } = render(AngleFallback, { props: defaultProps });
		expect(container.querySelector('[role="status"]')).toBeTruthy();
	});

	// --- Heading text per reason ---

	it('shows "NOT ENOUGH SIGNAL" heading for weak_signal reason', () => {
		const { container } = render(AngleFallback, { props: defaultProps });
		const heading = container.querySelector('.angle-fallback-heading');
		expect(heading?.textContent).toBe('NOT ENOUGH SIGNAL');
	});

	it('shows timeout heading for timeout reason', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: 'timeout' }
		});
		const heading = container.querySelector('.angle-fallback-heading');
		expect(heading?.textContent).toContain('Mining took too long');
	});

	it('shows parse_error heading for parse_error reason', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: 'parse_error' }
		});
		const heading = container.querySelector('.angle-fallback-heading');
		expect(heading?.textContent).toContain("Couldn't parse mined angles");
	});

	it('shows default heading when reason is undefined', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: undefined }
		});
		const heading = container.querySelector('.angle-fallback-heading');
		expect(heading?.textContent).toBe('NOT ENOUGH SIGNAL');
	});

	// --- Body visibility ---

	it('shows body text for weak_signal reason', () => {
		const { container } = render(AngleFallback, { props: defaultProps });
		const body = container.querySelector('.angle-fallback-body');
		expect(body).toBeTruthy();
		expect(body?.textContent).toContain("didn't surface enough evidence");
	});

	it('hides body text for timeout reason', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: 'timeout' }
		});
		const body = container.querySelector('.angle-fallback-body');
		expect(body).toBeFalsy();
	});

	it('hides body text for parse_error reason', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: 'parse_error' }
		});
		const body = container.querySelector('.angle-fallback-body');
		expect(body).toBeFalsy();
	});

	// --- Button labels ---

	it('primary button says "Use generic hooks"', () => {
		const { container } = render(AngleFallback, { props: defaultProps });
		const primary = container.querySelector('.angle-fallback-primary');
		expect(primary?.textContent).toBe('Use generic hooks');
	});

	it('secondary button says "Back to related notes" for weak_signal', () => {
		const { container } = render(AngleFallback, { props: defaultProps });
		const secondary = container.querySelector('.angle-fallback-secondary');
		expect(secondary?.textContent).toContain('Back to related notes');
	});

	it('secondary button says "Mine again" for timeout', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: 'timeout' }
		});
		const secondary = container.querySelector('.angle-fallback-secondary');
		expect(secondary?.textContent).toBe('Mine again');
	});

	it('secondary button says "Mine again" for parse_error', () => {
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, reason: 'parse_error' }
		});
		const secondary = container.querySelector('.angle-fallback-secondary');
		expect(secondary?.textContent).toBe('Mine again');
	});

	// --- Button callbacks ---

	it('primary button calls onusegenerichooks', async () => {
		const onusegenerichooks = vi.fn();
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, onusegenerichooks }
		});
		const primary = container.querySelector('.angle-fallback-primary') as HTMLButtonElement;
		await fireEvent.click(primary);
		expect(onusegenerichooks).toHaveBeenCalled();
	});

	it('secondary button calls onbacktoneighbors', async () => {
		const onbacktoneighbors = vi.fn();
		const { container } = render(AngleFallback, {
			props: { ...defaultProps, onbacktoneighbors }
		});
		const secondary = container.querySelector('.angle-fallback-secondary') as HTMLButtonElement;
		await fireEvent.click(secondary);
		expect(onbacktoneighbors).toHaveBeenCalled();
	});

	// --- Analytics tracking ---

	it('calls trackFallbackOpened on mount with reason, sessionId, acceptedCount', () => {
		render(AngleFallback, { props: defaultProps });
		expect(trackFallbackOpened).toHaveBeenCalledWith('weak_signal', 'sess-abc', 3);
	});

	it('calls trackFallbackOpened with "weak_signal" when reason is undefined', () => {
		render(AngleFallback, {
			props: { ...defaultProps, reason: undefined }
		});
		expect(trackFallbackOpened).toHaveBeenCalledWith('weak_signal', 'sess-abc', 3);
	});

	it('calls trackFallbackOpened with defaults for sessionId and acceptedCount', () => {
		render(AngleFallback, {
			props: {
				onusegenerichooks: vi.fn(),
				onbacktoneighbors: vi.fn(),
			}
		});
		expect(trackFallbackOpened).toHaveBeenCalledWith('weak_signal', 'unknown', 0);
	});
});
