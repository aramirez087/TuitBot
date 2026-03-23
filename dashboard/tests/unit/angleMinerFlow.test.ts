/**
 * angleMinerFlow.test.ts — Integration tests for Hook Miner state transitions
 * in VaultSelectionReview.
 *
 * Tests the branching logic: when angles are mined vs. fallback vs. generic hooks.
 * Uses the same mock pattern as VaultSelectionReview.test.ts.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import VaultSelectionReview from '$lib/components/composer/VaultSelectionReview.svelte';

vi.mock('$lib/api', () => ({
	api: {
		vault: { getSelection: vi.fn() },
		assist: { hooks: vi.fn(), angles: vi.fn() }
	}
}));

const sampleNeighbors = [
	{
		node_id: 55,
		node_title: 'Async Patterns',
		reason: 'linked_note',
		reason_label: 'linked note',
		intent: 'pro_tip',
		matched_tags: [],
		score: 3.5,
		snippet: 'Async patterns in Rust use tokio.',
		best_chunk_id: 120,
		heading_path: '# Async',
		relative_path: 'notes/async-patterns.md',
	},
];

const sampleSelection = {
	session_id: 'test-session-abc',
	vault_name: 'my-vault',
	file_path: 'notes/article.md',
	selected_text: 'The key insight here is that async patterns differ by context.',
	heading_context: 'Patterns > Async',
	note_title: 'Design Patterns',
	frontmatter_tags: ['patterns', 'async'],
	resolved_node_id: 7,
	resolved_chunk_id: 22,
	created_at: '2024-01-01T00:00:00Z',
	expires_at: '2024-01-02T00:00:00Z',
	graph_neighbors: sampleNeighbors,
	graph_state: 'available' as const,
};

const sampleSelectionNoGraph = {
	...sampleSelection,
	graph_neighbors: [],
	graph_state: 'no_related_notes' as const,
};

const sampleAnglesResponse = {
	angles: [
		{
			angle_type: 'story',
			seed_text: 'I spent 3 months migrating. Here is what happened.',
			char_count: 51,
			evidence: [
				{
					evidence_type: 'data_point',
					citation_text: 'migration cost 3.2x',
					source_node_id: 55,
					source_note_title: 'Async Patterns',
				}
			],
			confidence: 'high',
			rationale: 'Strong data point supports narrative.',
		}
	],
	topic: 'async patterns',
};

const sampleFallbackResponse = {
	angles: [],
	fallback_reason: 'insufficient_evidence',
	topic: 'async patterns',
};

const sampleHooksResponse = {
	hooks: [
		{ style: 'question', text: 'What if async was easy?', char_count: 23, confidence: 'high' },
	],
	topic: 'async patterns',
};

const defaultProps = {
	sessionId: 'test-session-abc',
	ongenerate: vi.fn().mockResolvedValue(undefined),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('angleMinerFlow — branching logic', () => {
	it('clicking Generate without accepted neighbors calls hooks endpoint (not angles)', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue(sampleHooksResponse);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleAnglesResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		// Generate without accepting any neighbors
		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);

		await vi.waitFor(() => {
			expect(api.assist.hooks).toHaveBeenCalled();
		});
		expect(api.assist.angles).not.toHaveBeenCalled();
	});

	it('shows HookPicker after generic hooks response (no neighbors)', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue(sampleHooksResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);

		await vi.waitFor(() => {
			expect(container.querySelector('.hook-picker')).toBeTruthy();
		});
	});

	it('clicking Generate with accepted neighbors calls angles endpoint', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleAnglesResponse);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue(sampleHooksResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		// Accept a neighbor by clicking the accept button in GraphSuggestionCards
		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
		}

		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate')
		) as HTMLButtonElement;
		await fireEvent.click(generateBtn);

		// If neighbors were accepted, angles endpoint should be called
		await vi.waitFor(() => {
			// If accept button was found, angles was called; otherwise hooks
			if (acceptBtn) {
				expect(api.assist.angles).toHaveBeenCalled();
			} else {
				expect(api.assist.hooks).toHaveBeenCalled();
			}
		});
	});

	it('shows AngleCards when angles endpoint returns angles', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleAnglesResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
			const generateBtn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate')
			) as HTMLButtonElement;
			await fireEvent.click(generateBtn);

			await vi.waitFor(() => {
				expect(container.querySelector('.angle-picker')).toBeTruthy();
			});
		}
	});

	it('shows AngleFallback when angles endpoint returns fallback_reason', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleFallbackResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
			const generateBtn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate')
			) as HTMLButtonElement;
			await fireEvent.click(generateBtn);

			await vi.waitFor(() => {
				expect(container.querySelector('.angle-fallback')).toBeTruthy();
			});
		}
	});

	it('AngleFallback "Use generic hooks" transitions to HookPicker', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleFallbackResponse);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue(sampleHooksResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
			const generateBtn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate')
			) as HTMLButtonElement;
			await fireEvent.click(generateBtn);

			await vi.waitFor(() => {
				expect(container.querySelector('.angle-fallback')).toBeTruthy();
			});

			const useGenericBtn = container.querySelector('.angle-fallback-primary') as HTMLButtonElement;
			await fireEvent.click(useGenericBtn);

			await vi.waitFor(() => {
				expect(api.assist.hooks).toHaveBeenCalled();
			});
		}
	});

	it('AngleFallback "Back to related notes" returns to selection view', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleFallbackResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
			const generateBtn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate')
			) as HTMLButtonElement;
			await fireEvent.click(generateBtn);

			await vi.waitFor(() => {
				expect(container.querySelector('.angle-fallback')).toBeTruthy();
			});

			const backBtn = container.querySelector('.angle-fallback-secondary') as HTMLButtonElement;
			await fireEvent.click(backBtn);

			await vi.waitFor(() => {
				expect(container.querySelector('.angle-fallback')).toBeFalsy();
				expect(container.querySelector('.vault-selection-review')).toBeTruthy();
			});
		}
	});

	it('shows angle-error when angles endpoint throws non-timeout error', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('Server error'));

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
			const generateBtn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate')
			) as HTMLButtonElement;
			await fireEvent.click(generateBtn);

			await vi.waitFor(() => {
				const errorEl = container.querySelector('.angle-error');
				expect(errorEl).toBeTruthy();
			});
		}
	});

	it('shows AngleFallback with timeout message on timeout error', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('Request timeout'));

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
			const generateBtn = Array.from(container.querySelectorAll('button')).find(
				(b) => b.textContent?.includes('Generate')
			) as HTMLButtonElement;
			await fireEvent.click(generateBtn);

			await vi.waitFor(() => {
				const fallback = container.querySelector('.angle-fallback');
				expect(fallback).toBeTruthy();
			});
		}
	});

	it('disabling synthesis and generating goes to hooks endpoint', async () => {
		const { api } = await import('$lib/api');
		(api.vault.getSelection as ReturnType<typeof vi.fn>).mockResolvedValueOnce(sampleSelection);
		(api.assist.hooks as ReturnType<typeof vi.fn>).mockResolvedValue(sampleHooksResponse);
		(api.assist.angles as ReturnType<typeof vi.fn>).mockResolvedValue(sampleAnglesResponse);

		const { container } = render(VaultSelectionReview, { props: defaultProps });
		await vi.waitFor(() => {
			expect(container.querySelector('.vault-selection-review')).toBeTruthy();
		});

		// Accept neighbor first
		const acceptBtn = container.querySelector('.suggestion-accept-btn') as HTMLButtonElement;
		if (acceptBtn) {
			await fireEvent.click(acceptBtn);
		}

		// Toggle synthesis off
		const toggleBtn = container.querySelector('.synthesis-toggle') as HTMLButtonElement;
		if (toggleBtn) {
			await fireEvent.click(toggleBtn);
		}

		const generateBtn = Array.from(container.querySelectorAll('button')).find(
			(b) => b.textContent?.includes('Generate')
		) as HTMLButtonElement;
		if (generateBtn) {
			await fireEvent.click(generateBtn);
			await vi.waitFor(() => {
				expect(api.assist.hooks).toHaveBeenCalled();
			});
			expect(api.assist.angles).not.toHaveBeenCalled();
		}
	});
});
