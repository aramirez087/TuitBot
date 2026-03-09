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

const TIER_COLORS: Record<CapabilityTier, string> = {
	unconfigured: 'var(--color-text-subtle)',
	profile_ready: 'var(--color-warning)',
	exploration_ready: 'var(--color-warning)',
	generation_ready: 'var(--color-accent)',
	posting_ready: 'var(--color-success)'
};

export function tierRank(tier: CapabilityTier): number {
	return TIER_RANK[tier] ?? 0;
}

export function tierLabel(tier: CapabilityTier): string {
	return TIER_LABELS[tier] ?? 'Unknown';
}

export function tierColor(tier: CapabilityTier): string {
	return TIER_COLORS[tier] ?? 'var(--color-text-subtle)';
}

export const capabilityTier = rawTier;
export const canExplore = derived(rawTier, (t) => tierRank(t) >= 2);
export const canGenerate = derived(rawTier, (t) => tierRank(t) >= 3);
export const canPublish = derived(rawTier, (t) => tierRank(t) >= 4);

// --- Activation Checklist ---

export interface ChecklistItem {
	id: string;
	label: string;
	description: string;
	completed: boolean;
	href: string;
	tier: number;
	optional: boolean;
}

export interface TierAction {
	label: string;
	description: string;
}

export function computeChecklistItems(tier: CapabilityTier): ChecklistItem[] {
	const rank = tierRank(tier);
	return [
		{
			id: 'business',
			label: 'Business profile',
			description: 'Your product name, audience, and keywords',
			completed: rank >= 1,
			href: '/settings#business',
			tier: 1,
			optional: false,
		},
		{
			id: 'xapi',
			label: 'X credentials',
			description: 'Connect to X to discover and score conversations',
			completed: rank >= 2,
			href: '/settings#xapi',
			tier: 2,
			optional: false,
		},
		{
			id: 'llm',
			label: 'LLM provider',
			description: 'Required for AI-generated drafts and replies',
			completed: rank >= 3,
			href: '/settings#llm',
			tier: 3,
			optional: false,
		},
		{
			id: 'posting',
			label: 'Posting access',
			description: 'Authorize Tuitbot to post on your behalf',
			completed: rank >= 4,
			href: '/settings#xapi',
			tier: 4,
			optional: false,
		},
		{
			id: 'sources',
			label: 'Knowledge vault',
			description: 'Improve AI drafts with your own notes and ideas',
			completed: false, // always optional, never blocks tier
			href: '/settings#sources',
			tier: 5,
			optional: true,
		},
	];
}

export function currentTierActions(tier: CapabilityTier): TierAction[] {
	const actions: TierAction[] = [];
	const rank = tierRank(tier);

	if (rank >= 1) {
		actions.push(
			{ label: 'View dashboard', description: 'Explore your home page and settings' },
			{ label: 'Edit profile', description: 'Refine your business profile in Settings' },
		);
	}
	if (rank >= 2) {
		actions.push(
			{ label: 'Browse discovery', description: 'View scored tweets from your targets' },
			{ label: 'Manage targets', description: 'Add and review target accounts' },
		);
	}
	if (rank >= 3) {
		actions.push(
			{ label: 'Create AI drafts', description: 'Generate replies and original content' },
			{ label: 'Use Draft Studio', description: 'Compose and refine posts with AI assistance' },
		);
	}
	if (rank >= 4) {
		actions.push(
			{ label: 'Schedule posts', description: 'Queue content on your calendar' },
			{ label: 'Enable autopilot', description: 'Let Tuitbot post on your behalf' },
		);
	}

	return actions;
}
