/**
 * EvidenceCard.test.ts — Unit tests for EvidenceCard.svelte
 *
 * Tests: rendering states (pinned, suggested, unpinned), match reason badges,
 * truncation, actions (pin, unpin, dismiss, apply), slot picker dropdown,
 * conditional rendering branches, edge cases.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import EvidenceCard from '$lib/components/composer/EvidenceCard.svelte';
import type { EvidenceResult } from '$lib/api/types';

function makeResult(overrides: Partial<EvidenceResult> = {}): EvidenceResult {
	return {
		chunk_id: overrides.chunk_id ?? 1,
		node_id: overrides.node_id ?? 10,
		heading_path: overrides.heading_path ?? '# Test Heading',
		snippet: overrides.snippet ?? 'This is a test snippet for evidence card rendering.',
		match_reason: overrides.match_reason ?? 'semantic',
		score: overrides.score ?? 0.85,
		node_title: overrides.node_title ?? 'Test Note',
		relative_path: overrides.relative_path ?? 'notes/test.md',
	};
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe('EvidenceCard', () => {
	// ── Basic rendering ─────────────────────────────────
	it('renders without crashing', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult() },
		});
		expect(container.querySelector('.evidence-card')).toBeTruthy();
	});

	it('renders the snippet text', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ snippet: 'My unique snippet' }) },
		});
		const snippetEl = container.querySelector('.card-snippet');
		expect(snippetEl?.textContent).toContain('My unique snippet');
	});

	it('renders the heading path when present', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ heading_path: '# Deep Heading' }) },
		});
		const heading = container.querySelector('.card-heading');
		expect(heading?.textContent).toContain('Deep Heading');
	});

	it('hides heading when heading_path is empty', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ heading_path: '' }) },
		});
		const heading = container.querySelector('.card-heading');
		expect(heading).toBeNull();
	});

	it('renders node_title as source', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ node_title: 'My Note Title' }) },
		});
		const source = container.querySelector('.card-source');
		expect(source?.textContent).toContain('My Note Title');
	});

	it('hides source when node_title is null', () => {
		const result: EvidenceResult = {
			...makeResult(),
			node_title: null,
		};
		const { container } = render(EvidenceCard, {
			props: { result },
		});
		const source = container.querySelector('.card-source');
		expect(source).toBeNull();
	});

	it('renders the score as percentage', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ score: 0.923 }) },
		});
		const score = container.querySelector('.card-score');
		expect(score?.textContent).toContain('92');
	});

	it('has aria-label with heading path', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ heading_path: '# Aria Test' }) },
		});
		const card = container.querySelector('.evidence-card');
		expect(card?.getAttribute('aria-label')).toContain('Aria Test');
	});

	// ── Match reason badges ─────────────────────────────
	it('renders semantic badge for semantic match', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ match_reason: 'semantic' }) },
		});
		const badge = container.querySelector('.match-badge');
		expect(badge?.textContent).toBe('Semantic');
		expect(badge?.classList.contains('badge-semantic')).toBe(true);
	});

	it('renders keyword badge for keyword match', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ match_reason: 'keyword' }) },
		});
		const badge = container.querySelector('.match-badge');
		expect(badge?.textContent).toBe('Keyword');
		expect(badge?.classList.contains('badge-keyword')).toBe(true);
	});

	it('renders graph badge for graph match', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ match_reason: 'graph' }) },
		});
		const badge = container.querySelector('.match-badge');
		expect(badge?.textContent).toBe('Graph');
		expect(badge?.classList.contains('badge-graph')).toBe(true);
	});

	it('renders hybrid badge for hybrid match', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ match_reason: 'hybrid' }) },
		});
		const badge = container.querySelector('.match-badge');
		expect(badge?.textContent).toBe('Hybrid');
		expect(badge?.classList.contains('badge-hybrid')).toBe(true);
	});

	// ── Suggested badge ─────────────────────────────────
	it('shows suggested badge when suggested=true', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), suggested: true },
		});
		const suggestedBadge = container.querySelector('.suggested-badge');
		expect(suggestedBadge).not.toBeNull();
		expect(suggestedBadge?.textContent).toContain('Suggested');
	});

	it('hides suggested badge when suggested=false', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), suggested: false },
		});
		const suggestedBadge = container.querySelector('.suggested-badge');
		expect(suggestedBadge).toBeNull();
	});

	// ── Pinned state ────────────────────────────────────
	it('adds pinned class when pinned=true', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), pinned: true },
		});
		const card = container.querySelector('.evidence-card');
		expect(card?.classList.contains('pinned')).toBe(true);
	});

	it('shows unpin button when pinned', () => {
		const onunpin = vi.fn();
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), pinned: true, onunpin },
		});
		const unpinBtn = container.querySelector('[aria-label="Unpin"]');
		expect(unpinBtn).not.toBeNull();
	});

	it('shows pin button when not pinned', () => {
		const onpin = vi.fn();
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), pinned: false, onpin },
		});
		const pinBtn = container.querySelector('[aria-label="Pin"]');
		expect(pinBtn).not.toBeNull();
	});

	it('hides dismiss button when pinned', () => {
		const ondismiss = vi.fn();
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), pinned: true, ondismiss },
		});
		const dismissBtn = container.querySelector('[aria-label="Dismiss"]');
		expect(dismissBtn).toBeNull();
	});

	// ── Action callbacks ────────────────────────────────
	it('fires onpin when pin button is clicked', async () => {
		const onpin = vi.fn();
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), onpin },
		});
		const pinBtn = container.querySelector('[aria-label="Pin"]');
		await fireEvent.click(pinBtn!);
		expect(onpin).toHaveBeenCalledOnce();
	});

	it('fires onunpin when unpin button is clicked', async () => {
		const onunpin = vi.fn();
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), pinned: true, onunpin },
		});
		const unpinBtn = container.querySelector('[aria-label="Unpin"]');
		await fireEvent.click(unpinBtn!);
		expect(onunpin).toHaveBeenCalledOnce();
	});

	it('fires ondismiss when dismiss button is clicked', async () => {
		const ondismiss = vi.fn();
		const { container } = render(EvidenceCard, {
			props: { result: makeResult(), ondismiss },
		});
		const dismissBtn = container.querySelector('[aria-label="Dismiss"]');
		await fireEvent.click(dismissBtn!);
		expect(ondismiss).toHaveBeenCalledOnce();
	});

	// ── Apply button (single slot) ──────────────────────
	it('shows apply button when hasExistingContent and onapply provided', () => {
		const onapply = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapply,
				slotOptions: [{ index: 0, label: 'Tweet' }],
			},
		});
		const applyBtn = container.querySelector('[aria-label="Apply to slot"]');
		expect(applyBtn).not.toBeNull();
	});

	it('hides apply button when hasExistingContent is false', () => {
		const onapply = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: false,
				onapply,
			},
		});
		const applyBtn = container.querySelector('[aria-label="Apply to slot"]');
		expect(applyBtn).toBeNull();
	});

	it('fires onapply directly when single slot', async () => {
		const onapply = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapply,
				slotOptions: [{ index: 0, label: 'Tweet' }],
			},
		});
		const applyBtn = container.querySelector('[aria-label="Apply to slot"]');
		await fireEvent.click(applyBtn!);
		expect(onapply).toHaveBeenCalledOnce();
	});

	// ── Apply button (multi-slot / slot picker) ─────────
	it('shows chevron indicator for multiple slots', () => {
		const onapply = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapply,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		expect(applyBtn).not.toBeNull();
	});

	it('opens slot picker when clicking apply with multiple slots', async () => {
		const onapply = vi.fn();
		const onapplyToSlot = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapply,
				onapplyToSlot,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		await fireEvent.click(applyBtn!);
		const slotPicker = container.querySelector('.slot-picker');
		expect(slotPicker).not.toBeNull();
	});

	it('renders slot options in picker', async () => {
		const onapplyToSlot = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapplyToSlot,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Body' },
					{ index: 2, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		await fireEvent.click(applyBtn!);
		const options = container.querySelectorAll('.slot-option');
		expect(options.length).toBe(3);
		expect(options[0]?.textContent).toContain('Opening hook');
		expect(options[1]?.textContent).toContain('Body');
		expect(options[2]?.textContent).toContain('Closing takeaway');
	});

	it('fires onapplyToSlot with correct index and label when slot selected', async () => {
		const onapplyToSlot = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapplyToSlot,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		await fireEvent.click(applyBtn!);
		const options = container.querySelectorAll('.slot-option');
		await fireEvent.click(options[1]);
		expect(onapplyToSlot).toHaveBeenCalledWith(1, 'Closing takeaway');
	});

	it('closes slot picker after selecting a slot', async () => {
		const onapplyToSlot = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapplyToSlot,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		await fireEvent.click(applyBtn!);
		const options = container.querySelectorAll('.slot-option');
		await fireEvent.click(options[0]);
		const picker = container.querySelector('.slot-picker');
		expect(picker).toBeNull();
	});

	it('closes slot picker on Escape keydown', async () => {
		const onapplyToSlot = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapplyToSlot,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		await fireEvent.click(applyBtn!);
		const picker = container.querySelector('.slot-picker');
		expect(picker).not.toBeNull();
		await fireEvent.keyDown(picker!, { key: 'Escape' });
		expect(container.querySelector('.slot-picker')).toBeNull();
	});

	it('toggles slot picker open and closed on repeated apply clicks', async () => {
		const onapplyToSlot = vi.fn();
		const { container } = render(EvidenceCard, {
			props: {
				result: makeResult(),
				hasExistingContent: true,
				onapplyToSlot,
				slotOptions: [
					{ index: 0, label: 'Opening hook' },
					{ index: 1, label: 'Closing takeaway' },
				],
			},
		});
		const applyBtn = container.querySelector('[aria-haspopup="true"]');
		// Open
		await fireEvent.click(applyBtn!);
		expect(container.querySelector('.slot-picker')).not.toBeNull();
		// Close by clicking again
		await fireEvent.click(applyBtn!);
		expect(container.querySelector('.slot-picker')).toBeNull();
	});

	// ── Truncation ──────────────────────────────────────
	it('truncates long snippets with ellipsis', () => {
		const longSnippet = 'A'.repeat(200);
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ snippet: longSnippet }) },
		});
		const snippetEl = container.querySelector('.card-snippet');
		const text = snippetEl?.textContent ?? '';
		expect(text.length).toBeLessThan(200);
		expect(text).toContain('\u2026');
	});

	it('does not truncate short snippets', () => {
		const shortSnippet = 'Short text here.';
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ snippet: shortSnippet }) },
		});
		const snippetEl = container.querySelector('.card-snippet');
		expect(snippetEl?.textContent).toBe(shortSnippet);
	});

	// ── Edge cases ──────────────────────────────────────
	it('renders with score of 0', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ score: 0 }) },
		});
		const score = container.querySelector('.card-score');
		expect(score?.textContent).toContain('0');
	});

	it('renders with score of 1.0', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ score: 1.0 }) },
		});
		const score = container.querySelector('.card-score');
		expect(score?.textContent).toContain('100');
	});

	it('renders without any action callbacks (read-only mode)', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult() },
		});
		// Should not crash, pin button is still rendered
		expect(container.querySelector('.evidence-card')).toBeTruthy();
	});

	it('renders with empty snippet', () => {
		const { container } = render(EvidenceCard, {
			props: { result: makeResult({ snippet: '' }) },
		});
		const snippetEl = container.querySelector('.card-snippet');
		expect(snippetEl?.textContent).toBe('');
	});
});
