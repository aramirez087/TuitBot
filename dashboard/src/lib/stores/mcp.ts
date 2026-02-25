import { writable } from 'svelte/store';
import {
	api,
	type McpPolicyStatus,
	type McpTelemetrySummary,
	type McpToolMetrics,
	type McpErrorBreakdown,
	type McpTelemetryEntry,
	type McpPolicyPatch
} from '$lib/api';

// --- Writable stores ---

export const policy = writable<McpPolicyStatus | null>(null);
export const summary = writable<McpTelemetrySummary | null>(null);
export const metrics = writable<McpToolMetrics[]>([]);
export const errors = writable<McpErrorBreakdown[]>([]);
export const recentExecutions = writable<McpTelemetryEntry[]>([]);
export const loading = writable(true);
export const error = writable<string | null>(null);

// --- Data loading ---

export async function loadMcpData(hours: number = 24) {
	loading.set(true);
	error.set(null);

	try {
		const [policyData, summaryData, metricsData, errorsData, recentData] = await Promise.all([
			api.mcp.policy(),
			api.mcp.telemetrySummary(hours),
			api.mcp.telemetryMetrics(hours),
			api.mcp.telemetryErrors(hours),
			api.mcp.telemetryRecent(50)
		]);

		policy.set(policyData);
		summary.set(summaryData);
		metrics.set(metricsData);
		errors.set(errorsData);
		recentExecutions.set(recentData);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load MCP data');
	} finally {
		loading.set(false);
	}
}

export async function updatePolicy(patch: McpPolicyPatch) {
	error.set(null);
	try {
		await api.mcp.patchPolicy(patch);
		// Reload policy to get the full state including rate limit info
		const policyData = await api.mcp.policy();
		policy.set(policyData);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to update policy');
		throw e;
	}
}

// --- Auto-refresh ---

let refreshInterval: ReturnType<typeof setInterval> | null = null;

export function startAutoRefresh(intervalMs: number = 30_000, hours: number = 24) {
	stopAutoRefresh();
	refreshInterval = setInterval(() => {
		loadMcpData(hours);
	}, intervalMs);
}

export function stopAutoRefresh() {
	if (refreshInterval) {
		clearInterval(refreshInterval);
		refreshInterval = null;
	}
}
