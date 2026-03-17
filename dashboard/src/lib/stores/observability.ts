import { writable } from 'svelte/store';
import {
	api,
	type RateLimitUsage,
	type ApprovalStats,
	type ActionLogEntry,
	type CostSummary,
	type XApiUsageSummary,
	type McpTelemetrySummary
} from '$lib/api';

export const runtimeStatus = writable<{ running: boolean; mode: string } | null>(null);
export const approvalStats = writable<ApprovalStats | null>(null);
export const rateLimits = writable<RateLimitUsage | null>(null);
export const llmCosts = writable<CostSummary | null>(null);
export const xApiCosts = writable<XApiUsageSummary | null>(null);
export const mcpSummary = writable<McpTelemetrySummary | null>(null);
export const recentErrors = writable<ActionLogEntry[]>([]);
export const errorsToday = writable(0);
export const loading = writable(true);
export const error = writable<string | null>(null);

export async function loadAll() {
	loading.set(true);
	error.set(null);

	try {
		const [runtime, stats, limits, llm, xApi, mcp, errorsRes] = await Promise.allSettled([
			api.assist.mode(),
			api.approval.stats(),
			api.activity.rateLimits(),
			api.costs.summary(),
			api.costs.xApi.summary(),
			api.mcp.telemetrySummary(24),
			api.activity.list({ limit: 10, status: 'failure' })
		]);

		if (runtime.status === 'fulfilled') {
			runtimeStatus.set({ running: true, mode: runtime.value.mode });
		}
		if (stats.status === 'fulfilled') {
			approvalStats.set(stats.value);
		}
		if (limits.status === 'fulfilled') {
			rateLimits.set(limits.value);
		}
		if (llm.status === 'fulfilled') {
			llmCosts.set(llm.value);
		}
		if (xApi.status === 'fulfilled') {
			xApiCosts.set(xApi.value);
		}
		if (mcp.status === 'fulfilled') {
			mcpSummary.set(mcp.value);
		}
		if (errorsRes.status === 'fulfilled') {
			recentErrors.set(errorsRes.value.actions);
			errorsToday.set(errorsRes.value.total);
		}
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load observability data');
	} finally {
		loading.set(false);
	}
}

// --- Account switching integration ---

// When user switches accounts, refetch all observability data for the new account.
if (typeof window !== 'undefined') {
	window.addEventListener('tuitbot:account-switched', () => {
		loadAll();
	});
}
