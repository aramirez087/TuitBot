/**
 * apiClientVault.test.ts — Unit tests for api.vault.searchEvidence and
 * api.vault.indexStatus client methods added in the vault-indexer PR.
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

describe('api.vault.searchEvidence', () => {
	it('is a callable function', () => {
		expect(typeof api.vault.searchEvidence).toBe('function');
	});

	it('calls GET /api/vault/evidence with query param', async () => {
		await api.vault.searchEvidence({ q: 'machine learning' });
		expect(mockFetch).toHaveBeenCalledTimes(1);
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toContain('/api/vault/evidence');
		expect(url).toContain('q=machine+learning');
	});

	it('includes limit param when provided', async () => {
		await api.vault.searchEvidence({ q: 'test', limit: 5 });
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toContain('limit=5');
	});

	it('includes mode param when provided', async () => {
		await api.vault.searchEvidence({ q: 'test', mode: 'hybrid' });
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toContain('mode=hybrid');
	});

	it('includes scope param when provided', async () => {
		await api.vault.searchEvidence({ q: 'test', scope: 'selected' });
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toContain('scope=selected');
	});

	it('includes all optional params together', async () => {
		await api.vault.searchEvidence({ q: 'ai', limit: 10, mode: 'semantic', scope: 'all' });
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toContain('q=ai');
		expect(url).toContain('limit=10');
		expect(url).toContain('mode=semantic');
		expect(url).toContain('scope=all');
	});

	it('omits limit when not provided', async () => {
		await api.vault.searchEvidence({ q: 'test' });
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).not.toContain('limit=');
	});

	it('omits mode when not provided', async () => {
		await api.vault.searchEvidence({ q: 'test' });
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).not.toContain('mode=');
	});
});

describe('api.vault.indexStatus', () => {
	it('is a callable function', () => {
		expect(typeof api.vault.indexStatus).toBe('function');
	});

	it('calls GET /api/vault/index-status', async () => {
		await api.vault.indexStatus();
		expect(mockFetch).toHaveBeenCalledTimes(1);
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toBe('/api/vault/index-status');
	});

	it('does not send a request body', async () => {
		await api.vault.indexStatus();
		const init = mockFetch.mock.calls[0][1];
		// GET requests should not have body or method override
		expect(init?.body).toBeUndefined();
	});
});

describe('api.sources.reindex', () => {
	it('is a callable function', () => {
		expect(typeof api.sources.reindex).toBe('function');
	});

	it('calls POST /api/sources/:id/reindex', async () => {
		await api.sources.reindex(42);
		expect(mockFetch).toHaveBeenCalledTimes(1);
		const url = mockFetch.mock.calls[0][0] as string;
		expect(url).toBe('/api/sources/42/reindex');
		const init = mockFetch.mock.calls[0][1];
		expect(init?.method).toBe('POST');
	});
});
