/**
 * AngleCards.test.ts — Unit tests for AngleCards.svelte
 *
 * Tests: card rendering, selection, confirm, remine, fallback action,
 * format toggle, loading state, error state, evidence badges, accessibility.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import AngleCards from '$lib/components/composer/AngleCards.svelte';
import type { MinedAngle } from '$lib/api/types';

const sampleAngles: MinedAngle[] = [
	{
		angle_type: 'story',
		seed_text: 'I spent 3 months migrating from X to Y. Here\'s what nobody tells you.',
		char_count: 68,
		evidence: [
			{
				evidence_type: 'data_point',
				citation_text: 'migration cost 3.2x the initial estimate',
				source_node_id: 42,
				source_note_title: 'Migration Retrospective',
				source_heading_path: '# Costs',
			},
			{
				evidence_type: 'contradiction',
				citation_text: 'vendor claimed 2-week migration window',
				source_node_id: 57,
				source_note_title: 'Vendor Evaluation Notes',
			},
		],
		confidence: 'high',
		rationale: 'Tension between growth and cost creates a compelling narrative arc.',
	},
	{
		angle_type: 'listicle',
		seed_text: '5 hidden costs of platform migration that nobody warns you about:',
		char_count: 64,
		evidence: [
			{
				evidence_type: 'aha_moment',
				citation_text: 'team velocity dropped 40% during migration',
				source_node_id: 42,
				source_note_title: 'Migration Retrospective',
			},
		],
		confidence: 'medium',
		rationale: 'Multiple data points suit a list format.',
	},
	{
		angle_type: 'hot_take',
		seed_text: 'Platform migrations are a scam. Most vendors know their timeline estimates are fiction.',
		char_count: 82,
		evidence: [
			{
				evidence_type: 'contradiction',
				citation_text: 'actual timeline was 4x the estimate',
				source_node_id: 57,
				source_note_title: 'Vendor Evaluation Notes',
			},
		],
		confidence: 'high',
		rationale: 'Strong contradiction supports a bold claim.',
	},
];

const defaultProps = {
	angles: sampleAngles,
	outputFormat: 'tweet' as const,
	loading: false,
	error: null as string | null,
	onselect: vi.fn(),
	onremine: vi.fn(),
	onback: vi.fn(),
	onfallback: vi.fn(),
	onformatchange: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('AngleCards', () => {
	it('renders without crashing', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		expect(container.querySelector('.angle-picker')).toBeTruthy();
	});

	it('renders "Mined Angles" header', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const label = container.querySelector('.angle-label');
		expect(label?.textContent).toContain('Mined Angles');
	});

	it('renders all 3 angle cards', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const cards = container.querySelectorAll('.angle-card');
		expect(cards.length).toBe(3);
	});

	it('renders angle type labels on cards', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const pills = container.querySelectorAll('.angle-type-pill');
		expect(pills[0]?.textContent).toBe('Story');
		expect(pills[1]?.textContent).toBe('Listicle');
		expect(pills[2]?.textContent).toBe('Hot Take');
	});

	it('renders seed text content', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const texts = container.querySelectorAll('.angle-seed-text');
		expect(texts[0]?.textContent).toContain('I spent 3 months migrating');
	});

	it('renders char count on cards', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const counts = container.querySelectorAll('.angle-char-count');
		expect(counts[0]?.textContent).toContain('68');
	});

	it('renders confidence badges correctly', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const badges = container.querySelectorAll('.angle-confidence');
		expect(badges[0]?.textContent).toBe('Strong');
		expect(badges[0]?.classList.contains('confidence-high')).toBe(true);
		expect(badges[1]?.textContent).toBe('Good');
		expect(badges[1]?.classList.contains('confidence-medium')).toBe(true);
	});

	it('renders evidence items with type pills', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const evidencePills = container.querySelectorAll('.angle-evidence-pill');
		expect(evidencePills.length).toBeGreaterThanOrEqual(2);
		expect(evidencePills[0]?.textContent).toBe('Data Point');
		expect(evidencePills[1]?.textContent).toBe('Contradiction');
	});

	it('renders source attribution', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const sources = container.querySelectorAll('.angle-evidence-source');
		expect(sources[0]?.textContent).toContain('Migration Retrospective');
	});

	it('renders rationale as title attribute on type pill', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const pill = container.querySelector('.angle-type-pill');
		expect(pill?.getAttribute('title')).toContain('Tension between growth');
	});

	it('selects a card on click', async () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.click(cards[1]);
		expect(cards[1].classList.contains('selected')).toBe(true);
		expect(cards[1].getAttribute('aria-selected')).toBe('true');
	});

	it('deselects previous card when selecting new one', async () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.click(cards[0]);
		expect(cards[0].classList.contains('selected')).toBe(true);
		await fireEvent.click(cards[2]);
		expect(cards[0].classList.contains('selected')).toBe(false);
		expect(cards[2].classList.contains('selected')).toBe(true);
	});

	it('confirm button is disabled until selection', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const confirmBtn = container.querySelector('.angle-confirm-btn') as HTMLButtonElement;
		expect(confirmBtn?.disabled).toBe(true);
	});

	it('confirm button becomes enabled after selection', async () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.angle-confirm-btn') as HTMLButtonElement;
		expect(confirmBtn?.disabled).toBe(false);
	});

	it('fires onselect with angle and format on confirm', async () => {
		const onselect = vi.fn();
		const { container } = render(AngleCards, { props: { ...defaultProps, onselect } });
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.click(cards[1]);
		const confirmBtn = container.querySelector('.angle-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		expect(onselect).toHaveBeenCalledWith(sampleAngles[1], 'tweet');
	});

	it('fires onselect with thread format when toggled', async () => {
		const onselect = vi.fn();
		const { container } = render(AngleCards, {
			props: { ...defaultProps, outputFormat: 'thread' as const, onselect }
		});
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.click(cards[0]);
		const confirmBtn = container.querySelector('.angle-confirm-btn') as HTMLButtonElement;
		await fireEvent.click(confirmBtn);
		expect(onselect).toHaveBeenCalledWith(sampleAngles[0], 'thread');
	});

	it('fires onremine when Mine again is clicked', async () => {
		const onremine = vi.fn();
		const { container } = render(AngleCards, { props: { ...defaultProps, onremine } });
		const remineBtn = container.querySelector('.angle-remine-btn') as HTMLButtonElement;
		await fireEvent.click(remineBtn);
		expect(onremine).toHaveBeenCalled();
	});

	it('fires onfallback when More hook styles is clicked', async () => {
		const onfallback = vi.fn();
		const { container } = render(AngleCards, { props: { ...defaultProps, onfallback } });
		const fallbackBtn = container.querySelector('.angle-fallback-btn') as HTMLButtonElement;
		await fireEvent.click(fallbackBtn);
		expect(onfallback).toHaveBeenCalled();
	});

	it('fires onback when back button is clicked', async () => {
		const onback = vi.fn();
		const { container } = render(AngleCards, { props: { ...defaultProps, onback } });
		const backBtn = container.querySelector('.angle-back') as HTMLButtonElement;
		await fireEvent.click(backBtn);
		expect(onback).toHaveBeenCalled();
	});

	it('fires onformatchange when format toggle is clicked', async () => {
		const onformatchange = vi.fn();
		const { container } = render(AngleCards, { props: { ...defaultProps, onformatchange } });
		const threadBtn = container.querySelectorAll('.angle-format-opt')[1] as HTMLButtonElement;
		await fireEvent.click(threadBtn);
		expect(onformatchange).toHaveBeenCalledWith('thread');
	});

	it('shows 3 shimmer cards in loading state', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, loading: true, angles: [] }
		});
		const shimmers = container.querySelectorAll('.angle-card-shimmer');
		expect(shimmers.length).toBe(3);
		const cards = container.querySelectorAll('.angle-card');
		expect(cards.length).toBe(0);
	});

	it('shows loading label in loading state', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, loading: true, angles: [] }
		});
		const label = container.querySelector('.angle-loading-label');
		expect(label?.textContent).toContain('Mining angles from your notes');
	});

	it('shows error message when error is set', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, error: 'Failed to mine angles' }
		});
		const errorEl = container.querySelector('.angle-error');
		expect(errorEl?.textContent).toContain('Failed to mine angles');
	});

	it('shows retry button in error state', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, error: 'Network error' }
		});
		const retryBtn = container.querySelector('.angle-retry-btn');
		expect(retryBtn?.textContent).toContain('Retry');
	});

	it('disables remine button during loading', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, loading: true, angles: [] }
		});
		const remineBtn = container.querySelector('.angle-remine-btn') as HTMLButtonElement;
		expect(remineBtn?.disabled).toBe(true);
	});

	it('disables confirm button during loading', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, loading: true, angles: [] }
		});
		const confirmBtn = container.querySelector('.angle-confirm-btn') as HTMLButtonElement;
		expect(confirmBtn?.disabled).toBe(true);
	});

	it('shows active format state correctly', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, outputFormat: 'tweet' as const }
		});
		const opts = container.querySelectorAll('.angle-format-opt');
		expect(opts[0].classList.contains('active')).toBe(true);
		expect(opts[1].classList.contains('active')).toBe(false);
	});

	it('has proper aria attributes on card list', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const list = container.querySelector('[role="listbox"]');
		expect(list).toBeTruthy();
		expect(list?.getAttribute('aria-label')).toBe('Mined angle options');
	});

	it('has proper aria-label on back button', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const backBtn = container.querySelector('.angle-back');
		expect(backBtn?.getAttribute('aria-label')).toBe('Back to related notes');
	});

	it('confirm button text says "Use this angle"', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const confirmBtn = container.querySelector('.angle-confirm-btn');
		expect(confirmBtn?.textContent).toContain('Use this angle');
	});

	it('supports keyboard selection with Enter', async () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.keyDown(cards[1], { key: 'Enter' });
		expect(cards[1].classList.contains('selected')).toBe(true);
	});

	it('supports keyboard selection with Space', async () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const cards = container.querySelectorAll('.angle-card');
		await fireEvent.keyDown(cards[2], { key: ' ' });
		expect(cards[2].classList.contains('selected')).toBe(true);
	});

	it('renders with empty angles array', () => {
		const { container } = render(AngleCards, {
			props: { ...defaultProps, angles: [] }
		});
		const cards = container.querySelectorAll('.angle-card');
		expect(cards.length).toBe(0);
	});

	it('renders evidence with aria-label referencing source title', () => {
		const { container } = render(AngleCards, { props: defaultProps });
		const evidenceItems = container.querySelectorAll('.angle-evidence-item');
		expect(evidenceItems[0]?.getAttribute('aria-label')).toBe('Evidence from Migration Retrospective');
	});
});
