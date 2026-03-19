/**
 * DiscoveryTweetPreview.test.ts — Smoke tests for preview component
 */

import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import DiscoveryTweetPreview from '../../src/routes/(app)/discovery/DiscoveryTweetPreview.svelte';
import type { DiscoveredTweet } from '../../src/routes/(app)/discovery/DiscoveryTweetCard.svelte';

const makeTweet = (id: string = '1'): DiscoveredTweet => ({
	id,
	author_username: 'author',
	content: 'Tweet content',
	relevance_score: 75,
	matched_keyword: null,
	like_count: 100,
	retweet_count: 50,
	reply_count: 10,
	replied_to: false,
	discovered_at: '2026-03-19T22:00:00Z'
});

describe('DiscoveryTweetPreview', () => {
	it('renders nothing when tweet is null', () => {
		const { container } = render(DiscoveryTweetPreview, {
			props: { tweet: null, allTweets: [], onClose: vi.fn(), onSelectTweet: vi.fn() }
		});
		expect(container.querySelector('.preview-overlay')).toBeFalsy();
	});

	it('renders preview pane when tweet is provided', () => {
		const tweet = makeTweet();
		const { container } = render(DiscoveryTweetPreview, {
			props: { tweet, allTweets: [tweet], onClose: vi.fn(), onSelectTweet: vi.fn() }
		});
		expect(container.querySelector('.preview-pane')).toBeTruthy();
	});

	it('displays tweet content', () => {
		const tweet = makeTweet();
		const { container } = render(DiscoveryTweetPreview, {
			props: { tweet, allTweets: [tweet], onClose: vi.fn(), onSelectTweet: vi.fn() }
		});
		expect(container.textContent).toContain('Tweet content');
		expect(container.textContent).toContain('@author');
	});

	it('closes on Escape key', async () => {
		const onClose = vi.fn();
		const tweet = makeTweet();
		render(DiscoveryTweetPreview, {
			props: { tweet, allTweets: [tweet], onClose, onSelectTweet: vi.fn() }
		});
		await fireEvent.keyDown(window, { key: 'Escape' });
		expect(onClose).toHaveBeenCalled();
	});

	it('navigates with arrow keys', async () => {
		const onSelectTweet = vi.fn();
		const tweets = [makeTweet('1'), makeTweet('2')];
		render(DiscoveryTweetPreview, {
			props: { tweet: tweets[0], allTweets: tweets, onClose: vi.fn(), onSelectTweet }
		});
		await fireEvent.keyDown(window, { key: 'ArrowDown' });
		expect(onSelectTweet).toHaveBeenCalledWith(tweets[1]);
	});

	it('calls onClose when close button clicked', async () => {
		const onClose = vi.fn();
		const tweet = makeTweet();
		const { container } = render(DiscoveryTweetPreview, {
			props: { tweet, allTweets: [tweet], onClose, onSelectTweet: vi.fn() }
		});
		const closeBtn = container.querySelector('.close-btn') as HTMLButtonElement;
		await fireEvent.click(closeBtn);
		expect(onClose).toHaveBeenCalled();
	});

	it('has correct role and aria attributes', () => {
		const tweet = makeTweet();
		const { container } = render(DiscoveryTweetPreview, {
			props: { tweet, allTweets: [tweet], onClose: vi.fn(), onSelectTweet: vi.fn() }
		});
		const pane = container.querySelector('[role="region"]');
		expect(pane?.getAttribute('aria-labelledby')).toBe('preview-title');
	});

	it('shows engagement stats', () => {
		const tweet = makeTweet();
		const { container } = render(DiscoveryTweetPreview, {
			props: { tweet, allTweets: [tweet], onClose: vi.fn(), onSelectTweet: vi.fn() }
		});
		expect(container.textContent).toContain('100'); // likes
		expect(container.textContent).toContain('50'); // retweets
	});
});
