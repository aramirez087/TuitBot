/**
 * apiClientAngles.test.ts — Unit tests for api.assist.angles client method.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockFetch = vi.fn().mockResolvedValue({
	ok: true,
	json: () => Promise.resolve({}),
	headers: new Headers({ 'content-type': 'application/json' })
});
vi.stubGlobal('fetch', mockFetch);

vi.mock('$lib/api/http', () => ({
	BASE_URL: '',
	getAuthMode: vi.fn(() => 'cookie'),
	getCsrfToken: vi.fn(() => null),
	request: vi.fn(async (path: string, init?: RequestInit) => {
		const res = await fetch(path, init);
		return res.json();
	}),
	uploadFile: vi.fn()
}));

import { api } from '$lib/api/client';

beforeEach(() => {
	mockFetch.mockClear();
	mockFetch.mockResolvedValue({
		ok: true,
		json: () => Promise.resolve({}),
		headers: new Headers({ 'content-type': 'application/json' })
	});
});

describe('api.assist.angles', () => {
	it('is a callable function', () => {
		expect(typeof api.assist.angles).toBe('function');
	});

	it('calls POST /api/assist/angles with topic and neighbor IDs', async () => {
		await api.assist.angles('growth metrics', [42, 57]);
		expect(mockFetch).toHaveBeenCalledWith(
			'/api/assist/angles',
			expect.objectContaining({
				method: 'POST',
			})
		);
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.topic).toBe('growth metrics');
		expect(body.accepted_neighbor_ids).toEqual([42, 57]);
	});

	it('includes session_id when provided in opts', async () => {
		await api.assist.angles('topic', [1], { sessionId: 'sess-xyz' });
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.session_id).toBe('sess-xyz');
	});

	it('includes selected_node_ids when provided in opts', async () => {
		await api.assist.angles('topic', [1], { selectedNodeIds: [10, 20] });
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.selected_node_ids).toEqual([10, 20]);
	});

	it('omits session_id when not provided', async () => {
		await api.assist.angles('topic', [1]);
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.session_id).toBeUndefined();
	});

	it('omits selected_node_ids when not provided', async () => {
		await api.assist.angles('topic', [1]);
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.selected_node_ids).toBeUndefined();
	});

	it('sends both optional params when both provided', async () => {
		await api.assist.angles('topic', [1, 2], { sessionId: 's1', selectedNodeIds: [10] });
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.session_id).toBe('s1');
		expect(body.selected_node_ids).toEqual([10]);
		expect(body.accepted_neighbor_ids).toEqual([1, 2]);
	});
});
