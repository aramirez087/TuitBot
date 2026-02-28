const BASE_URL = 'http://localhost:3001';
let token: string = '';
let accountId: string = '00000000-0000-0000-0000-000000000000';
let authMode: 'bearer' | 'cookie' = 'bearer';
let csrfToken: string = '';

export function setToken(t: string) {
	token = t;
}

export function getToken(): string {
	return token;
}

export function setAccountId(id: string) {
	accountId = id;
}

export function getAccountId(): string {
	return accountId;
}

export function setAuthMode(mode: 'bearer' | 'cookie') {
	authMode = mode;
}

export function getAuthMode(): 'bearer' | 'cookie' {
	return authMode;
}

export function setCsrfToken(t: string) {
	csrfToken = t;
}

export function getCsrfToken(): string {
	return csrfToken;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
		'X-Account-Id': accountId
	};

	if (authMode === 'bearer' && token) {
		headers['Authorization'] = `Bearer ${token}`;
	}
	if (authMode === 'cookie' && csrfToken) {
		const method = (options?.method || 'GET').toUpperCase();
		if (method !== 'GET' && method !== 'HEAD') {
			headers['X-CSRF-Token'] = csrfToken;
		}
	}

	const fetchOptions: RequestInit = {
		...options,
		headers: {
			...headers,
			...options?.headers
		}
	};

	// Include cookies for cookie-based auth
	if (authMode === 'cookie') {
		fetchOptions.credentials = 'include';
	}

	const res = await fetch(`${BASE_URL}${path}`, fetchOptions);
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
	media_paths: string[];
	reviewed_by?: string;
	review_notes?: string;
	reason?: string;
	detected_risks: string[];
	qa_score: number;
	qa_hard_flags: string[];
	qa_soft_flags: string[];
	qa_requires_override: boolean;
	qa_override_by?: string;
	qa_override_note?: string;
	qa_override_at?: string;
}

export interface EditHistoryEntry {
	id: number;
	approval_id: number;
	editor: string;
	field: string;
	old_value: string;
	new_value: string;
	created_at: string;
}

export interface MediaUploadResponse {
	path: string;
	media_type: string;
	size: number;
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

export interface ThreadBlock {
	id: string;
	text: string;
	media_paths: string[];
	order: number;
}

export interface ThreadBlocksPayload {
	version: number;
	blocks: ThreadBlock[];
}

export interface ComposeRequest {
	content_type: string;
	content: string;
	scheduled_for?: string;
	media_paths?: string[];
	blocks?: ThreadBlock[];
}

/**
 * Parse stored thread content, detecting new blocks format vs legacy string array.
 * Returns `ThreadBlock[]` for blocks format, `string[]` for legacy format.
 */
export function parseThreadContent(content: string): ThreadBlock[] | string[] {
	try {
		const parsed = JSON.parse(content);
		if (parsed && typeof parsed === 'object' && !Array.isArray(parsed) && parsed.blocks) {
			return (parsed as ThreadBlocksPayload).blocks;
		}
		if (Array.isArray(parsed)) {
			return parsed as string[];
		}
	} catch {
		// Not JSON — return as single-item array
	}
	return [content];
}

/**
 * Check whether stored content uses the versioned blocks payload format.
 */
export function isBlocksPayload(content: string): boolean {
	try {
		const parsed = JSON.parse(content);
		return parsed && typeof parsed === 'object' && !Array.isArray(parsed) && 'blocks' in parsed;
	} catch {
		return false;
	}
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

// --- Target types ---

export interface TargetAccount {
	account_id: string;
	username: string;
	followed_at: string | null;
	first_engagement_at: string | null;
	total_replies_sent: number;
	last_reply_at: string | null;
	status: string;
	interactions_today: number;
}

export interface TargetTimelineItem {
	tweet_id: string;
	text: string;
	posted_at: string;
	relevance_score: number;
	replied_to: boolean;
	tweet_reply_count: number;
	tweet_like_count: number;
	reply_content: string | null;
	reply_created_at: string | null;
}

export interface TargetStats {
	total_replies: number;
	avg_score: number;
	best_reply_content: string | null;
	best_reply_score: number | null;
	first_interaction: string | null;
	interaction_frequency_days: number | null;
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
	};
	mode: 'autopilot' | 'composer';
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

// --- Strategy types ---

export interface TopicPerformance {
	topic: string;
	format: string;
	avg_score: number;
	post_count: number;
}

export interface ContentHighlight {
	content_type: string;
	content_preview: string;
	performance_score: number;
	likes: number;
	replies_received: number;
}

export interface Recommendation {
	category: string;
	priority: string;
	title: string;
	description: string;
}

export interface StrategyReport {
	id: number;
	week_start: string;
	week_end: string;
	replies_sent: number;
	tweets_posted: number;
	threads_posted: number;
	target_replies: number;
	follower_start: number;
	follower_end: number;
	follower_delta: number;
	avg_reply_score: number;
	avg_tweet_score: number;
	reply_acceptance_rate: number;
	estimated_follow_conversion: number;
	top_topics: TopicPerformance[];
	bottom_topics: TopicPerformance[];
	top_content: ContentHighlight[];
	recommendations: Recommendation[];
}

export interface StrategyInputs {
	content_pillars: string[];
	industry_topics: string[];
	product_keywords: string[];
	competitor_keywords: string[];
	target_accounts: string[];
}

// --- Cost types ---

export interface CostSummary {
	cost_today: number;
	cost_7d: number;
	cost_30d: number;
	cost_all_time: number;
	calls_today: number;
	calls_7d: number;
	calls_30d: number;
	calls_all_time: number;
}

export interface DailyCostSummary {
	date: string;
	cost: number;
	calls: number;
	input_tokens: number;
	output_tokens: number;
}

export interface ModelCostBreakdown {
	provider: string;
	model: string;
	cost: number;
	calls: number;
	input_tokens: number;
	output_tokens: number;
}

export interface TypeCostBreakdown {
	generation_type: string;
	cost: number;
	calls: number;
	avg_cost: number;
}

// --- X API cost types ---

export interface XApiUsageSummary {
	cost_today: number;
	cost_7d: number;
	cost_30d: number;
	cost_all_time: number;
	calls_today: number;
	calls_7d: number;
	calls_30d: number;
	calls_all_time: number;
}

export interface DailyXApiUsage {
	date: string;
	calls: number;
	cost: number;
}

export interface EndpointBreakdown {
	endpoint: string;
	method: string;
	calls: number;
	cost: number;
	error_count: number;
}

// --- MCP types ---

export interface McpPolicyRuleConditions {
	tools?: string[];
	categories?: string[];
	modes?: string[];
	schedule_window?: {
		start_hour: number;
		end_hour: number;
		timezone: string;
		days: string[];
	};
}

export interface McpPolicyAction {
	type: 'allow' | 'deny' | 'require_approval' | 'dry_run';
	reason?: string;
}

export interface McpPolicyRule {
	id: string;
	priority: number;
	label: string;
	enabled: boolean;
	conditions: McpPolicyRuleConditions;
	action: McpPolicyAction;
}

export interface McpPolicyRateLimit {
	key: string;
	dimension: string;
	match_value: string;
	max_count: number;
	period_seconds: number;
}

export interface McpPolicyTemplate {
	name: string;
	description: string;
	rules: McpPolicyRule[];
	rate_limits: McpPolicyRateLimit[];
}

export interface McpPolicyStatus {
	enforce_for_mutations: boolean;
	require_approval_for: string[];
	blocked_tools: string[];
	dry_run_mutations: boolean;
	max_mutations_per_hour: number;
	mode: string;
	rate_limit: {
		used: number;
		max: number;
		period_seconds?: number;
		period_start?: string;
	};
	template?: string;
	rules: McpPolicyRule[];
	rate_limits: McpPolicyRateLimit[];
}

export interface McpPolicyPatch {
	enforce_for_mutations?: boolean;
	require_approval_for?: string[];
	blocked_tools?: string[];
	dry_run_mutations?: boolean;
	max_mutations_per_hour?: number;
	template?: string;
	rules?: McpPolicyRule[];
	rate_limits?: McpPolicyRateLimit[];
}

export interface McpTelemetrySummary {
	total_calls: number;
	total_successes: number;
	total_failures: number;
	overall_success_rate: number;
	avg_latency_ms: number;
	unique_tools: number;
	policy_decisions: Record<string, number>;
}

export interface McpToolMetrics {
	tool_name: string;
	category: string;
	total_calls: number;
	success_count: number;
	failure_count: number;
	success_rate: number;
	avg_latency_ms: number;
	p50_latency_ms: number;
	p95_latency_ms: number;
	min_latency_ms: number;
	max_latency_ms: number;
}

export interface McpErrorBreakdown {
	tool_name: string;
	error_code: string;
	count: number;
	latest_at: string;
}

export interface McpTelemetryEntry {
	id: number;
	tool_name: string;
	category: string;
	latency_ms: number;
	success: boolean;
	error_code: string | null;
	policy_decision: string | null;
	metadata: string | null;
	created_at: string;
}

// --- File upload helper ---

async function uploadFile(path: string, file: File): Promise<MediaUploadResponse> {
	const formData = new FormData();
	formData.append('file', file);

	const headers: Record<string, string> = {
		'X-Account-Id': accountId
		// No Content-Type — browser sets multipart boundary automatically.
	};

	if (authMode === 'bearer' && token) {
		headers['Authorization'] = `Bearer ${token}`;
	}
	if (authMode === 'cookie' && csrfToken) {
		headers['X-CSRF-Token'] = csrfToken;
	}

	const res = await fetch(`${BASE_URL}${path}`, {
		method: 'POST',
		headers,
		body: formData,
		...(authMode === 'cookie' ? { credentials: 'include' as RequestCredentials } : {})
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || res.statusText);
	}
	return res.json();
}

// --- API client ---

// --- Account types ---

export interface Account {
	id: string;
	label: string;
	x_username: string | null;
	x_user_id: string | null;
	config_overrides: string | null;
	status: string;
	created_at: string;
	updated_at: string;
}

export const api = {
	health: () => request<HealthResponse>('/api/health'),

	auth: {
		login: async (passphrase: string): Promise<{ csrf_token: string; expires_at: string }> => {
			const res = await fetch(`${BASE_URL}/api/auth/login`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ passphrase }),
				credentials: 'include'
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({ error: res.statusText }));
				throw new Error(body.error || res.statusText);
			}
			return res.json();
		},
		logout: async (): Promise<void> => {
			await fetch(`${BASE_URL}/api/auth/logout`, {
				method: 'POST',
				headers: csrfToken ? { 'X-CSRF-Token': csrfToken } : {},
				credentials: 'include'
			});
		},
		status: async (): Promise<{ authenticated: boolean; csrf_token?: string; expires_at?: string }> => {
			const res = await fetch(`${BASE_URL}/api/auth/status`, {
				credentials: 'include'
			});
			if (!res.ok) {
				return { authenticated: false };
			}
			return res.json();
		}
	},

	accounts: {
		list: () => request<Account[]>('/api/accounts'),
		get: (id: string) => request<Account>(`/api/accounts/${id}`),
		create: (label: string) =>
			request<Account>('/api/accounts', {
				method: 'POST',
				body: JSON.stringify({ label })
			}),
		update: (id: string, data: { label?: string; config_overrides?: string }) =>
			request<Account>(`/api/accounts/${id}`, {
				method: 'PATCH',
				body: JSON.stringify(data)
			}),
		delete: (id: string) =>
			request<{ status: string }>(`/api/accounts/${id}`, { method: 'DELETE' })
	},

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
		rateLimits: () => request<RateLimitUsage>('/api/activity/rate-limits'),
		exportUrl: (format: 'csv' | 'json', type_?: string, status?: string): string => {
			const query = new URLSearchParams({ format });
			if (type_) query.set('type', type_);
			if (status) query.set('status', status);
			return `${BASE_URL}/api/activity/export?${query.toString()}`;
		}
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

	lan: {
		status: () =>
			request<{
				bind_host: string;
				bind_port: number;
				lan_enabled: boolean;
				local_ip: string | null;
				passphrase_configured: boolean;
			}>('/api/settings/lan'),
		toggle: (host: string) =>
			request<{ restart_required: boolean }>('/api/settings/lan', {
				method: 'PATCH',
				body: JSON.stringify({ host })
			}),
		resetPassphrase: () =>
			request<{ passphrase: string }>('/api/settings/lan/reset-passphrase', {
				method: 'POST'
			})
	},

	settings: {
		configStatus: () => request<{ configured: boolean }>('/api/settings/status'),
		init: (data: Partial<TuitbotConfig>) =>
			request<{ status: string; config?: TuitbotConfig; errors?: Array<{ field: string; message: string }> }>(
				'/api/settings/init',
				{
					method: 'POST',
					body: JSON.stringify(data)
				}
			),
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

	targets: {
		list: () => request<TargetAccount[]>('/api/targets'),
		add: (username: string) =>
			request<{ status: string; username: string }>('/api/targets', {
				method: 'POST',
				body: JSON.stringify({ username })
			}),
		remove: (username: string) =>
			request<{ status: string; username: string }>(
				`/api/targets/${encodeURIComponent(username)}`,
				{ method: 'DELETE' }
			),
		timeline: (username: string, limit: number = 50) =>
			request<TargetTimelineItem[]>(
				`/api/targets/${encodeURIComponent(username)}/timeline?limit=${limit}`
			),
		stats: (username: string) =>
			request<TargetStats>(`/api/targets/${encodeURIComponent(username)}/stats`)
	},

	strategy: {
		current: () => request<StrategyReport>('/api/strategy/current'),
		history: (limit: number = 12) =>
			request<StrategyReport[]>(`/api/strategy/history?limit=${limit}`),
		refresh: () =>
			request<StrategyReport>('/api/strategy/refresh', { method: 'POST' }),
		inputs: () => request<StrategyInputs>('/api/strategy/inputs')
	},

	costs: {
		summary: () => request<CostSummary>('/api/costs/summary'),
		daily: (days: number = 30) =>
			request<DailyCostSummary[]>(`/api/costs/daily?days=${days}`),
		byModel: (days: number = 30) =>
			request<ModelCostBreakdown[]>(`/api/costs/by-model?days=${days}`),
		byType: (days: number = 30) =>
			request<TypeCostBreakdown[]>(`/api/costs/by-type?days=${days}`),
		xApi: {
			summary: () => request<XApiUsageSummary>('/api/costs/x-api/summary'),
			daily: (days: number = 30) =>
				request<DailyXApiUsage[]>(`/api/costs/x-api/daily?days=${days}`),
			byEndpoint: (days: number = 30) =>
				request<EndpointBreakdown[]>(`/api/costs/x-api/by-endpoint?days=${days}`)
		}
	},

	media: {
		upload: (file: File) => uploadFile('/api/media/upload', file),
		fileUrl: (path: string) =>
			`${BASE_URL}/api/media/file?path=${encodeURIComponent(path)}`
	},

	approval: {
		list: (params: { status?: string; type?: string; reviewed_by?: string; since?: string } = {}) => {
			const query = new URLSearchParams();
			if (params.status) query.set('status', params.status);
			if (params.type) query.set('type', params.type);
			if (params.reviewed_by) query.set('reviewed_by', params.reviewed_by);
			if (params.since) query.set('since', params.since);
			const qs = query.toString();
			return request<ApprovalItem[]>(`/api/approval${qs ? `?${qs}` : ''}`);
		},
		stats: () => request<ApprovalStats>('/api/approval/stats'),
		approve: (id: number, actor?: string, notes?: string) =>
			request<{ status: string; id: number }>(`/api/approval/${id}/approve`, {
				method: 'POST',
				body: JSON.stringify({ actor, notes })
			}),
		reject: (id: number, actor?: string, notes?: string) =>
			request<{ status: string; id: number }>(`/api/approval/${id}/reject`, {
				method: 'POST',
				body: JSON.stringify({ actor, notes })
			}),
		edit: (id: number, content: string, media_paths?: string[], editor?: string) =>
			request<ApprovalItem>(`/api/approval/${id}`, {
				method: 'PATCH',
				body: JSON.stringify({
					content,
					...(media_paths !== undefined && { media_paths }),
					...(editor !== undefined && { editor })
				})
			}),
		approveAll: (max?: number, ids?: number[]) =>
			request<{ status: string; count: number; ids: number[]; max_batch: number }>(
				'/api/approval/approve-all',
				{
					method: 'POST',
					body: JSON.stringify({
						...(max !== undefined && { max }),
						...(ids !== undefined && { ids }),
						review: { actor: 'dashboard' }
					})
				}
			),
		editHistory: (id: number) =>
			request<EditHistoryEntry[]>(`/api/approval/${id}/history`),
		exportUrl: (format: 'csv' | 'json', status?: string, type_?: string): string => {
			const query = new URLSearchParams({ format });
			if (status) query.set('status', status);
			if (type_) query.set('type', type_);
			return `${BASE_URL}/api/approval/export?${query.toString()}`;
		}
	},

	assist: {
		tweet: (topic: string) =>
			request<{ content: string; topic: string }>('/api/assist/tweet', {
				method: 'POST',
				body: JSON.stringify({ topic })
			}),
		reply: (tweetText: string, tweetAuthor: string, mentionProduct: boolean = false) =>
			request<{ content: string }>('/api/assist/reply', {
				method: 'POST',
				body: JSON.stringify({
					tweet_text: tweetText,
					tweet_author: tweetAuthor,
					mention_product: mentionProduct
				})
			}),
		thread: (topic: string) =>
			request<{ tweets: string[]; topic: string }>('/api/assist/thread', {
				method: 'POST',
				body: JSON.stringify({ topic })
			}),
		improve: (draft: string, context?: string) =>
			request<{ content: string }>('/api/assist/improve', {
				method: 'POST',
				body: JSON.stringify({ draft, context })
			}),
		topics: () =>
			request<{ topics: Array<{ topic: string; score: number }> }>('/api/assist/topics'),
		optimalTimes: () =>
			request<{ times: Array<{ hour: number; avg_engagement: number; post_count: number }> }>(
				'/api/assist/optimal-times'
			),
		mode: () => request<{ mode: string; approval_mode: boolean }>('/api/assist/mode')
	},

	drafts: {
		list: () => request<ScheduledContentItem[]>('/api/content/drafts'),
		create: (
			contentType: string,
			content: string,
			source: string = 'manual',
			blocks?: ThreadBlock[]
		) =>
			request<{ id: number; status: string }>('/api/content/drafts', {
				method: 'POST',
				body: JSON.stringify({
					content_type: contentType,
					content,
					source,
					...(blocks && { blocks })
				})
			}),
		edit: (id: number, content?: string, blocks?: ThreadBlock[]) =>
			request<{ id: number; status: string }>(`/api/content/drafts/${id}`, {
				method: 'PATCH',
				body: JSON.stringify({
					...(content !== undefined && { content }),
					...(blocks && { blocks })
				})
			}),
		delete: (id: number) =>
			request<{ id: number; status: string }>(`/api/content/drafts/${id}`, {
				method: 'DELETE'
			}),
		schedule: (id: number, scheduledFor: string) =>
			request<{ id: number; status: string; scheduled_for: string }>(
				`/api/content/drafts/${id}/schedule`,
				{
					method: 'POST',
					body: JSON.stringify({ scheduled_for: scheduledFor })
				}
			),
		publish: (id: number) =>
			request<{ id: number; approval_queue_id: number; status: string }>(
				`/api/content/drafts/${id}/publish`,
				{ method: 'POST' }
			)
	},

	mcp: {
		policy: () => request<McpPolicyStatus>('/api/mcp/policy'),
		patchPolicy: (data: McpPolicyPatch) =>
			request<McpPolicyPatch>('/api/mcp/policy', {
				method: 'PATCH',
				body: JSON.stringify(data)
			}),
		listTemplates: () => request<McpPolicyTemplate[]>('/api/mcp/policy/templates'),
		applyTemplate: (name: string) =>
			request<{ applied_template: string; description: string }>(
				`/api/mcp/policy/templates/${name}`,
				{ method: 'POST' }
			),
		telemetrySummary: (hours: number = 24) =>
			request<McpTelemetrySummary>(`/api/mcp/telemetry/summary?hours=${hours}`),
		telemetryMetrics: (hours: number = 24) =>
			request<McpToolMetrics[]>(`/api/mcp/telemetry/metrics?hours=${hours}`),
		telemetryErrors: (hours: number = 24) =>
			request<McpErrorBreakdown[]>(`/api/mcp/telemetry/errors?hours=${hours}`),
		telemetryRecent: (limit: number = 50) =>
			request<McpTelemetryEntry[]>(`/api/mcp/telemetry/recent?limit=${limit}`)
	},

	discovery: {
		feed: (params: { minScore?: number; maxScore?: number; keyword?: string; limit?: number } = {}) => {
			const query = new URLSearchParams();
			query.set('min_score', (params.minScore ?? 50).toString());
			query.set('limit', (params.limit ?? 20).toString());
			if (params.maxScore !== undefined) query.set('max_score', params.maxScore.toString());
			if (params.keyword) query.set('keyword', params.keyword);
			return request<
				Array<{
					id: string;
					author_username: string;
					content: string;
					relevance_score: number;
					matched_keyword: string | null;
					like_count: number;
					retweet_count: number;
					reply_count: number;
					replied_to: boolean;
					discovered_at: string;
				}>
			>(`/api/discovery/feed?${query.toString()}`);
		},
		keywords: () => request<string[]>('/api/discovery/keywords'),
		composeReply: (tweetId: string, mentionProduct: boolean = false) =>
			request<{ content: string; tweet_id: string }>(
				`/api/discovery/${tweetId}/compose-reply`,
				{
					method: 'POST',
					body: JSON.stringify({ mention_product: mentionProduct })
				}
			),
		queueReply: (tweetId: string, content: string) =>
			request<{ approval_queue_id: number; tweet_id: string; status: string }>(
				`/api/discovery/${tweetId}/queue-reply`,
				{
					method: 'POST',
					body: JSON.stringify({ content })
				}
			)
	}
};
