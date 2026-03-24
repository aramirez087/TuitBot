//! Score computation and analytics summary types.

/// Summary of an analytics iteration.
#[derive(Debug, Default)]
pub struct AnalyticsSummary {
    pub follower_count: i64,
    pub replies_measured: usize,
    pub tweets_measured: usize,
    pub forge_synced: bool,
}

/// Compute the performance score for content engagement.
///
/// Formula: `(likes * 3 + replies * 5 + retweets * 4) / max(impressions, 1) * 1000`
pub fn compute_performance_score(likes: i64, replies: i64, retweets: i64, impressions: i64) -> f64 {
    let numerator = (likes * 3 + replies * 5 + retweets * 4) as f64;
    let denominator = impressions.max(1) as f64;
    numerator / denominator * 1000.0
}
