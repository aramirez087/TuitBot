/**
 * mockApi.ts — Mock API client returning predictable responses for tests.
 *
 * Mirrors the shape of `src/lib/api/client.ts` but replaces all network
 * calls with vi.fn() stubs pre-loaded with fixture data.
 *
 * Usage:
 *   import { mockApi, resetMockApi } from '../helpers/mockApi';
 *   vi.mock('$lib/api', () => ({ api: mockApi, ...mockApi }));
 */

import { vi } from 'vitest';
import { fixtures } from './fixtures';
import type {
	HealthResponse,
	RuntimeStatus,
	ApprovalItem,
	ApprovalStats,
	AnalyticsSummary
} from '../../src/lib/api/types';

// ---------------------------------------------------------------------------
// Health & runtime
// ---------------------------------------------------------------------------

const health = vi.fn<() => Promise<HealthResponse>>().mockResolvedValue({
	status: 'ok',
	version: '0.1.0-test'
});

const runtime = {
	status: vi.fn<() => Promise<RuntimeStatus>>().mockResolvedValue({
		running: true,
		task_count: 3,
		deployment_mode: 'desktop',
		capabilities: {
			local_folder: true,
			manual_local_path: true,
			google_drive: false,
			inline_ingest: true,
			file_picker_native: true,
			preferred_source_default: 'local',
			privacy_envelope: 'local_first',
			ghostwriter_local_only: true
		},
		provider_backend: 'local',
		can_post: false,
		capability_tier: 'generation_ready'
	}),
	start: vi.fn().mockResolvedValue({ status: 'started' }),
	stop: vi.fn().mockResolvedValue({ status: 'stopped' })
};

// ---------------------------------------------------------------------------
// Approval
// ---------------------------------------------------------------------------

const approval = {
	list: vi.fn<() => Promise<ApprovalItem[]>>()
		.mockResolvedValue(fixtures.approvalItems),
	stats: vi.fn<() => Promise<ApprovalStats>>().mockResolvedValue(fixtures.approvalStats),
	approve: vi.fn().mockResolvedValue({ success: true }),
	reject: vi.fn().mockResolvedValue({ success: true }),
	edit: vi.fn().mockResolvedValue(fixtures.approvalItem()),
	history: vi.fn().mockResolvedValue([])
};

// ---------------------------------------------------------------------------
// Analytics
// ---------------------------------------------------------------------------

const analytics = {
	summary: vi.fn<() => Promise<AnalyticsSummary>>().mockResolvedValue(fixtures.analyticsSummary),
	followers: vi.fn().mockResolvedValue([]),
	performance: vi.fn().mockResolvedValue([])
};

// ---------------------------------------------------------------------------
// Settings / config
// ---------------------------------------------------------------------------

const settings = {
	get: vi.fn().mockResolvedValue({ config: fixtures.config, status: { connected: true } }),
	update: vi.fn().mockResolvedValue({ success: true }),
	validate: vi.fn().mockResolvedValue({ valid: true, errors: [] }),
	test: vi.fn().mockResolvedValue({ success: true, message: 'Connection OK' })
};

// ---------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------

const targets = {
	list: vi.fn().mockResolvedValue(fixtures.targets),
	add: vi.fn().mockResolvedValue(fixtures.targetAccount()),
	remove: vi.fn().mockResolvedValue({ success: true }),
	stats: vi.fn().mockResolvedValue({ followers: [], interactions: [] })
};

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

const auth = {
	status: vi.fn().mockResolvedValue({ authenticated: false, account_id: null }),
	link: vi.fn().mockResolvedValue({ url: 'https://x.com/oauth/authorize?...' }),
	callback: vi.fn().mockResolvedValue({ success: true }),
	logout: vi.fn().mockResolvedValue({ success: true })
};

// ---------------------------------------------------------------------------
// Assembled mock API object
// ---------------------------------------------------------------------------

export const mockApi = {
	health,
	runtime,
	approval,
	analytics,
	settings,
	targets,
	auth
};

// ---------------------------------------------------------------------------
// Reset all mocks to default resolved values (call in beforeEach)
// ---------------------------------------------------------------------------

export function resetMockApi(): void {
	health.mockResolvedValue({ status: 'ok', version: '0.1.0-test' });
	runtime.status.mockResolvedValue({
		running: true,
		task_count: 3,
		deployment_mode: 'desktop',
		capabilities: {
			local_folder: true,
			manual_local_path: true,
			google_drive: false,
			inline_ingest: true,
			file_picker_native: true,
			preferred_source_default: 'local',
			privacy_envelope: 'local_first',
			ghostwriter_local_only: true
		},
		provider_backend: 'local',
		can_post: false,
		capability_tier: 'generation_ready'
	});
	approval.list.mockResolvedValue(fixtures.approvalItems);
	approval.stats.mockResolvedValue(fixtures.approvalStats);
	analytics.summary.mockResolvedValue(fixtures.analyticsSummary);
	settings.get.mockResolvedValue({ config: fixtures.config, status: { connected: true } });
	targets.list.mockResolvedValue(fixtures.targets);

	vi.clearAllMocks();
}
