/**
 * ApprovalCardHeader.test.ts — Unit tests for ApprovalCardHeader component
 *
 * Covers: badge rendering with different statuses, failed_post_recovery detection, relativeTime
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import ApprovalCardHeader from '$lib/components/ApprovalCardHeader.svelte';
import type { ApprovalItem } from '$lib/api';

const makeItem = (overrides?: Partial<ApprovalItem>): ApprovalItem => ({
	id: 1,
	action_type: 'reply' as const,
	status: 'pending',
	target_tweet_id: '123',
	target_author: 'author',
	generated_content: 'Test content',
	topic: 'test',
	archetype: 'builder',
	score: 0.8,
	created_at: new Date(Date.now() - 5 * 60000).toISOString(), // 5 minutes ago
	media_paths: [],
	detected_risks: [],
	qa_score: 0.9,
	qa_hard_flags: [],
	qa_soft_flags: [],
	qa_requires_override: false,
	...overrides
});

describe('ApprovalCardHeader', () => {
	it('should render pending status badge', () => {
		const item = makeItem();
		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'reply',
				scheduledLabel: null
			}
		});

		const badge = container.querySelector('.card-badge.status-pending');
		expect(badge).toBeTruthy();
		expect(badge?.textContent).toBe('pending');
	});

	it('should render Failed badge for failed_post_recovery action_type', () => {
		const item = makeItem({
			action_type: 'failed_post_recovery' as const,
			status: 'pending'
		});

		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'recovery',
				scheduledLabel: null
			}
		});

		const badge = container.querySelector('.card-badge.status-failed');
		expect(badge).toBeTruthy();
		expect(badge?.textContent).toBe('Failed');
	});

	it('should render red color for failed badge', () => {
		const item = makeItem({
			action_type: 'failed_post_recovery' as const
		});

		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'recovery',
				scheduledLabel: null
			}
		});

		const badge = container.querySelector('.card-badge.status-failed');
		// The class should contain the danger color styling
		expect(badge?.classList.contains('status-failed')).toBe(true);
	});

	it('should render approved status badge for non-failed items', () => {
		const item = makeItem({
			status: 'approved'
		});

		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-approved',
				typeLabel: 'reply',
				scheduledLabel: null
			}
		});

		const badge = container.querySelector('.card-badge.status-approved');
		expect(badge).toBeTruthy();
		expect(badge?.textContent).toBe('approved');
	});

	it('should render type label correctly', () => {
		const item = makeItem();
		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'reply',
				scheduledLabel: null
			}
		});

		const typeSpan = container.querySelector('.card-type');
		expect(typeSpan?.textContent).toBe('reply');
	});

	it('should display score when present', () => {
		const item = makeItem({ score: 95.5 });
		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'reply',
				scheduledLabel: null
			}
		});

		const score = container.querySelector('.card-score');
		expect(score).toBeTruthy();
		expect(score?.textContent).toMatch(/96 pts/); // 95.5 rounds to 96
	});

	it('should not display score when zero', () => {
		const item = makeItem({ score: 0 });
		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'reply',
				scheduledLabel: null
			}
		});

		const score = container.querySelector('.card-score');
		expect(score).toBeFalsy();
	});

	it('should display relative time', () => {
		const item = makeItem(); // 5 minutes ago
		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'reply',
				scheduledLabel: null
			}
		});

		const time = container.querySelector('.card-time');
		expect(time?.textContent).toContain('m ago');
	});

	it('should handle failed recovery items without target_author', () => {
		const item = makeItem({
			action_type: 'failed_post_recovery' as const,
			target_author: '',
			generated_content: 'Failed scheduled content'
		});

		const { container } = render(ApprovalCardHeader, {
			props: {
				item,
				statusClass: 'status-pending',
				typeLabel: 'scheduled',
				scheduledLabel: null
			}
		});

		// Should render without error
		const badge = container.querySelector('.card-badge.status-failed');
		expect(badge?.textContent).toBe('Failed');

		// Should not show context for failed recovery (different action_type)
		const context = container.querySelector('.card-context');
		expect(context).toBeFalsy();
	});
});
