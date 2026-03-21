/**
 * ComposerPreviewSurface.test.ts — Unit tests for ComposerPreviewSurface.svelte
 *
 * Tests: renders preview, device toggle switches container.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ComposerPreviewSurface from '$lib/components/composer/ComposerPreviewSurface.svelte';

const defaultProps = {
	mode: 'tweet' as const,
	tweetText: 'Hello world',
	blocks: [],
	tweetMediaPaths: [],
	tweetLocalPreviews: new Map<string, string>(),
	handle: '@testuser',
	avatarUrl: null,
	onclose: vi.fn()
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('ComposerPreviewSurface', () => {
	it('renders without crashing', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders Preview title', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const title = container.querySelector('.preview-title');
		expect(title?.textContent).toBe('Preview');
	});

	it('renders device toggle with desktop and mobile buttons', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const toggle = container.querySelector('.device-toggle');
		expect(toggle).toBeTruthy();
		const buttons = toggle?.querySelectorAll('.device-btn');
		expect(buttons?.length).toBe(2);
	});

	it('starts in desktop mode by default', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const desktopBtn = container.querySelector('[aria-label="Desktop preview"]');
		expect(desktopBtn?.classList.contains('active')).toBe(true);
	});

	it('does not have preview-mobile class by default', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const previewContainer = container.querySelector('.preview-container');
		expect(previewContainer?.classList.contains('preview-mobile')).toBe(false);
	});

	it('switches to mobile mode when mobile button is clicked', async () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const mobileBtn = container.querySelector('[aria-label="Mobile preview"]') as HTMLButtonElement;
		await fireEvent.click(mobileBtn);
		const previewContainer = container.querySelector('.preview-container');
		expect(previewContainer?.classList.contains('preview-mobile')).toBe(true);
	});

	it('renders close button', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const closeBtn = container.querySelector('.preview-close');
		expect(closeBtn).toBeTruthy();
	});

	it('calls onclose when close button clicked', async () => {
		const onclose = vi.fn();
		const { container } = render(ComposerPreviewSurface, {
			props: { ...defaultProps, onclose }
		});
		const closeBtn = container.querySelector('.preview-close') as HTMLButtonElement;
		await fireEvent.click(closeBtn);
		expect(onclose).toHaveBeenCalled();
	});

	it('shows tweet content in tweet mode', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		expect(container.textContent).toContain('Hello world');
	});

	it('shows empty message when no content', () => {
		const { container } = render(ComposerPreviewSurface, {
			props: { ...defaultProps, tweetText: '', tweetMediaPaths: [] }
		});
		expect(container.textContent).toContain('Nothing to preview');
	});

	it('renders thread blocks in thread mode', () => {
		const blocks = [
			{ id: '1', text: 'Thread tweet 1', media_paths: [] },
			{ id: '2', text: 'Thread tweet 2', media_paths: [] }
		];
		const { container } = render(ComposerPreviewSurface, {
			props: { ...defaultProps, mode: 'thread' as const, blocks }
		});
		expect(container.textContent).toContain('Thread tweet 1');
		expect(container.textContent).toContain('Thread tweet 2');
	});

	it('has proper aria role for dialog', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const dialog = container.querySelector('[role="dialog"]');
		expect(dialog).toBeTruthy();
		expect(dialog?.getAttribute('aria-modal')).toBe('true');
	});

	it('device toggle has radiogroup role', () => {
		const { container } = render(ComposerPreviewSurface, { props: defaultProps });
		const toggle = container.querySelector('[role="radiogroup"]');
		expect(toggle).toBeTruthy();
	});
});
