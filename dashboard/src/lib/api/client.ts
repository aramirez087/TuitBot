import { request, uploadFile, BASE_URL, getAuthMode } from './http';
import type {
	HealthResponse,
	RuntimeStatus,
	Account,
	AnalyticsSummary,
	FollowerSnapshot,
	ContentScore,
	PerformanceItem,
	ActivityResponse,
	RateLimitUsage,
	CalendarItem,
	ScheduleConfig,
	ComposeRequest,
	ScheduledContentItem,
	ThreadBlock,
	TuitbotConfig,
	ConfigStatus,
	SettingsValidationResult,
	SettingsTestResult,
	TargetAccount,
	TargetTimelineItem,
	TargetStats,
	StrategyReport,
	StrategyInputs,
	CostSummary,
	DailyCostSummary,
	ModelCostBreakdown,
	TypeCostBreakdown,
	XApiUsageSummary,
	DailyXApiUsage,
	EndpointBreakdown,
	MediaUploadResponse,
	ApprovalItem,
	ApprovalStats,
	EditHistoryEntry,
	McpPolicyStatus,
	McpPolicyPatch,
	McpPolicyTemplate,
	McpTelemetrySummary,
	McpToolMetrics,
	McpErrorBreakdown,
	McpTelemetryEntry,
	McpAvailableTool,
	LinkResponse,
	ConnectorStatusResponse,
	DisconnectResponse,
	AccountAuthStatus,
	VaultCitation,
	VaultSourceStatus,
	VaultNoteItem,
	VaultNoteDetail,
	VaultSelectionResponse,
	VaultSourcesResponse,
	ProvenanceRef,
	ProvenanceLink,
	DraftSummary,
	AutosaveResponse,
	ContentRevision,
	ContentActivity,
	ContentTag,
	AnalyzeProfileResponse,
	AssistHooksResponse
} from './types';
import { getCsrfToken } from './http';

export const api = {
	health: () => request<HealthResponse>('/api/health'),

	runtime: {
		status: () => request<RuntimeStatus>('/api/runtime/status'),
		start: () => request<{ status: string }>('/api/runtime/start', { method: 'POST' }),
		stop: () => request<{ status: string }>('/api/runtime/stop', { method: 'POST' })
	},

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
			const csrfToken = getCsrfToken();
			await fetch(`${BASE_URL}/api/auth/logout`, {
				method: 'POST',
				headers: csrfToken ? { 'X-CSRF-Token': csrfToken } : {},
				credentials: 'include'
			});
		},
		status: async (): Promise<{
			authenticated: boolean;
			csrf_token?: string;
			expires_at?: string;
		}> => {
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
			request<{ status: string }>(`/api/accounts/${id}`, { method: 'DELETE' }),
		syncProfile: (id: string) =>
			request<Account>(`/api/accounts/${id}/sync-profile`, { method: 'POST' }),
		authStatus: (id: string) =>
			request<AccountAuthStatus>(`/api/accounts/${id}/x-auth/status`),
		startAuth: (id: string) =>
			request<{ authorization_url: string; state: string }>(
				`/api/accounts/${id}/x-auth/start`,
				{ method: 'POST' }
			),
		completeAuth: (id: string, code: string, state: string) =>
			request<{ status: string; token_path: string }>(
				`/api/accounts/${id}/x-auth/callback`,
				{
					method: 'POST',
					body: JSON.stringify({ code, state })
				}
			),
		unlinkOAuth: (id: string) =>
			request<{ deleted: boolean }>(`/api/accounts/${id}/x-auth/tokens`, {
				method: 'DELETE'
			}),
		scraperSession: {
			get: (id: string) =>
				request<{ exists: boolean; username?: string; created_at?: string }>(
					'/api/settings/scraper-session',
					{ headers: { 'X-Account-Id': id } }
				),
			import: (
				id: string,
				data: { auth_token: string; ct0: string; username?: string }
			) =>
				request<{ status: string; username?: string; created_at?: string; backend_updated?: boolean }>(
					'/api/settings/scraper-session',
					{
						method: 'POST',
						body: JSON.stringify(data),
						headers: { 'X-Account-Id': id }
					}
				),
			delete: (id: string) =>
				request<{ deleted: boolean }>('/api/settings/scraper-session', {
					method: 'DELETE',
					headers: { 'X-Account-Id': id }
				})
		}
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
		list: (
			params: { limit?: number; offset?: number; type?: string; status?: string } = {}
		) => {
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
		tweets: (limit: number = 50) => request<unknown[]>(`/api/content/tweets?limit=${limit}`),
		threads: (limit: number = 20) => request<unknown[]>(`/api/content/threads?limit=${limit}`)
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
		configStatus: () => request<ConfigStatus>('/api/settings/status'),
		init: async (
			data: Record<string, unknown>
		): Promise<{
			status: string;
			config?: TuitbotConfig;
			csrf_token?: string;
			errors?: Array<{ field: string; message: string }>;
		}> => {
			const res = await fetch(`${BASE_URL}/api/settings/init`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(data),
				// Only include credentials for same-origin or cookie auth.
				// Cross-origin + credentials + wildcard CORS origin is blocked by browsers.
				...((!BASE_URL || getAuthMode() === 'cookie') ? { credentials: 'include' as RequestCredentials } : {})
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({ error: res.statusText }));
				throw new Error(body.error || res.statusText);
			}
			return res.json();
		},
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
			}),
		factoryReset: (confirmation: string) =>
			request<{ status: string; cleared: Record<string, unknown> }>(
				'/api/settings/factory-reset',
				{
					method: 'POST',
					body: JSON.stringify({ confirmation })
				}
			),
		scraperSession: {
			get: () =>
				request<{ exists: boolean; username?: string; created_at?: string }>(
					'/api/settings/scraper-session'
				),
			import: (data: { auth_token: string; ct0: string; username?: string }) =>
				request<{ status: string; username?: string; created_at?: string; backend_updated?: boolean }>(
					'/api/settings/scraper-session',
					{
						method: 'POST',
						body: JSON.stringify(data)
					}
				),
			delete: () =>
				request<{ deleted: boolean }>('/api/settings/scraper-session', {
					method: 'DELETE'
				})
		}
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
		refresh: () => request<StrategyReport>('/api/strategy/refresh', { method: 'POST' }),
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
		upload: (file: File) => uploadFile<MediaUploadResponse>('/api/media/upload', file),
		fileUrl: (path: string) =>
			`${BASE_URL}/api/media/file?path=${encodeURIComponent(path)}`
	},

	approval: {
		list: (
			params: {
				status?: string;
				type?: string;
				reviewed_by?: string;
				since?: string;
				action_type?: string;
			} = {}
		) => {
			const query = new URLSearchParams();
			if (params.status) query.set('status', params.status);
			if (params.type) query.set('type', params.type);
			if (params.reviewed_by) query.set('reviewed_by', params.reviewed_by);
			if (params.since) query.set('since', params.since);
			if (params.action_type) query.set('action_type', params.action_type);
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
		tweet: (topic: string, selectedNodeIds?: number[]) =>
			request<{ content: string; topic: string; vault_citations?: VaultCitation[] }>(
				'/api/assist/tweet',
				{
					method: 'POST',
					body: JSON.stringify({
						topic,
						...(selectedNodeIds && { selected_node_ids: selectedNodeIds })
					})
				}
			),
		reply: (tweetText: string, tweetAuthor: string, mentionProduct: boolean = false) =>
			request<{ content: string }>('/api/assist/reply', {
				method: 'POST',
				body: JSON.stringify({
					tweet_text: tweetText,
					tweet_author: tweetAuthor,
					mention_product: mentionProduct
				})
			}),
		thread: (topic: string, selectedNodeIds?: number[]) =>
			request<{ tweets: string[]; topic: string; vault_citations?: VaultCitation[] }>(
				'/api/assist/thread',
				{
					method: 'POST',
					body: JSON.stringify({
						topic,
						...(selectedNodeIds && { selected_node_ids: selectedNodeIds })
					})
				}
			),
		improve: (draft: string, context?: string, selectedNodeIds?: number[]) =>
			request<{ content: string; vault_citations?: VaultCitation[] }>(
				'/api/assist/improve',
				{
					method: 'POST',
					body: JSON.stringify({
						draft,
						context,
						...(selectedNodeIds && { selected_node_ids: selectedNodeIds })
					})
				}
			),
		topics: () =>
			request<{ topics: Array<{ topic: string; score: number }> }>('/api/assist/topics'),
		optimalTimes: () =>
			request<{
				times: Array<{ hour: number; avg_engagement: number; post_count: number }>;
			}>('/api/assist/optimal-times'),
		highlights: (selectedNodeIds: number[]) =>
			request<{ highlights: string[]; vault_citations?: VaultCitation[] }>(
				'/api/assist/highlights',
				{
					method: 'POST',
					body: JSON.stringify({ selected_node_ids: selectedNodeIds })
				}
			),
		hooks: (
			topic: string,
			opts?: { selectedNodeIds?: number[]; sessionId?: string }
		) =>
			request<AssistHooksResponse>('/api/assist/hooks', {
				method: 'POST',
				body: JSON.stringify({
					topic,
					...(opts?.selectedNodeIds && { selected_node_ids: opts.selectedNodeIds }),
					...(opts?.sessionId && { session_id: opts.sessionId })
				})
			}),
		mode: () => request<{ mode: string; approval_mode: boolean }>('/api/assist/mode')
	},

	drafts: {
		list: () => request<ScheduledContentItem[]>('/api/content/drafts'),
		create: (
			contentType: string,
			content: string,
			source: string = 'manual',
			blocks?: ThreadBlock[],
			provenance?: ProvenanceRef[]
		) =>
			request<{ id: number; status: string }>('/api/content/drafts', {
				method: 'POST',
				body: JSON.stringify({
					content_type: contentType,
					content,
					source,
					...(blocks && { blocks }),
					...(provenance && { provenance })
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

	draftStudio: {
		list: (params?: {
			status?: string;
			tag?: number;
			search?: string;
			archived?: boolean;
		}) => {
			const query = new URLSearchParams();
			if (params?.status) query.set('status', params.status);
			if (params?.tag !== undefined) query.set('tag', params.tag.toString());
			if (params?.search) query.set('search', params.search);
			if (params?.archived !== undefined) query.set('archived', params.archived.toString());
			const qs = query.toString();
			return request<DraftSummary[]>(`/api/drafts${qs ? `?${qs}` : ''}`);
		},
		get: (id: number) => request<ScheduledContentItem>(`/api/drafts/${id}`),
		create: (data?: {
			content_type?: string;
			content?: string;
			title?: string;
		}) =>
			request<{ id: number; updated_at: string }>('/api/drafts', {
				method: 'POST',
				body: JSON.stringify(data ?? {})
			}),
		autosave: (
			id: number,
			data: { content: string; content_type: string; updated_at: string }
		) =>
			request<AutosaveResponse>(`/api/drafts/${id}`, {
				method: 'PATCH',
				body: JSON.stringify(data)
			}),
		updateMeta: (id: number, data: { title?: string; notes?: string }) =>
			request<ScheduledContentItem>(`/api/drafts/${id}/meta`, {
				method: 'PATCH',
				body: JSON.stringify(data)
			}),
		schedule: (id: number, scheduledFor: string) =>
			request<{ id: number; status: string; scheduled_for: string }>(
				`/api/drafts/${id}/schedule`,
				{
					method: 'POST',
					body: JSON.stringify({ scheduled_for: scheduledFor })
				}
			),
		reschedule: (id: number, scheduledFor: string) =>
			request<{ id: number; status: string; scheduled_for: string }>(
				`/api/drafts/${id}/reschedule`,
				{
					method: 'PATCH',
					body: JSON.stringify({ scheduled_for: scheduledFor })
				}
			),
		unschedule: (id: number) =>
			request<{ id: number; status: string }>(`/api/drafts/${id}/unschedule`, {
				method: 'POST'
			}),
		archive: (id: number) =>
			request<{ id: number; archived_at: string }>(`/api/drafts/${id}/archive`, {
				method: 'POST'
			}),
		restore: (id: number) =>
			request<{ id: number }>(`/api/drafts/${id}/restore`, {
				method: 'POST'
			}),
		delete: (id: number) =>
			request<{ id: number }>(`/api/drafts/${id}`, {
				method: 'DELETE'
			}),
		duplicate: (id: number) =>
			request<{ id: number }>(`/api/drafts/${id}/duplicate`, {
				method: 'POST'
			}),
		revisions: (id: number) => request<ContentRevision[]>(`/api/drafts/${id}/revisions`),
		createRevision: (id: number, triggerKind?: string) =>
			request<{ id: number }>(`/api/drafts/${id}/revisions`, {
				method: 'POST',
				body: JSON.stringify({ trigger_kind: triggerKind ?? 'manual' })
			}),
		activity: (id: number) => request<ContentActivity[]>(`/api/drafts/${id}/activity`),
		restoreRevision: (id: number, revisionId: number) =>
			request<ScheduledContentItem>(`/api/drafts/${id}/revisions/${revisionId}/restore`, {
				method: 'POST'
			}),
		tags: (id: number) => request<ContentTag[]>(`/api/drafts/${id}/tags`),
		assignTag: (id: number, tagId: number) =>
			request<{ status: string }>(`/api/drafts/${id}/tags/${tagId}`, { method: 'POST' }),
		unassignTag: (id: number, tagId: number) =>
			request<{ status: string }>(`/api/drafts/${id}/tags/${tagId}`, { method: 'DELETE' }),
		provenance: (id: number) => request<ProvenanceLink[]>(`/api/drafts/${id}/provenance`)
	},

	tags: {
		list: () => request<ContentTag[]>('/api/tags'),
		create: (name: string, color?: string) =>
			request<{ id: number }>('/api/tags', {
				method: 'POST',
				body: JSON.stringify({ name, color: color ?? null })
			})
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
			request<McpTelemetryEntry[]>(`/api/mcp/telemetry/recent?limit=${limit}`),
		// Discovery: list all available MCP tools with their parameter hints.
		tools: () => request<McpAvailableTool[]>('/api/mcp/tools')
	},

	connectors: {
		googleDrive: {
			link: (force?: boolean) =>
				request<LinkResponse>(
					`/api/connectors/google-drive/link${force ? '?force=true' : ''}`,
					{ method: 'POST' }
				),
			status: () =>
				request<ConnectorStatusResponse>('/api/connectors/google-drive/status'),
			disconnect: (id: number) =>
				request<DisconnectResponse>(`/api/connectors/google-drive/${id}`, {
					method: 'DELETE'
				})
		}
	},

	discovery: {
		feed: (
			params: {
				minScore?: number;
				maxScore?: number;
				keyword?: string;
				limit?: number;
			} = {}
		) => {
			const query = new URLSearchParams();
			query.set('min_score', (params.minScore ?? 50).toString());
			query.set('limit', (params.limit ?? 20).toString());
			if (params.maxScore !== undefined)
				query.set('max_score', params.maxScore.toString());
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
	},

	sources: {
		reindex: (id: number) =>
			request<{ status: string; source_id: number }>(
				`/api/sources/${id}/reindex`,
				{ method: 'POST' }
			),
	},

	onboarding: {
		startAuth: (clientId?: string) =>
			request<{ authorization_url: string; state: string }>(
				'/api/onboarding/x-auth/start',
				{
					method: 'POST',
					body: JSON.stringify(clientId ? { client_id: clientId } : {})
				}
			),
		completeAuth: (code: string, state: string) =>
			request<{
				status: string;
				user: {
					id: string;
					username: string;
					name: string;
					profile_image_url: string | null;
					description: string | null;
					location: string | null;
					url: string | null;
				};
			}>('/api/onboarding/x-auth/callback', {
				method: 'POST',
				body: JSON.stringify({ code, state })
			}),
		authStatus: () =>
			request<{
				connected: boolean;
				user?: {
					id: string;
					username: string;
					name: string;
					profile_image_url: string | null;
					description: string | null;
					location: string | null;
					url: string | null;
				};
			}>('/api/onboarding/x-auth/status'),
		analyzeProfile: (llm?: {
			provider: string;
			api_key?: string;
			model: string;
			base_url?: string;
		}) =>
			request<AnalyzeProfileResponse>('/api/onboarding/analyze-profile', {
				method: 'POST',
				body: JSON.stringify({ llm: llm ?? null })
			})
	},

	vault: {
		sources: () => request<VaultSourcesResponse>('/api/vault/sources'),
		searchNotes: (params: { q?: string; source_id?: number; limit?: number } = {}) => {
			const query = new URLSearchParams();
			if (params.q) query.set('q', params.q);
			if (params.source_id) query.set('source_id', params.source_id.toString());
			if (params.limit) query.set('limit', params.limit.toString());
			const qs = query.toString();
			return request<{ notes: VaultNoteItem[] }>(`/api/vault/notes${qs ? `?${qs}` : ''}`);
		},
		noteDetail: (id: number) => request<VaultNoteDetail>(`/api/vault/notes/${id}`),
		searchFragments: (params: { q: string; limit?: number }) => {
			const query = new URLSearchParams({ q: params.q });
			if (params.limit) query.set('limit', params.limit.toString());
			return request<{ fragments: VaultCitation[] }>(
				`/api/vault/search?${query.toString()}`
			);
		},
		resolveRefs: (nodeIds: number[]) =>
			request<{ citations: VaultCitation[] }>('/api/vault/resolve-refs', {
				method: 'POST',
				body: JSON.stringify({ node_ids: nodeIds })
			}),
		getSelection: (sessionId: string) =>
			request<VaultSelectionResponse>(`/api/vault/selection/${encodeURIComponent(sessionId)}`)
	}
};
