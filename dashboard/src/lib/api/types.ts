// --- Profile inference types ---

export type Confidence = 'high' | 'medium' | 'low';
export type InferenceProvenance =
	| 'bio'
	| 'tweets'
	| 'bio_and_tweets'
	| 'profile_url'
	| 'display_name'
	| 'default';

export interface InferredField<T> {
	value: T;
	confidence: Confidence;
	provenance: InferenceProvenance;
}

export interface InferredProfile {
	account_type: InferredField<'individual' | 'business'>;
	product_name: InferredField<string>;
	product_description: InferredField<string>;
	product_url: InferredField<string | null>;
	target_audience: InferredField<string>;
	product_keywords: InferredField<string[]>;
	industry_topics: InferredField<string[]>;
	brand_voice: InferredField<string | null>;
}

export type AnalyzeProfileStatus = 'ok' | 'partial' | 'x_api_error' | 'llm_error';

export interface AnalyzeProfileResponse {
	status: AnalyzeProfileStatus;
	profile?: InferredProfile;
	warnings?: string[];
	error?: string;
}

// --- Capability tier types ---

export type CapabilityTier =
	| 'unconfigured'
	| 'profile_ready'
	| 'exploration_ready'
	| 'generation_ready'
	| 'posting_ready';

// --- Deployment types ---

export type DeploymentModeValue = 'desktop' | 'self_host' | 'cloud';

export interface DeploymentCapabilities {
	local_folder: boolean;
	manual_local_path: boolean;
	google_drive: boolean;
	inline_ingest: boolean;
	file_picker_native: boolean;
	preferred_source_default: string;
}

export interface RuntimeStatus {
	running: boolean;
	task_count: number;
	deployment_mode: DeploymentModeValue;
	capabilities: DeploymentCapabilities;
	provider_backend: string;
	can_post: boolean;
	capability_tier: CapabilityTier;
}

// --- Health & Analytics types ---

/** Runtime health status returned by `GET /api/health`. */
export interface HealthResponse {
	status: string;
	version: string;
	/** Scraper-specific health details (optional, populated by Sprint 4 scraper reliability). */
	scraper?: {
		status: 'healthy' | 'degraded' | 'down';
		last_success_at?: string;
		consecutive_failures?: number;
		circuit_state?: 'closed' | 'open' | 'half-open';
	};
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

/** Aggregated analytics summary returned by `GET /api/analytics/summary`. */
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

// --- Activity types ---

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

// --- Approval types ---

/** An item in the approval queue. `action_type` distinguishes content type (e.g. `failed_post_recovery`). */
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
	scheduled_for?: string;
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
	failed: number;
	scheduled: number;
}

// --- Content types ---

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
	provenance?: ProvenanceRef[];
	hook_style?: string;
}

// Thread content parsing utilities moved to $lib/utils/parseThreadContent.ts
// Re-exported here for backward compatibility.
export { parseThreadContent, isBlocksPayload } from '$lib/utils/parseThreadContent';

export interface ScheduledContentItem {
	id: number;
	content_type: string;
	content: string;
	scheduled_for: string | null;
	status: string;
	posted_tweet_id: string | null;
	created_at: string;
	updated_at: string;
	title?: string | null;
	notes?: string | null;
	archived_at?: string | null;
}

// --- Draft Studio types ---

/** Lightweight draft summary returned from draft list endpoints. Full content is in `ScheduledContentItem`. */
export interface DraftSummary {
	id: number;
	title: string | null;
	content_type: string;
	content_preview: string;
	status: string;
	scheduled_for: string | null;
	archived_at: string | null;
	updated_at: string;
	created_at: string;
	source: string;
}

export interface ContentTag {
	id: number;
	account_id: string;
	name: string;
	color: string | null;
}

export interface AutosaveResponse {
	id: number;
	updated_at: string;
}

export interface StaleWriteError {
	error: 'stale_write';
	server_updated_at: string;
}

export interface ContentRevision {
	id: number;
	content_id: number;
	account_id: string;
	content: string;
	content_type: string;
	trigger_kind: string;
	created_at: string;
}

export interface ContentActivity {
	id: number;
	content_id: number;
	account_id: string;
	action: string;
	detail: string | null;
	created_at: string;
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

// --- Connector types ---

export interface Connection {
	id: number;
	connector_type: string;
	account_email: string | null;
	display_name: string | null;
	status: string;
	metadata_json: string;
	created_at: string;
	updated_at: string;
}

export interface LinkResponse {
	authorization_url: string;
	state: string;
}

export interface ConnectorStatusResponse {
	connections: Connection[];
}

export interface DisconnectResponse {
	disconnected: boolean;
	id: number;
}

// --- Settings types ---

export interface TuitbotConfig {
	x_api: {
		client_id: string;
		client_secret: string | null;
		provider_backend: string;
		scraper_allow_mutations: boolean;
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
	content_sources: {
		sources: Array<{
			source_type: string;
			path: string | null;
			folder_id: string | null;
			service_account_key: string | null;
			connection_id: number | null;
			watch: boolean;
			file_patterns: string[];
			loop_back_enabled: boolean;
			poll_interval_seconds: number | null;
		}>;
	};
	deployment_mode: DeploymentModeValue;
	connectors?: {
		google_drive?: {
			client_id?: string | null;
			client_secret?: string | null;
			redirect_uri?: string | null;
		};
	};
}

/** Response from GET /api/settings when a non-default account is active. */
export interface EffectiveSettingsResponse {
	config: TuitbotConfig;
	/** Top-level keys that are overridden by this account (empty for default). */
	_overrides: string[];
}

export interface ConfigStatus {
	configured: boolean;
	claimed: boolean;
	deployment_mode: DeploymentModeValue;
	capabilities: DeploymentCapabilities;
	capability_tier: CapabilityTier;
	has_x_client_id: boolean;
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

// --- Account types ---

export interface Account {
	id: string;
	label: string;
	x_username: string | null;
	x_user_id: string | null;
	x_display_name: string | null;
	x_avatar_url: string | null;
	config_overrides: string | null;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface AccountAuthStatus {
	oauth_linked: boolean;
	oauth_expired: boolean;
	oauth_expires_at: string | null;
	scraper_linked: boolean;
	has_credentials: boolean;
}

// --- Vault types ---

export interface VaultCitation {
	chunk_id: number;
	node_id: number;
	heading_path: string;
	source_path: string;
	source_title: string | null;
	snippet: string;
	retrieval_boost: number;
}

export interface ProvenanceRef {
	node_id?: number;
	chunk_id?: number;
	seed_id?: number;
	source_path?: string;
	heading_path?: string;
	snippet?: string;
}

export interface ProvenanceLink {
	id: number;
	account_id: string;
	entity_type: string;
	entity_id: number;
	node_id: number | null;
	chunk_id: number | null;
	seed_id: number | null;
	source_path: string | null;
	heading_path: string | null;
	snippet: string | null;
	created_at: string;
}

export interface VaultSourceStatus {
	id: number;
	source_type: string;
	status: string;
	error_message: string | null;
	node_count: number;
	updated_at: string;
	/** Local vault path for `local_fs` sources — used for Obsidian URI deep links. */
	path?: string;
}

export interface VaultNoteItem {
	node_id: number;
	source_id: number;
	title: string | null;
	relative_path: string;
	tags: string | null;
	status: string;
	chunk_count: number;
	updated_at: string;
}

export interface VaultChunkSummary {
	chunk_id: number;
	heading_path: string;
	snippet: string;
	retrieval_boost: number;
}

export interface VaultNoteDetail {
	node_id: number;
	source_id: number;
	title: string | null;
	relative_path: string;
	tags: string | null;
	status: string;
	ingested_at: string;
	updated_at: string;
	chunks: VaultChunkSummary[];
}

// --- Ghostwriter selection (from Obsidian plugin) ---

export interface VaultSelectionResponse {
	session_id: string;
	vault_name: string;
	file_path: string;
	selected_text: string | null;
	heading_context: string | null;
	note_title: string | null;
	frontmatter_tags: string[] | null;
	resolved_node_id: number | null;
	resolved_chunk_id: number | null;
	created_at: string;
	expires_at: string;
}

// --- Hook generation types (Ghostwriter) ---

export interface HookOption {
	style: string;
	text: string;
	char_count: number;
	confidence: string;
}

export interface AssistHooksResponse {
	hooks: HookOption[];
	topic: string;
	vault_citations?: VaultCitation[];
}

// MCP tool discovery (GET /mcp/tools) — read-only, shows available tools + parameter hints.
export interface McpToolParam {
	name: string;
	type: string;
	description: string;
	required: boolean;
}

export interface McpAvailableTool {
	name: string;
	description: string;
	category: string;
	params: McpToolParam[];
}
