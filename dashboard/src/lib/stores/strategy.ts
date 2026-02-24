import { writable, derived } from 'svelte/store';
import {
	api,
	type StrategyReport,
	type StrategyInputs,
	type Recommendation
} from '$lib/api';

// --- Writable stores ---

export const currentReport = writable<StrategyReport | null>(null);
export const reportHistory = writable<StrategyReport[]>([]);
export const inputs = writable<StrategyInputs | null>(null);
export const loading = writable(true);
export const error = writable<string | null>(null);

// --- Derived stores ---

export const followerDelta = derived(currentReport, ($r) => $r?.follower_delta ?? 0);
export const recommendations = derived(
	currentReport,
	($r) => $r?.recommendations ?? []
);
export const highPriorityCount = derived(recommendations, ($recs) =>
	$recs.filter((r: Recommendation) => r.priority === 'high').length
);

// --- Data loading ---

export async function loadStrategy() {
	loading.set(true);
	error.set(null);

	try {
		const [report, history, strategyInputs] = await Promise.all([
			api.strategy.current(),
			api.strategy.history(12),
			api.strategy.inputs()
		]);

		currentReport.set(report);
		reportHistory.set(history);
		inputs.set(strategyInputs);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load strategy data');
	} finally {
		loading.set(false);
	}
}

export async function refreshReport() {
	try {
		const report = await api.strategy.refresh();
		currentReport.set(report);
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to refresh report');
	}
}
