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

export interface CalendarItem {
	id: number;
	content_type: string;
	content: string;
	target_author: string | null;
	topic: string | null;
	timestamp: string;
	status: string;
	performance_score: number | null;
	source: string;
}

export interface ScheduleConfig {
	timezone: string;
	active_hours: { start: number; end: number };
	preferred_times: string[];
	preferred_times_override: Record<string, string[]>;
	thread_day: string | null;
	thread_time: string;
}

export interface ComposeRequest {
	content_type: string;
	content: string;
	scheduled_for?: string;
}

export interface ScheduledContentItem {
	id: number;
	content_type: string;
	content: string;
	scheduled_for: string | null;
	status: string;
	posted_tweet_id: string | null;
	created_at: string;
	updated_at: string;
}

// --- Settings types ---

export interface TuitbotConfig {
	x_api: {
		client_id: string;
		client_secret: string | null;
	};
	auth: {
		mode: string;
		callback_host: string;
		callback_port: number;
	};
	business: {
		product_name: string;
		product_description: string;
		product_url: string | null;
		target_audience: string;
		product_keywords: string[];
		competitor_keywords: string[];
		industry_topics: string[];
		brand_voice: string | null;
		reply_style: string | null;
		content_style: string | null;
		persona_opinions: string[];
		persona_experiences: string[];
		content_pillars: string[];
	};
	scoring: {
		threshold: number;
		keyword_relevance_max: number;
		follower_count_max: number;
		recency_max: number;
		engagement_rate_max: number;
		reply_count_max: number;
		content_type_max: number;
	};
	limits: {
		max_replies_per_day: number;
		max_tweets_per_day: number;
		max_threads_per_week: number;
		min_action_delay_seconds: number;
		max_action_delay_seconds: number;
		max_replies_per_author_per_day: number;
		banned_phrases: string[];
		product_mention_ratio: number;
	};
	intervals: {
		mentions_check_seconds: number;
		discovery_search_seconds: number;
		content_post_window_seconds: number;
		thread_interval_seconds: number;
	};
	llm: {
		provider: string;
		api_key: string | null;
		model: string;
		base_url: string | null;
	};
	targets: {
		accounts: string[];
		max_target_replies_per_day: number;
		auto_follow: boolean;
		follow_warmup_days: number;
	};
	approval_mode: boolean;
	storage: {
		db_path: string;
		retention_days: number;
	};
	logging: {
		status_interval_seconds: number;
	};
	schedule: {
		timezone: string;
		active_hours_start: number;
		active_hours_end: number;
		active_days: string[];
		preferred_times: string[];
		preferred_times_override: Record<string, string[]>;
		thread_preferred_day: string | null;
		thread_preferred_time: string;
	};
}

export interface SettingsValidationResult {
	valid: boolean;
	errors: Array<{ field: string; message: string }>;
}

export interface SettingsTestResult {
	success: boolean;
	error?: string;
	latency_ms?: number;
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

	content: {
		calendar: (from: string, to: string) =>
			request<CalendarItem[]>(`/api/content/calendar?from=${from}&to=${to}`),
		schedule: () => request<ScheduleConfig>('/api/content/schedule'),
		compose: (data: ComposeRequest) =>
			request<{ status: string; id: number }>('/api/content/compose', {
				method: 'POST',
				body: JSON.stringify(data)
			}),
		updateScheduled: (id: number, data: { content?: string; scheduled_for?: string }) =>
			request<ScheduledContentItem>(`/api/content/scheduled/${id}`, {
				method: 'PATCH',
				body: JSON.stringify(data)
			}),
		cancelScheduled: (id: number) =>
			request<{ status: string; id: number }>(`/api/content/scheduled/${id}`, {
				method: 'DELETE'
			}),
		tweets: (limit: number = 50) =>
			request<unknown[]>(`/api/content/tweets?limit=${limit}`),
		threads: (limit: number = 20) =>
			request<unknown[]>(`/api/content/threads?limit=${limit}`)
	},

	settings: {
		get: () => request<TuitbotConfig>('/api/settings'),
		patch: (data: Partial<TuitbotConfig>) =>
			request<TuitbotConfig>('/api/settings', {
				method: 'PATCH',
				body: JSON.stringify(data)
			}),
		validate: (data: Partial<TuitbotConfig>) =>
			request<SettingsValidationResult>('/api/settings/validate', {
				method: 'POST',
				body: JSON.stringify(data)
			}),
		defaults: () => request<TuitbotConfig>('/api/settings/defaults'),
		testLlm: (data: {
			provider: string;
			api_key?: string | null;
			model: string;
			base_url?: string | null;
		}) =>
			request<SettingsTestResult>('/api/settings/test-llm', {
				method: 'POST',
				body: JSON.stringify(data)
			})
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
