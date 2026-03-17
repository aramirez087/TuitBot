/**
 * SchedulePickerSlots.test.ts — Unit tests for SchedulePickerSlots.svelte
 *
 * Covers: preferred time rendering, quick-slot button, loading state,
 * accessibility attributes, and clear button visibility.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

// Mock lucide-svelte icons
vi.mock('lucide-svelte', () => ({
	Zap: vi.fn().mockReturnValue(null),
	X: vi.fn().mockReturnValue(null),
	Loader2: vi.fn().mockReturnValue(null)
}));

import SchedulePickerSlots from '$lib/components/SchedulePickerSlots.svelte';

const defaultProps = {
	preferredTimes: ['09:00', '12:00', '18:00'],
	selectedTime: null as string | null,
	hasSelection: false,
	status: 'draft' as const,
	onselecttime: vi.fn(),
	onquickslot: vi.fn()
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('SchedulePickerSlots', () => {
	it('renders preferred time pills', () => {
		const { container } = render(SchedulePickerSlots, { props: defaultProps });
		const pills = container.querySelectorAll('.slot-pill');
		expect(pills.length).toBe(3);
		expect(pills[0].textContent?.trim()).toBe('09:00');
	});

	it('marks selected time as active', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, selectedTime: '12:00' }
		});
		const pills = container.querySelectorAll('.slot-pill');
		const activePill = Array.from(pills).find((p) => p.classList.contains('active'));
		expect(activePill?.textContent?.trim()).toBe('12:00');
	});

	it('calls onselecttime when time pill clicked', async () => {
		const onselecttime = vi.fn();
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, onselecttime }
		});
		const pills = container.querySelectorAll('.slot-pill');
		await fireEvent.click(pills[1]);
		expect(onselecttime).toHaveBeenCalledWith('12:00');
	});

	it('renders next free slot button', () => {
		const { container } = render(SchedulePickerSlots, { props: defaultProps });
		const btn = container.querySelector('.quick-btn') as HTMLButtonElement;
		expect(btn).toBeTruthy();
		expect(btn.textContent).toContain('Next free slot');
	});

	it('calls onquickslot when next free slot clicked', async () => {
		const onquickslot = vi.fn();
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, onquickslot }
		});
		const btn = container.querySelector('.quick-btn') as HTMLButtonElement;
		await fireEvent.click(btn);
		expect(onquickslot).toHaveBeenCalled();
	});

	it('disables next free slot button when loadingNextSlot is true', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, loadingNextSlot: true }
		});
		const btn = container.querySelector('.quick-btn') as HTMLButtonElement;
		expect(btn.disabled).toBe(true);
	});

	it('sets aria-busy on next free slot button when loading', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, loadingNextSlot: true }
		});
		const btn = container.querySelector('.quick-btn') as HTMLButtonElement;
		expect(btn.getAttribute('aria-busy')).toBe('true');
	});

	it('does not disable next free slot button when not loading', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, loadingNextSlot: false }
		});
		const btn = container.querySelector('.quick-btn') as HTMLButtonElement;
		expect(btn.disabled).toBe(false);
		expect(btn.getAttribute('aria-busy')).toBe('false');
	});

	it('shows clear button when hasSelection is true and status is draft', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, hasSelection: true, status: 'draft' }
		});
		const clearBtn = container.querySelector('.quick-btn.clear');
		expect(clearBtn).toBeTruthy();
	});

	it('hides clear button when hasSelection is false', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, hasSelection: false }
		});
		const clearBtn = container.querySelector('.quick-btn.clear');
		expect(clearBtn).toBeFalsy();
	});

	it('hides clear button when status is scheduled', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, hasSelection: true, status: 'scheduled' }
		});
		const clearBtn = container.querySelector('.quick-btn.clear');
		expect(clearBtn).toBeFalsy();
	});

	it('renders no pills when preferredTimes is empty', () => {
		const { container } = render(SchedulePickerSlots, {
			props: { ...defaultProps, preferredTimes: [] }
		});
		const pills = container.querySelectorAll('.slot-pill');
		expect(pills.length).toBe(0);
	});

	it('has proper aria-label on next free slot button', () => {
		const { container } = render(SchedulePickerSlots, { props: defaultProps });
		const btn = container.querySelector('.quick-btn') as HTMLButtonElement;
		expect(btn.getAttribute('aria-label')).toBe('Schedule for next free slot');
	});
});
