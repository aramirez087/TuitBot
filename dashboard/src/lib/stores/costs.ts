import { writable } from 'svelte/store';
import {
	api,
	type CostSummary,
	type DailyCostSummary,
	type ModelCostBreakdown,
	type TypeCostBreakdown
} from '$lib/api';

// --- Writable stores ---

export const summary = writable<CostSummary | null>(null);
export const dailyCosts = writable<DailyCostSummary[]>([]);
export const modelBreakdown = writable<ModelCostBreakdown[]>([]);
export const typeBreakdown = writable<TypeCostBreakdown[]>([]);
export const loading = writable(true);
export const error = writable<string | null>(null);

// --- Data loading ---

export async function loadCosts(days: number = 30) {
	loading.set(true);
	error.set(null);

	try {
		const [s, daily, models, types] = await Promise.all([
			api.costs.summary(),
			api.costs.daily(days),
			api.costs.byModel(days),
			api.costs.byType(days)
		]);

		summary.set(s);
		dailyCosts.set(daily);
		modelBreakdown.set(models);
		typeBreakdown.set(types);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load cost data');
	} finally {
		loading.set(false);
	}
}
