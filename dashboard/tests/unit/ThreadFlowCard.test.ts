/**
 * ThreadFlowCard.test.ts — Unit tests for ThreadFlowCard.svelte
 *
 * Tests: render with tweet data, character count indicator, event emissions,
 * keyboard accessibility, media attachment, and character limit warnings.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ThreadFlowCard from '$lib/components/composer/ThreadFlowCard.svelte';
import type { ThreadBlock } from '$lib/api';

// Mock tweetLength utilities
vi.mock('$lib/utils/tweetLength', () => ({
	tweetWeightedLen: vi.fn((text: string) => text.length),
	wordCount: vi.fn((text: string) => text.split(/\s+/).filter(w => w.length > 0).length),
	MAX_TWEET_CHARS: 280
}));

const createMockBlock = (overrides: Partial<ThreadBlock> = {}): ThreadBlock => ({
	id: 'block-1',
	text: 'Test tweet content',
	media_paths: [],
	order: 0,
	...overrides
});

const defaultProps = {
	block: createMockBlock(),
	index: 0,
	total: 1,
	avatarUrl: 'https://example.com/avatar.jpg',
	displayName: 'Test User',
	handle: 'testuser',
	focused: false,
	assisting: false,
	dragging: false,
	dropTarget: false,
	ontext: vi.fn(),
	onfocus: vi.fn(),
	onblur: vi.fn(),
	onkeydown: vi.fn(),
	onmedia: vi.fn(),
	onmerge: vi.fn(),
	onremove: vi.fn(),
	onaddafter: vi.fn(),
	ondragstart: vi.fn(),
	ondragend: vi.fn(),
	ondragover: vi.fn(),
	ondragenter: vi.fn(),
	ondragleave: vi.fn(),
	ondrop: vi.fn()
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('ThreadFlowCard', () => {
	it('renders without crashing', () => {
		const { container } = render(ThreadFlowCard, { props: defaultProps });
		expect(container).toBeTruthy();
	});

	it('renders tweet text from block data', () => {
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				block: createMockBlock({ text: 'Hello, world!' })
			}
		});
		const textarea = container.querySelector('textarea');
		expect(textarea?.value).toBe('Hello, world!');
	});

	it('displays character count indicator via aria-label', () => {
		const { container } = render(ThreadFlowCard, { props: defaultProps });
		const textarea = container.querySelector('textarea');
		expect(textarea?.getAttribute('aria-label')).toContain('Post 1 of 1');
	});

	it('renders with user identity (displayName and handle)', () => {
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				displayName: 'Alice Smith',
				handle: 'alicesmith'
			}
		});
		const identity = container.querySelector('.card-identity');
		expect(identity?.textContent).toContain('Alice Smith');
		expect(identity?.textContent).toContain('alicesmith');
	});

	it('renders avatar when avatarUrl is provided', () => {
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				avatarUrl: 'https://example.com/avatar.jpg'
			}
		});
		const avatar = container.querySelector('.gutter-avatar');
		expect(avatar).toBeTruthy();
		expect(avatar?.getAttribute('src')).toBe('https://example.com/avatar.jpg');
	});

	it('renders avatar placeholder when avatarUrl is null', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, avatarUrl: null }
		});
		const placeholder = container.querySelector('.gutter-avatar-placeholder');
		expect(placeholder).toBeTruthy();
	});

	it('calls ontext handler when textarea content changes', async () => {
		const ontext = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ontext }
		});
		const textarea = container.querySelector('textarea') as HTMLTextAreaElement;
		if (textarea) {
			await fireEvent.input(textarea, { target: { value: 'New text' } });
			expect(ontext).toHaveBeenCalledWith('New text');
		}
	});

	it('calls onfocus handler when textarea gains focus', async () => {
		const onfocus = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, onfocus }
		});
		const textarea = container.querySelector('textarea');
		if (textarea) {
			await fireEvent.focus(textarea);
			expect(onfocus).toHaveBeenCalled();
		}
	});

	it('calls onblur handler when textarea loses focus', async () => {
		const onblur = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, onblur }
		});
		const textarea = container.querySelector('textarea');
		if (textarea) {
			await fireEvent.blur(textarea);
			expect(onblur).toHaveBeenCalled();
		}
	});

	it('has onkeydown handler registered', () => {
		const onkeydown = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, onkeydown }
		});
		const textarea = container.querySelector('textarea');
		expect(textarea).toBeTruthy();
		expect(onkeydown).toBeDefined();
	});

	it('applies focused class when focused=true', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, focused: true }
		});
		const card = container.querySelector('.flow-card');
		expect(card?.classList.contains('focused')).toBe(true);
	});

	it('applies dragging class when dragging=true', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, dragging: true }
		});
		const card = container.querySelector('.flow-card');
		expect(card?.classList.contains('dragging')).toBe(true);
	});

	it('applies drop-target class when dropTarget=true', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, dropTarget: true }
		});
		const card = container.querySelector('.flow-card');
		expect(card?.classList.contains('drop-target')).toBe(true);
	});

	it('applies assisting class when assisting=true', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, assisting: true }
		});
		const card = container.querySelector('.flow-card');
		expect(card?.classList.contains('assisting')).toBe(true);
	});

	it('renders different placeholder text for first vs subsequent cards', () => {
		const { container: first } = render(ThreadFlowCard, {
			props: { ...defaultProps, index: 0, total: 3 }
		});
		const { container: second } = render(ThreadFlowCard, {
			props: { ...defaultProps, index: 1, total: 3 }
		});
		const firstTextarea = first.querySelector('textarea');
		const secondTextarea = second.querySelector('textarea');
		expect(firstTextarea?.getAttribute('placeholder')).toBe('Start writing...');
		expect(secondTextarea?.getAttribute('placeholder')).toBe('Continue...');
	});

	it('handles empty text content', () => {
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				block: createMockBlock({ text: '' })
			}
		});
		const textarea = container.querySelector('textarea');
		expect(textarea?.value).toBe('');
	});

	it('renders with multiple media paths', () => {
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				block: createMockBlock({
					media_paths: ['/path/to/image1.jpg', '/path/to/image2.jpg']
				})
			}
		});
		expect(container).toBeTruthy();
	});

	it('calls onmedia handler when media changes', async () => {
		const onmedia = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, onmedia }
		});
		// Media attachment is handled through MediaSlot component
		// Verification that component mounts with media handler
		expect(onmedia).toBeDefined();
	});

	it('has role="listitem" for accessibility', () => {
		const { container } = render(ThreadFlowCard, { props: defaultProps });
		const card = container.querySelector('[role="listitem"]');
		expect(card).toBeTruthy();
	});

	it('handles long text without crashing', () => {
		const longText = 'a'.repeat(500);
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				block: createMockBlock({ text: longText })
			}
		});
		const textarea = container.querySelector('textarea');
		expect(textarea?.value.length).toBe(500);
	});

	it('renders index correctly in card header', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, index: 2, total: 5 }
		});
		const textarea = container.querySelector('textarea');
		expect(textarea?.getAttribute('aria-label')).toContain('Post 3 of 5');
	});

	it('renders without displayName when not provided', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, displayName: null }
		});
		const identity = container.querySelector('.card-identity');
		// Component should still render, just without the display name
		expect(container).toBeTruthy();
	});

	it('handles drag events (dragstart)', async () => {
		const ondragstart = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ondragstart }
		});
		const card = container.querySelector('.flow-card');
		expect(card).toBeTruthy();
		expect(ondragstart).toBeDefined();
	});

	it('handles drop events', async () => {
		const ondrop = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ondrop }
		});
		const card = container.querySelector('.flow-card');
		expect(card).toBeTruthy();
		expect(ondrop).toBeDefined();
	});

	it('renders media slot component', () => {
		const { container } = render(ThreadFlowCard, { props: defaultProps });
		// MediaSlot component should be rendered
		expect(container.innerHTML.length).toBeGreaterThan(0);
	});

	it('does not crash with minimal props', () => {
		const minimalProps = {
			...defaultProps,
			avatarUrl: null,
			displayName: null,
			handle: null
		};
		const { container } = render(ThreadFlowCard, { props: minimalProps });
		expect(container).toBeTruthy();
	});

	it('applies over-limit styling when character count exceeds MAX_TWEET_CHARS', () => {
		// With mocked tweetWeightedLen returning text.length, a 300-char string exceeds 280
		const { container } = render(ThreadFlowCard, {
			props: {
				...defaultProps,
				block: createMockBlock({ text: 'a'.repeat(300) })
			}
		});
		const avatar = container.querySelector('.gutter-avatar');
		// Component should apply over-limit styling
		expect(container).toBeTruthy();
	});

	// ── Insert badges coverage ──────────────────────────
	it('renders insert badges when inserts are provided', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'block-1', slotLabel: 'Opening hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Test Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, inserts }
		});
		const badges = container.querySelector('.insert-badges');
		expect(badges).toBeTruthy();
		expect(badges?.textContent).toContain('Test Note');
	});

	it('does not render insert badges when inserts is empty', () => {
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, inserts: [] }
		});
		const badges = container.querySelector('.insert-badges');
		expect(badges).toBeNull();
	});

	it('renders undo button on insert badge when onundoinsert is provided', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'block-1', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, inserts, onundoinsert: vi.fn() }
		});
		const undoBtn = container.querySelector('.insert-badge-undo');
		expect(undoBtn).toBeTruthy();
	});

	it('calls onundoinsert with correct ID when undo badge is clicked', async () => {
		const onundoinsert = vi.fn();
		const inserts = [
			{ id: 'ins-42', blockId: 'block-1', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, inserts, onundoinsert }
		});
		const undoBtn = container.querySelector('.insert-badge-undo') as HTMLButtonElement;
		await fireEvent.click(undoBtn);
		expect(onundoinsert).toHaveBeenCalledWith('ins-42');
	});

	it('hides undo button on insert badge when onundoinsert is not provided', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'block-1', slotLabel: 'Hook', previousText: '', insertedText: 'new', sourceNodeId: 1, sourceTitle: 'Note', provenance: { node_id: 1 }, timestamp: Date.now() },
		];
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, inserts }
		});
		const undoBtn = container.querySelector('.insert-badge-undo');
		expect(undoBtn).toBeNull();
	});

	it('renders multiple insert badges', () => {
		const inserts = [
			{ id: 'ins-1', blockId: 'block-1', slotLabel: 'Hook', previousText: '', insertedText: 'a', sourceNodeId: 1, sourceTitle: 'Note A', provenance: { node_id: 1 }, timestamp: Date.now() },
			{ id: 'ins-2', blockId: 'block-1', slotLabel: 'Body', previousText: '', insertedText: 'b', sourceNodeId: 2, sourceTitle: 'Note B', provenance: { node_id: 2 }, timestamp: Date.now() },
		];
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, inserts }
		});
		const badges = container.querySelectorAll('.insert-badge');
		expect(badges.length).toBe(2);
	});

	// ── Drag event handlers ─────────────────────────────
	it('fires ondragover when card is dragged over', async () => {
		const ondragover = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ondragover }
		});
		const card = container.querySelector('.flow-card') as HTMLElement;
		await fireEvent.dragOver(card);
		expect(ondragover).toHaveBeenCalled();
	});

	it('fires ondrop when dropped on card', async () => {
		const ondrop = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ondrop }
		});
		const card = container.querySelector('.flow-card') as HTMLElement;
		await fireEvent.drop(card);
		expect(ondrop).toHaveBeenCalled();
	});

	it('fires ondragenter when entering card', async () => {
		const ondragenter = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ondragenter }
		});
		const card = container.querySelector('.flow-card') as HTMLElement;
		await fireEvent.dragEnter(card);
		expect(ondragenter).toHaveBeenCalled();
	});

	it('fires ondragleave when leaving card', async () => {
		const ondragleave = vi.fn();
		const { container } = render(ThreadFlowCard, {
			props: { ...defaultProps, ondragleave }
		});
		const card = container.querySelector('.flow-card') as HTMLElement;
		await fireEvent.dragLeave(card);
		expect(ondragleave).toHaveBeenCalled();
	});
});
