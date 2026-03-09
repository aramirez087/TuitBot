import { derived } from 'svelte/store';
import type { CapabilityTier } from '$lib/api/types';
import { capabilityTier as rawTier } from './runtime';

const TIER_RANK: Record<CapabilityTier, number> = {
	unconfigured: 0,
	profile_ready: 1,
	exploration_ready: 2,
	generation_ready: 3,
	posting_ready: 4
};

const TIER_LABELS: Record<CapabilityTier, string> = {
	unconfigured: 'Unconfigured',
	profile_ready: 'Profile Ready',
	exploration_ready: 'Exploration Ready',
	generation_ready: 'Generation Ready',
	posting_ready: 'Posting Ready'
};

export function tierRank(tier: CapabilityTier): number {
	return TIER_RANK[tier] ?? 0;
}

export function tierLabel(tier: CapabilityTier): string {
	return TIER_LABELS[tier] ?? 'Unknown';
}

export const capabilityTier = rawTier;
export const canExplore = derived(rawTier, (t) => tierRank(t) >= 2);
export const canGenerate = derived(rawTier, (t) => tierRank(t) >= 3);
export const canPublish = derived(rawTier, (t) => tierRank(t) >= 4);
