/**
 * apiClientHooks.test.ts — Unit tests for new api client methods:
 * - api.assist.hooks
 * - api.draftStudio.provenance
 * - api.vault.getSelection
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock fetch globally before importing the module under test
const mockFetch = vi.fn().mockResolvedValue({
	ok: true,
	json: () => Promise.resolve({}),
	headers: new Headers({ 'content-type': 'application/json' })
});
vi.stubGlobal('fetch', mockFetch);

// Mock the http module so request() delegates to the stubbed fetch
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

describe('api.assist.hooks', () => {
	it('is a callable function', () => {
		expect(typeof api.assist.hooks).toBe('function');
	});

	it('calls POST /api/assist/hooks with topic', async () => {
		await api.assist.hooks('growth mindset');
		expect(mockFetch).toHaveBeenCalledWith(
			'/api/assist/hooks',
			expect.objectContaining({
				method: 'POST',
				body: expect.stringContaining('growth mindset')
			})
		);
	});

	it('includes selected_node_ids when provided in opts', async () => {
		await api.assist.hooks('my topic', { selectedNodeIds: [1, 2, 3] });
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.selected_node_ids).toEqual([1, 2, 3]);
	});

	it('includes session_id when provided in opts', async () => {
		await api.assist.hooks('my topic', { sessionId: 'sess-abc' });
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.session_id).toBe('sess-abc');
	});

	it('omits selected_node_ids when not provided', async () => {
		await api.assist.hooks('my topic');
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.selected_node_ids).toBeUndefined();
	});

	it('omits session_id when not provided', async () => {
		await api.assist.hooks('my topic');
		const body = JSON.parse(mockFetch.mock.calls[0][1].body);
		expect(body.session_id).toBeUndefined();
	});
});

describe('api.draftStudio.provenance', () => {
	it('is a callable function', () => {
		expect(typeof api.draftStudio.provenance).toBe('function');
	});

	it('calls GET /api/drafts/{id}/provenance', async () => {
		await api.draftStudio.provenance(42);
		const [url] = mockFetch.mock.calls[0];
		expect(url).toBe('/api/drafts/42/provenance');
	});

	it('uses the correct draft id in the path', async () => {
		await api.draftStudio.provenance(99);
		const [url] = mockFetch.mock.calls[0];
		expect(url).toContain('/api/drafts/99/provenance');
	});
});

describe('api.vault.getSelection', () => {
	it('is a callable function', () => {
		expect(typeof api.vault.getSelection).toBe('function');
	});

	it('calls GET /api/vault/selection/{sessionId}', async () => {
		await api.vault.getSelection('my-session');
		const [url] = mockFetch.mock.calls[0];
		expect(url).toBe('/api/vault/selection/my-session');
	});

	it('encodes sessionId in the URL', async () => {
		await api.vault.getSelection('session with spaces');
		const [url] = mockFetch.mock.calls[0];
		expect(url).toContain('session%20with%20spaces');
	});
});
