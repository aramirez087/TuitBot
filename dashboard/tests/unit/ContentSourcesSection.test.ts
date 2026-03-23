/**
 * ContentSourcesSection.test.ts — Unit tests for the settings
 * ContentSourcesSection.svelte component, focusing on semantic index
 * status display and vault health summary rendering.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import { writable } from 'svelte/store';

// vi.hoisted ensures these are available when vi.mock factories run (hoisted above imports)
const { mockDraft, mockCapabilities, mockDeploymentMode, mockExpiredGoogleDrive } = vi.hoisted(() => {
	// Inline writable implementation to avoid import ordering issues
	const { writable } = require('svelte/store');
	return {
		mockDraft: writable(null as Record<string, unknown> | null),
		mockCapabilities: writable({ local_folder: true, manual_local_path: true, file_picker_native: false, google_drive: true }),
		mockDeploymentMode: writable('desktop'),
		mockExpiredGoogleDrive: writable(false),
	};
});

vi.mock('$lib/stores/settings', () => ({
	draft: mockDraft,
	updateDraft: vi.fn(),
	resetAnalyticsSyncPrompt: vi.fn(),
}));

vi.mock('$lib/stores/runtime', () => ({
	capabilities: mockCapabilities,
	deploymentMode: mockDeploymentMode,
	loadCapabilities: vi.fn(),
}));

vi.mock('$lib/stores/connectors', () => ({
	loadConnections: vi.fn(),
	expiredGoogleDrive: mockExpiredGoogleDrive,
}));

vi.mock('$lib/api', () => ({
	api: {
		vault: {
			sources: vi.fn().mockResolvedValue({
				sources: [{
					id: 1,
					source_type: 'local_fs',
					status: 'active',
					node_count: 42,
					updated_at: new Date().toISOString(),
					error_message: null,
				}]
			}),
			indexStatus: vi.fn().mockResolvedValue({
				total_chunks: 100,
				embedded_chunks: 95,
				dirty_chunks: 5,
				freshness_pct: 95,
				last_indexed_at: '2026-03-23T10:00:00Z',
				model_id: 'text-embedding-3-small',
				provider_configured: true,
				index_loaded: true,
				index_size: 1024,
				deployment_mode: 'desktop',
				search_available: true,
				provider_name: 'openai',
			}),
		},
		sources: {
			reindex: vi.fn().mockResolvedValue({ status: 'ok', source_id: 1 }),
		},
	}
}));

vi.mock('$lib/utils/obsidianUri', () => ({
	buildObsidianVaultUri: vi.fn().mockReturnValue('obsidian://vault/test'),
	openExternalUrl: vi.fn().mockResolvedValue(true),
}));

import ContentSourcesSection from '../../src/routes/(app)/settings/ContentSourcesSection.svelte';
import { api } from '$lib/api';

const defaultDraft = {
	content_sources: {
		sources: [{
			source_type: 'local_fs',
			path: '/Users/alice/vault',
			folder_id: null,
			service_account_key: null,
			connection_id: null,
			watch: true,
			file_patterns: ['*.md', '*.txt'],
			loop_back_enabled: true,
			analytics_sync_enabled: false,
			poll_interval_seconds: null,
		}]
	}
};

beforeEach(() => {
	vi.clearAllMocks();
	mockDraft.set({ ...defaultDraft });
	// Re-set default mock implementations after clearAllMocks
	vi.mocked(api.vault.sources).mockResolvedValue({
		sources: [{
			id: 1,
			source_type: 'local_fs',
			status: 'active',
			node_count: 42,
			updated_at: new Date().toISOString(),
			error_message: null,
		}]
	} as never);
	vi.mocked(api.vault.indexStatus).mockResolvedValue({
		total_chunks: 100,
		embedded_chunks: 95,
		dirty_chunks: 5,
		freshness_pct: 95,
		last_indexed_at: '2026-03-23T10:00:00Z',
		model_id: 'text-embedding-3-small',
		provider_configured: true,
		index_loaded: true,
		index_size: 1024,
		deployment_mode: 'desktop',
		search_available: true,
		provider_name: 'openai',
	} as never);
});

describe('ContentSourcesSection', () => {
	it('renders without crashing', () => {
		const { container } = render(ContentSourcesSection);
		expect(container).toBeTruthy();
	});

	it('renders nothing when draft is null', () => {
		mockDraft.set(null);
		const { container } = render(ContentSourcesSection);
		expect(container.innerHTML.length).toBeLessThan(50);
	});

	it('calls vault.sources on mount for health data', async () => {
		render(ContentSourcesSection);
		await vi.waitFor(() => {
			expect(api.vault.sources).toHaveBeenCalled();
		});
	});

	it('calls vault.indexStatus on mount', async () => {
		render(ContentSourcesSection);
		await vi.waitFor(() => {
			expect(api.vault.indexStatus).toHaveBeenCalled();
		});
	});

	it('shows local privacy notice for desktop + local_fs', () => {
		const { container } = render(ContentSourcesSection);
		expect(container.textContent).toContain('processed locally');
	});

	it('shows source type selector', () => {
		const { container } = render(ContentSourcesSection);
		const select = container.querySelector('#source_type');
		expect(select).toBeTruthy();
	});

	it('shows file patterns section', () => {
		const { container } = render(ContentSourcesSection);
		expect(container.textContent).toContain('File Patterns');
	});

	it('displays semantic index section after indexStatus resolves', async () => {
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			const semanticTitle = container.querySelector('.semantic-title');
			expect(semanticTitle?.textContent).toContain('Semantic Index');
		});
	});

	it('shows embedded chunk counts in semantic index', async () => {
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			expect(container.textContent).toContain('95');
			expect(container.textContent).toContain('100');
		});
	});

	it('shows model_id in semantic index details', async () => {
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			expect(container.textContent).toContain('text-embedding-3-small');
		});
	});

	it('shows provider name in semantic index details', async () => {
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			expect(container.textContent).toContain('openai');
		});
	});

	it('shows search available status', async () => {
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			const searchOk = container.querySelector('.search-ok');
			expect(searchOk?.textContent).toContain('Available');
		});
	});

	it('shows privacy label for desktop mode', async () => {
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			const privacy = container.querySelector('.semantic-privacy');
			expect(privacy?.textContent).toContain('Vectors never leave this machine');
		});
	});

	it('hides semantic index section when provider not configured', async () => {
		vi.mocked(api.vault.indexStatus).mockResolvedValueOnce({
			total_chunks: 0,
			embedded_chunks: 0,
			dirty_chunks: 0,
			freshness_pct: 0,
			last_indexed_at: null,
			model_id: null,
			provider_configured: false,
			index_loaded: false,
			index_size: 0,
			search_available: false,
			provider_name: null,
		} as never);
		const { container } = render(ContentSourcesSection);
		await new Promise((r) => setTimeout(r, 50));
		const semanticTitle = container.querySelector('.semantic-title');
		expect(semanticTitle).toBeNull();
	});

	it('shows keyword fallback when search not available', async () => {
		vi.mocked(api.vault.indexStatus).mockResolvedValueOnce({
			total_chunks: 100,
			embedded_chunks: 50,
			dirty_chunks: 50,
			freshness_pct: 50,
			last_indexed_at: null,
			model_id: null,
			provider_configured: true,
			index_loaded: true,
			index_size: 512,
			search_available: false,
			provider_name: 'ollama',
		} as never);
		const { container } = render(ContentSourcesSection);
		await vi.waitFor(() => {
			const searchWarn = container.querySelector('.search-warn');
			expect(searchWarn?.textContent).toContain('Keyword fallback');
		});
	});
});
