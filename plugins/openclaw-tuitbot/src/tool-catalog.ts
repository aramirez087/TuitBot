/**
 * Static metadata catalog for all Tuitbot MCP tools.
 *
 * Maps each tool name to its category, risk level, and whether it
 * requires a policy check before execution.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type ToolCategory = "read" | "mutation" | "composite" | "ops";
export type RiskLevel = "low" | "medium" | "high";

export interface ToolMeta {
  category: ToolCategory;
  riskLevel: RiskLevel;
  requiresPolicyCheck: boolean;
}

// ---------------------------------------------------------------------------
// Risk ordering
// ---------------------------------------------------------------------------

export const RISK_ORDER: RiskLevel[] = ["low", "medium", "high"];

/** Returns true if `level` is at most `max` in the risk ordering. */
export function riskAtMost(level: RiskLevel, max: RiskLevel): boolean {
  return RISK_ORDER.indexOf(level) <= RISK_ORDER.indexOf(max);
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

const catalog: Record<string, ToolMeta> = {
  // -- read / low / no-policy (27 tools) -----------------------------------
  get_stats:                   { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_follower_trend:          { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  suggest_topics:              { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_action_log:              { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_action_counts:           { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_rate_limits:             { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_recent_replies:          { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_reply_count_today:       { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  list_target_accounts:        { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  list_unreplied_tweets:       { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_discovery_feed:          { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  score_tweet:                 { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  list_pending_approvals:      { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_pending_count:           { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_config:                  { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  validate_config:             { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_tweet_by_id:             { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  x_get_user_by_username:      { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  x_search_tweets:             { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  x_get_user_mentions:         { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  x_get_user_tweets:           { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  get_author_context:          { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  recommend_engagement_action: { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  topic_performance_snapshot:  { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  generate_reply:              { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  generate_tweet:              { category: "read", riskLevel: "low", requiresPolicyCheck: false },
  generate_thread:             { category: "read", riskLevel: "low", requiresPolicyCheck: false },

  // -- ops / low / no-policy (4 tools) -------------------------------------
  health_check:                { category: "ops", riskLevel: "low", requiresPolicyCheck: false },
  get_mode:                    { category: "ops", riskLevel: "low", requiresPolicyCheck: false },
  get_capabilities:            { category: "ops", riskLevel: "low", requiresPolicyCheck: false },
  get_policy_status:           { category: "ops", riskLevel: "low", requiresPolicyCheck: false },

  // -- composite / low / no-policy (3 tools) --------------------------------
  find_reply_opportunities:    { category: "composite", riskLevel: "low", requiresPolicyCheck: false },
  draft_replies_for_candidates:{ category: "composite", riskLevel: "low", requiresPolicyCheck: false },
  generate_thread_plan:        { category: "composite", riskLevel: "low", requiresPolicyCheck: false },

  // -- composite / high / policy-gated (1 tool) -----------------------------
  propose_and_queue_replies:   { category: "composite", riskLevel: "high", requiresPolicyCheck: true },

  // -- mutation / high / policy-gated (4 tools) -----------------------------
  x_post_tweet:                { category: "mutation", riskLevel: "high", requiresPolicyCheck: true },
  x_reply_to_tweet:            { category: "mutation", riskLevel: "high", requiresPolicyCheck: true },
  x_quote_tweet:               { category: "mutation", riskLevel: "high", requiresPolicyCheck: true },
  approve_all:                 { category: "mutation", riskLevel: "high", requiresPolicyCheck: true },

  // -- mutation / medium / policy-gated (5 tools) ---------------------------
  x_like_tweet:                { category: "mutation", riskLevel: "medium", requiresPolicyCheck: true },
  x_follow_user:               { category: "mutation", riskLevel: "medium", requiresPolicyCheck: true },
  x_unfollow_user:             { category: "mutation", riskLevel: "medium", requiresPolicyCheck: true },
  compose_tweet:               { category: "mutation", riskLevel: "medium", requiresPolicyCheck: true },
  approve_item:                { category: "mutation", riskLevel: "medium", requiresPolicyCheck: true },

  // -- mutation / low / policy-gated (1 tool) -------------------------------
  reject_item:                 { category: "mutation", riskLevel: "low", requiresPolicyCheck: true },
};

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

/** Look up metadata for a tool by name. Returns `undefined` for unknown tools. */
export function getToolMeta(name: string): ToolMeta | undefined {
  return catalog[name];
}
