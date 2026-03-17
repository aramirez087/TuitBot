import { writable } from 'svelte/store';
import {
	api,
	type CostSummary,
	type DailyCostSummary,
	type ModelCostBreakdown,
	type TypeCostBreakdown,
	type XApiUsageSummary,
	type DailyXApiUsage,
	type EndpointBreakdown
} from '$lib/api';

// --- LLM stores ---

export const summary = writable<CostSummary | null>(null);
export const dailyCosts = writable<DailyCostSummary[]>([]);
export const modelBreakdown = writable<ModelCostBreakdown[]>([]);
export const typeBreakdown = writable<TypeCostBreakdown[]>([]);

// --- X API stores ---

export const xApiSummary = writable<XApiUsageSummary | null>(null);
export const xApiDailyCalls = writable<DailyXApiUsage[]>([]);
export const xApiEndpoints = writable<EndpointBreakdown[]>([]);

// --- Shared state ---

export const loading = writable(true);
export const error = writable<string | null>(null);

// --- Data loading ---

export async function loadCosts(days: number = 30) {
	loading.set(true);
	error.set(null);

	try {
		const [s, daily, models, types, xSummary, xDaily, xEndpoints] = await Promise.all([
			api.costs.summary(),
			api.costs.daily(days),
			api.costs.byModel(days),
			api.costs.byType(days),
			api.costs.xApi.summary(),
			api.costs.xApi.daily(days),
			api.costs.xApi.byEndpoint(days)
		]);

		summary.set(s);
		dailyCosts.set(daily);
		modelBreakdown.set(models);
		typeBreakdown.set(types);
		xApiSummary.set(xSummary);
		xApiDailyCalls.set(xDaily);
		xApiEndpoints.set(xEndpoints);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load cost data');
	} finally {
		loading.set(false);
	}
}

// --- Account switching integration ---

// When user switches accounts, refetch all cost data for the new account.
if (typeof window !== 'undefined') {
	window.addEventListener('tuitbot:account-switched', () => {
		loadCosts();
	});
}
