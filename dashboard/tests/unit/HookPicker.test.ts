/**
 * HookPicker.test.ts — Unit tests for HookPicker.svelte
 *
 * Tests: card rendering, selection, confirm, regenerate, back, format toggle,
 * loading state, error state, confidence badges.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import HookPicker from '$lib/components/composer/HookPicker.svelte';
import type { HookOption } from '$lib/api/types';

const sampleHooks: HookOption[] = [
	{ style: 'question', text: 'What if your tests could write themselves?', char_count: 42, confidence: 'high' },
	{ style: 'contrarian_take', text: 'Most devs test too much. Here\'s why.', char_count: 36, confidence: 'high' },
	{ style: 'tip', text: 'One simple testing trick that saves hours.', char_count: 43, confidence: 'high' },
	{ style: 'storytelling', text: 'I spent 3 days debugging. Then I wrote one test.', char_count: 49, confidence: 'medium' },
	{ style: 'list', text: '5 testing patterns every dev should know:', char_count: 42, confidence: 'high' },
];

const defaultProps = {
	hooks: sampleHooks,
	outputFormat: 'tweet' as const,
	loading: false,
	error: null as string | null,
	onselect: vi.fn(),
	onregenerate: vi.fn(),
	onback: vi.fn(),
	onformatchange: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('HookPicker', () => {
	it('renders without crashing', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		expect(container.querySelector('.hook-picker')).toBeTruthy();
	});

	it('renders "Choose a Hook" header', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const label = container.querySelector('.hook-label');
		expect(label?.textContent).toContain('Choose a Hook');
	});

	it('renders all 5 hook cards', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const cards = container.querySelectorAll('.hook-card');
		expect(cards.length).toBe(5);
	});

	it('renders style labels on hook cards', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const pills = container.querySelectorAll('.hook-style-pill');
		expect(pills[0]?.textContent).toBe('Question');
		expect(pills[1]?.textContent).toBe('Hot Take');
		expect(pills[2]?.textContent).toBe('Quick Tip');
		expect(pills[3]?.textContent).toBe('Story');
		expect(pills[4]?.textContent).toBe('List');
	});

	it('renders hook text content', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const texts = container.querySelectorAll('.hook-text');
		expect(texts[0]?.textContent).toBe('What if your tests could write themselves?');
	});

	it('renders char count on cards', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const counts = container.querySelectorAll('.hook-char-count');
		expect(counts[0]?.textContent).toContain('42');
	});

	it('renders confidence badges correctly', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const badges = container.querySelectorAll('.hook-confidence');
		expect(badges[0]?.textContent).toBe('Strong');
		expect(badges[0]?.classList.contains('confidence-high')).toBe(true);
		expect(badges[3]?.textContent).toBe('Good');
		expect(badges[3]?.classList.contains('confidence-medium')).toBe(true);
	});

	it('selects a hook card on click', async () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[1]);
		expect(cards[1].classList.contains('selected')).toBe(true);
		expect(cards[1].getAttribute('aria-selected')).toBe('true');
	});

	it('deselects previous card when selecting a new one', async () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		expect(cards[0].classList.contains('selected')).toBe(true);
		await fireEvent.click(cards[2]);
		expect(cards[0].classList.contains('selected')).toBe(false);
		expect(cards[2].classList.contains('selected')).toBe(true);
	});

	it('confirm button is disabled until selection', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		expect(confirmBtn?.disabled).toBe(true);
	});

	it('confirm button becomes enabled after selection', async () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		expect(confirmBtn?.disabled).toBe(false);
	});

	it('fires onselect with hook and format on confirm', async () => {
		const onselect = vi.fn();
		const { container } = render(HookPicker, { props: { ...defaultProps, onselect } });
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[2]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		expect(onselect).toHaveBeenCalledWith(sampleHooks[2], 'tweet');
	});

	it('fires onselect with thread format when toggled', async () => {
		const onselect = vi.fn();
		const { container } = render(HookPicker, {
			props: { ...defaultProps, outputFormat: 'thread' as const, onselect }
		});
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		expect(onselect).toHaveBeenCalledWith(sampleHooks[0], 'thread');
	});

	it('fires onregenerate when Regenerate is clicked', async () => {
		const onregenerate = vi.fn();
		const { container } = render(HookPicker, { props: { ...defaultProps, onregenerate } });
		const regenBtn = container.querySelector('.hook-regen-btn') as HTMLButtonElement;
		await fireEvent.click(regenBtn);
		expect(onregenerate).toHaveBeenCalled();
	});

	it('fires onback when back button is clicked', async () => {
		const onback = vi.fn();
		const { container } = render(HookPicker, { props: { ...defaultProps, onback } });
		const backBtn = container.querySelector('.hook-back') as HTMLButtonElement;
		await fireEvent.click(backBtn);
		expect(onback).toHaveBeenCalled();
	});

	it('fires onformatchange when format toggle is clicked', async () => {
		const onformatchange = vi.fn();
		const { container } = render(HookPicker, { props: { ...defaultProps, onformatchange } });
		const threadBtn = container.querySelectorAll('.hook-format-opt')[1] as HTMLButtonElement;
		await fireEvent.click(threadBtn);
		expect(onformatchange).toHaveBeenCalledWith('thread');
	});

	it('shows shimmer cards in loading state', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, loading: true, hooks: [] }
		});
		const shimmers = container.querySelectorAll('.hook-card-shimmer');
		expect(shimmers.length).toBe(5);
		const cards = container.querySelectorAll('.hook-card');
		expect(cards.length).toBe(0);
	});

	it('shows error message when error is set', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, error: 'Something went wrong' }
		});
		const errorEl = container.querySelector('.hook-error');
		expect(errorEl?.textContent).toContain('Something went wrong');
	});

	it('shows retry button in error state', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, error: 'Network error' }
		});
		const retryBtn = container.querySelector('.hook-retry-btn');
		expect(retryBtn?.textContent).toContain('Retry');
	});

	it('retry button fires onregenerate', async () => {
		const onregenerate = vi.fn();
		const { container } = render(HookPicker, {
			props: { ...defaultProps, error: 'Network error', onregenerate }
		});
		const retryBtn = container.querySelector('.hook-retry-btn') as HTMLButtonElement;
		await fireEvent.click(retryBtn);
		expect(onregenerate).toHaveBeenCalled();
	});

	it('disables regenerate button during loading', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, loading: true, hooks: [] }
		});
		const regenBtn = container.querySelector('.hook-regen-btn') as HTMLButtonElement;
		expect(regenBtn?.disabled).toBe(true);
	});

	it('disables confirm button during loading', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, loading: true, hooks: [] }
		});
		const confirmBtn = container.querySelector('.hook-confirm-btn') as HTMLButtonElement;
		expect(confirmBtn?.disabled).toBe(true);
	});

	it('shows active format state correctly', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, outputFormat: 'tweet' as const }
		});
		const opts = container.querySelectorAll('.hook-format-opt');
		expect(opts[0].classList.contains('active')).toBe(true);
		expect(opts[1].classList.contains('active')).toBe(false);
	});

	it('renders format toggle with tweet and thread options', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const opts = container.querySelectorAll('.hook-format-opt');
		expect(opts.length).toBe(2);
		expect(opts[0]?.textContent).toBe('Tweet');
		expect(opts[1]?.textContent).toBe('Thread');
	});

	it('has proper aria attributes on card list', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const list = container.querySelector('[role="listbox"]');
		expect(list).toBeTruthy();
		expect(list?.getAttribute('aria-label')).toBe('Hook options');
	});

	it('has proper aria-label on back button', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const backBtn = container.querySelector('.hook-back');
		expect(backBtn?.getAttribute('aria-label')).toBe('Back to highlights');
	});

	it('confirm button text says "Use this hook"', () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const confirmBtn = container.querySelector('.hook-confirm-btn');
		expect(confirmBtn?.textContent).toContain('Use this hook');
	});

	it('supports keyboard selection with Enter', async () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.keyDown(cards[1], { key: 'Enter' });
		expect(cards[1].classList.contains('selected')).toBe(true);
	});

	it('supports keyboard selection with Space', async () => {
		const { container } = render(HookPicker, { props: defaultProps });
		const cards = container.querySelectorAll('.hook-card');
		await fireEvent.keyDown(cards[2], { key: ' ' });
		expect(cards[2].classList.contains('selected')).toBe(true);
	});

	it('renders with empty hooks array', () => {
		const { container } = render(HookPicker, {
			props: { ...defaultProps, hooks: [] }
		});
		const cards = container.querySelectorAll('.hook-card');
		expect(cards.length).toBe(0);
	});
});
