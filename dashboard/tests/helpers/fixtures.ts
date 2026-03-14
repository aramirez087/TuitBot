/**
 * fixtures.ts — Shared test data for TuitBot dashboard unit and component tests.
 *
 * Usage:
 *   import { fixtures } from '../helpers/fixtures';
 *   const item = fixtures.approvalItems[0];
 */

import type {
	ApprovalItem,
	ApprovalStats,
	AnalyticsSummary,
	TuitbotConfig,
	TargetAccount
} from '../../src/lib/api/types';

// ---------------------------------------------------------------------------
// Approval items
// ---------------------------------------------------------------------------

export const approvalItem = (overrides: Partial<ApprovalItem> = {}): ApprovalItem => ({
	id: 1,
	action_type: 'reply',
	target_tweet_id: '1234567890123456789',
	target_author: 'test_user',
	generated_content: 'Great insight! This connects directly to the core problem.',
	topic: 'product-led-growth',
	archetype: 'insight_builder',
	score: 0.87,
	status: 'pending',
	created_at: '2026-03-14T00:00:00.000Z',
	media_paths: [],
	detected_risks: [],
	qa_score: 0.92,
	qa_hard_flags: [],
	qa_soft_flags: [],
	qa_requires_override: false,
	...overrides
});

export const approvalItems: ApprovalItem[] = [
	approvalItem({ id: 1, target_author: 'alice', score: 0.87 }),
	approvalItem({ id: 2, target_author: 'bob', action_type: 'tweet', score: 0.74 }),
	approvalItem({ id: 3, target_author: 'carol', status: 'approved', score: 0.95 }),
	approvalItem({ id: 4, target_author: 'dave', status: 'rejected', score: 0.42, detected_risks: ['off_topic'] })
];

export const approvalStats: ApprovalStats = {
	pending: 2,
	approved: 1,
	rejected: 1
};

// ---------------------------------------------------------------------------
// Analytics
// ---------------------------------------------------------------------------

export const analyticsSummary: AnalyticsSummary = {
	followers: { current: 1250, change_7d: 42, change_30d: 180 },
	actions_today: { replies: 5, tweets: 2, threads: 0 },
	engagement: {
		avg_reply_score: 0.78,
		avg_tweet_score: 0.65,
		total_replies_sent: 127,
		total_tweets_posted: 34
	},
	top_topics: [
		{ topic: 'product-led-growth', format: 'reply', total_posts: 45, avg_performance: 0.81 },
		{ topic: 'developer-tools', format: 'tweet', total_posts: 28, avg_performance: 0.73 }
	]
};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

export const config: Partial<TuitbotConfig> = {
	x_api: {
		client_id: 'test-client-id',
		client_secret: null,
		provider_backend: 'local',
		scraper_allow_mutations: false
	},
	business: {
		product_name: 'TuitBot',
		product_description: 'Autonomous X growth assistant',
		product_url: 'https://example.com',
		target_audience: 'indie hackers',
		product_keywords: ['automation', 'growth'],
		competitor_keywords: [],
		industry_topics: ['saas', 'devtools'],
		brand_voice: 'helpful and direct'
	}
} as Partial<TuitbotConfig>;

// ---------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------

export const targetAccount = (overrides: Partial<TargetAccount> = {}): TargetAccount => ({
	id: 1,
	username: 'target_user',
	display_name: 'Target User',
	follower_count: 5000,
	following_count: 800,
	tweet_count: 1200,
	bio: 'Building in public.',
	profile_image_url: null,
	is_active: true,
	added_at: '2026-01-01T00:00:00.000Z',
	...overrides
});

export const targets: TargetAccount[] = [
	targetAccount({ id: 1, username: 'user_a', follower_count: 12000 }),
	targetAccount({ id: 2, username: 'user_b', follower_count: 3400 }),
	targetAccount({ id: 3, username: 'user_c', is_active: false, follower_count: 890 })
];

// ---------------------------------------------------------------------------
// Barrel export
// ---------------------------------------------------------------------------

export const fixtures = {
	approvalItem,
	approvalItems,
	approvalStats,
	analyticsSummary,
	config,
	targetAccount,
	targets
};
