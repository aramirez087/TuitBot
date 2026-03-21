/**
 * TweetActionBar.test.ts — Unit tests for TweetActionBar.svelte
 *
 * Tests: renders icons, compact mode styling.
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import TweetActionBar from '$lib/components/composer/TweetActionBar.svelte';

describe('TweetActionBar', () => {
	it('renders without crashing', () => {
		const { container } = render(TweetActionBar);
		expect(container).toBeTruthy();
	});

	it('renders five action items', () => {
		const { container } = render(TweetActionBar);
		const items = container.querySelectorAll('.action-item');
		expect(items.length).toBe(5);
	});

	it('is hidden from screen readers (aria-hidden)', () => {
		const { container } = render(TweetActionBar);
		const bar = container.querySelector('.action-bar');
		expect(bar?.getAttribute('aria-hidden')).toBe('true');
	});

	it('applies compact class when compact=true', () => {
		const { container } = render(TweetActionBar, { props: { compact: true } });
		const bar = container.querySelector('.action-bar');
		expect(bar?.classList.contains('compact')).toBe(true);
	});

	it('does not apply compact class by default', () => {
		const { container } = render(TweetActionBar);
		const bar = container.querySelector('.action-bar');
		expect(bar?.classList.contains('compact')).toBe(false);
	});
});
