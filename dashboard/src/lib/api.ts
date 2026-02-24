const BASE_URL = 'http://localhost:3001';
let token: string = '';

export function setToken(t: string) {
	token = t;
}

export function getToken(): string {
	return token;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE_URL}${path}`, {
		...options,
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${token}`,
			...options?.headers
		}
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || res.statusText);
	}
	return res.json();
}

// --- Shared types ---

export interface HealthResponse {
	status: string;
	version: string;
}

export interface FollowerSummary {
	current: number;
	change_7d: number;
	change_30d: number;
}

export interface ActionsSummary {
	replies: number;
	tweets: number;
	threads: number;
}

export interface EngagementSummary {
	avg_reply_score: number;
	avg_tweet_score: number;
	total_replies_sent: number;
	total_tweets_posted: number;
}

export interface ContentScore {
	topic: string;
	format: string;
	total_posts: number;
	avg_performance: number;
}

export interface AnalyticsSummary {
	followers: FollowerSummary;
	actions_today: ActionsSummary;
	engagement: EngagementSummary;
	top_topics: ContentScore[];
}

export interface FollowerSnapshot {
	snapshot_date: string;
	follower_count: number;
	following_count: number;
	tweet_count: number;
}

export interface PerformanceItem {
	content_type: string;
	content_preview: string;
	likes: number;
	replies_received: number;
	retweets: number;
	impressions: number;
	performance_score: number;
	posted_at: string;
}

export interface ActionLogEntry {
	id: number;
	action_type: string;
	status: string;
	message: string | null;
	metadata: string | null;
	created_at: string;
}

export interface ActivityResponse {
	actions: ActionLogEntry[];
	total: number;
	limit: number;
	offset: number;
}

export interface ActionUsage {
	used: number;
	max: number;
}

export interface RateLimitUsage {
	replies: ActionUsage;
	tweets: ActionUsage;
	threads: ActionUsage;
}

export interface ApprovalItem {
	id: number;
	action_type: string;
	target_tweet_id: string;
	target_author: string;
	generated_content: string;
	topic: string;
	archetype: string;
	score: number;
	status: string;
	created_at: string;
}

export interface ApprovalStats {
	pending: number;
	approved: number;
	rejected: number;
}

// --- API client ---

export const api = {
	health: () => request<HealthResponse>('/api/health'),

	analytics: {
		summary: () => request<AnalyticsSummary>('/api/analytics/summary'),
		followers: (days: number = 30) =>
			request<FollowerSnapshot[]>(`/api/analytics/followers?days=${days}`),
		topics: (limit: number = 10) =>
			request<ContentScore[]>(`/api/analytics/topics?limit=${limit}`),
		recentPerformance: (limit: number = 20) =>
			request<PerformanceItem[]>(`/api/analytics/recent-performance?limit=${limit}`)
	},

	activity: {
		list: (params: { limit?: number; offset?: number; type?: string; status?: string } = {}) => {
			const query = new URLSearchParams();
			if (params.limit) query.set('limit', params.limit.toString());
			if (params.offset) query.set('offset', params.offset.toString());
			if (params.type) query.set('type', params.type);
			if (params.status) query.set('status', params.status);
			const qs = query.toString();
			return request<ActivityResponse>(`/api/activity${qs ? `?${qs}` : ''}`);
		},
		rateLimits: () => request<RateLimitUsage>('/api/activity/rate-limits')
	},

	approval: {
		list: (params: { status?: string; type?: string } = {}) => {
			const query = new URLSearchParams();
			if (params.status) query.set('status', params.status);
			if (params.type) query.set('type', params.type);
			const qs = query.toString();
			return request<ApprovalItem[]>(`/api/approval${qs ? `?${qs}` : ''}`);
		},
		stats: () => request<ApprovalStats>('/api/approval/stats'),
		approve: (id: number) =>
			request<{ status: string; id: number }>(`/api/approval/${id}/approve`, { method: 'POST' }),
		reject: (id: number) =>
			request<{ status: string; id: number }>(`/api/approval/${id}/reject`, { method: 'POST' }),
		edit: (id: number, content: string) =>
			request<ApprovalItem>(`/api/approval/${id}`, {
				method: 'PATCH',
				body: JSON.stringify({ content })
			}),
		approveAll: () =>
			request<{ status: string; count: number }>('/api/approval/approve-all', { method: 'POST' })
	}
};
