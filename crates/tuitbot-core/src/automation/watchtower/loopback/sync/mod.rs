//! Forge sync engine — enriches source-note frontmatter with analytics.
//!
//! The sync path is separate from the publish writeback path. It never creates
//! entries; it only updates analytics fields on entries that already exist
//! (created by `write_metadata_to_file` during the publish step).

use std::io;
use std::path::Path;

use super::{
    parse_tuitbot_metadata, serialize_frontmatter_to_file, split_front_matter, TuitbotFrontMatter,
};

#[cfg(test)]
mod tests;

/// Result of an analytics update attempt on a single entry.
#[derive(Debug, PartialEq, Eq)]
pub enum UpdateResult {
    Updated,
    EntryNotFound,
    FileNotFound,
}

/// Analytics data to write into a frontmatter entry.
#[derive(Debug, Clone)]
pub struct EntryAnalytics {
    pub impressions: i64,
    pub likes: i64,
    pub retweets: i64,
    pub replies: i64,
    pub engagement_rate: Option<f64>,
    pub performance_score: Option<f64>,
    pub synced_at: String,
}

// Re-export from storage so callers that import from sync continue to work.
pub use crate::storage::analytics::PerformancePercentiles;

/// Metrics from the tweet_performance table for Forge sync.
#[derive(Debug, Clone)]
pub struct TweetPerformanceRow {
    pub tweet_id: String,
    pub likes_received: i64,
    pub retweets_received: i64,
    pub replies_received: i64,
    pub impressions: i64,
    pub performance_score: f64,
}

/// Result summary of a Forge sync iteration.
#[derive(Debug, Default)]
pub struct ForgeSyncSummary {
    pub tweets_synced: usize,
    pub threads_synced: usize,
    pub entries_not_found: usize,
    pub files_not_found: usize,
    pub non_local_skipped: usize,
}

/// Update analytics fields on an existing frontmatter entry identified by tweet_id.
///
/// This is the Forge sync path — it never creates entries, only updates them.
/// After updating, recomputes note-level summary fields.
pub fn update_entry_analytics(
    path: &Path,
    tweet_id: &str,
    analytics: &EntryAnalytics,
    percentiles: &PerformancePercentiles,
) -> Result<UpdateResult, io::Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(UpdateResult::FileNotFound),
        Err(e) => return Err(e),
    };

    let existing = parse_tuitbot_metadata(&content);
    if !existing.iter().any(|e| e.tweet_id == tweet_id) {
        return Ok(UpdateResult::EntryNotFound);
    }

    let (yaml_str, body) = split_front_matter(&content);

    let mut fm: TuitbotFrontMatter = match yaml_str {
        Some(y) => serde_yaml::from_str(y).unwrap_or_default(),
        None => return Ok(UpdateResult::EntryNotFound),
    };

    // Find and update the matching entry (first match only).
    let mut found = false;
    for entry in &mut fm.tuitbot {
        if entry.tweet_id == tweet_id {
            entry.impressions = Some(analytics.impressions);
            entry.likes = Some(analytics.likes);
            entry.retweets = Some(analytics.retweets);
            entry.replies = Some(analytics.replies);
            entry.engagement_rate = analytics.engagement_rate;
            entry.performance_score = analytics.performance_score;
            entry.synced_at = Some(analytics.synced_at.clone());
            found = true;
            break;
        }
    }

    if !found {
        return Ok(UpdateResult::EntryNotFound);
    }

    recompute_summaries(&mut fm, percentiles);
    serialize_frontmatter_to_file(path, &fm, body)?;
    Ok(UpdateResult::Updated)
}

/// Recompute note-level summary fields from the tuitbot entries.
///
/// Sets `tuitbot_social_performance`, `tuitbot_best_post_impressions`,
/// `tuitbot_best_post_url`, and `tuitbot_last_synced_at` in the
/// `TuitbotFrontMatter.other` mapping.
pub fn recompute_summaries(fm: &mut TuitbotFrontMatter, percentiles: &PerformancePercentiles) {
    let key = |s: &str| serde_yaml::Value::String(s.to_string());
    let str_val = |s: &str| serde_yaml::Value::String(s.to_string());

    // Remove existing summary keys first (clean slate).
    let summary_keys = [
        "tuitbot_social_performance",
        "tuitbot_best_post_impressions",
        "tuitbot_best_post_url",
        "tuitbot_last_synced_at",
    ];
    for k in &summary_keys {
        fm.other.remove(key(k));
    }

    // Find the entry with the highest impressions (tie-break by latest published_at).
    let best = fm
        .tuitbot
        .iter()
        .filter(|e| e.impressions.is_some())
        .max_by(|a, b| {
            let imp_cmp = a.impressions.unwrap().cmp(&b.impressions.unwrap());
            if imp_cmp == std::cmp::Ordering::Equal {
                a.published_at.cmp(&b.published_at)
            } else {
                imp_cmp
            }
        });

    let best = match best {
        Some(b) => b,
        None => {
            fm.other
                .insert(key("tuitbot_social_performance"), str_val("none"));
            return;
        }
    };

    let best_impressions = best.impressions.unwrap();
    let best_url = best.url.clone();

    // Determine performance tier.
    let tier = if !percentiles.has_sufficient_data {
        "none"
    } else if best_impressions >= percentiles.p90_impressions {
        "high"
    } else if best_impressions >= percentiles.p50_impressions {
        "medium"
    } else {
        "low"
    };

    fm.other
        .insert(key("tuitbot_social_performance"), str_val(tier));
    fm.other.insert(
        key("tuitbot_best_post_impressions"),
        serde_yaml::Value::Number(serde_yaml::Number::from(best_impressions)),
    );
    fm.other
        .insert(key("tuitbot_best_post_url"), str_val(&best_url));

    // Find the most recent synced_at across all entries.
    let last_synced = fm
        .tuitbot
        .iter()
        .filter_map(|e| e.synced_at.as_deref())
        .max();

    if let Some(synced) = last_synced {
        fm.other
            .insert(key("tuitbot_last_synced_at"), str_val(synced));
    }
}

/// Aggregate tweet_performance metrics for a thread.
///
/// Sums counts (impressions, likes, retweets, replies) across all tweet IDs.
/// Computes engagement_rate from totals and performance_score as
/// impression-weighted average.
pub fn aggregate_thread_metrics(performances: &[TweetPerformanceRow]) -> Option<EntryAnalytics> {
    if performances.is_empty() {
        return None;
    }

    let total_impressions: i64 = performances.iter().map(|p| p.impressions).sum();
    let total_likes: i64 = performances.iter().map(|p| p.likes_received).sum();
    let total_retweets: i64 = performances.iter().map(|p| p.retweets_received).sum();
    let total_replies: i64 = performances.iter().map(|p| p.replies_received).sum();

    let engagement_rate = if total_impressions > 0 {
        let engagements = (total_likes + total_retweets + total_replies) as f64;
        Some((engagements / total_impressions as f64) * 100.0)
    } else {
        None
    };

    // Impression-weighted average performance score.
    let weighted_sum: f64 = performances
        .iter()
        .filter(|p| p.impressions > 0)
        .map(|p| p.performance_score * p.impressions as f64)
        .sum();
    let total_weight: i64 = performances
        .iter()
        .filter(|p| p.impressions > 0)
        .map(|p| p.impressions)
        .sum();

    let performance_score = if total_weight > 0 {
        Some(weighted_sum / total_weight as f64)
    } else {
        None
    };

    let synced_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    Some(EntryAnalytics {
        impressions: total_impressions,
        likes: total_likes,
        retweets: total_retweets,
        replies: total_replies,
        engagement_rate,
        performance_score,
        synced_at,
    })
}

/// Run a single Forge sync iteration.
///
/// Finds all tweets with measured performance that have provenance
/// to local_fs source notes, and writes aggregated analytics into
/// those notes' frontmatter.
pub async fn run_forge_sync(
    pool: &crate::storage::DbPool,
    account_id: &str,
    analytics_sync_enabled: bool,
    percentiles: &PerformancePercentiles,
) -> Result<ForgeSyncSummary, crate::error::StorageError> {
    if !analytics_sync_enabled {
        return Ok(ForgeSyncSummary::default());
    }

    let performances =
        crate::storage::analytics::get_all_tweet_performances_for(pool, account_id).await?;

    let mut summary = ForgeSyncSummary::default();

    for perf in &performances {
        // Look up original_tweets row for this tweet_id.
        let ot_id = match crate::storage::threads::get_original_tweet_id_by_tweet_id(
            pool,
            account_id,
            &perf.tweet_id,
        )
        .await?
        {
            Some(id) => id,
            None => continue,
        };

        // Look up provenance to find source path.
        let (source_path, source_type, base_path) =
            match crate::storage::provenance::get_primary_source_for_tweet(pool, account_id, ot_id)
                .await?
            {
                Some(info) => info,
                None => continue,
            };

        // Gate on source type — only local_fs is writable.
        if source_type != "local_fs" {
            summary.non_local_skipped += 1;
            continue;
        }

        let expanded = crate::storage::expand_tilde(&base_path);
        let full_path = std::path::PathBuf::from(expanded).join(&source_path);

        // Check if this is a thread root by looking for children.
        let child_ids = crate::storage::threads::get_thread_tweet_ids_by_root_for(
            pool,
            account_id,
            &perf.tweet_id,
        )
        .await
        .unwrap_or_default();

        let is_thread = !child_ids.is_empty();
        let analytics = if !is_thread {
            // Single tweet — use metrics directly.
            EntryAnalytics {
                impressions: perf.impressions,
                likes: perf.likes_received,
                retweets: perf.retweets_received,
                replies: perf.replies_received,
                engagement_rate: if perf.impressions > 0 {
                    let eng = (perf.likes_received + perf.retweets_received + perf.replies_received)
                        as f64;
                    Some((eng / perf.impressions as f64) * 100.0)
                } else {
                    None
                },
                performance_score: Some(perf.performance_score),
                synced_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            }
        } else {
            // Thread — gather all tweet IDs and aggregate.
            let mut all_ids = vec![perf.tweet_id.clone()];
            all_ids.extend(child_ids);

            let all_performances =
                crate::storage::analytics::get_tweet_performances_for(pool, account_id, &all_ids)
                    .await?;

            let converted: Vec<TweetPerformanceRow> = all_performances
                .into_iter()
                .map(|p| TweetPerformanceRow {
                    tweet_id: p.tweet_id,
                    likes_received: p.likes_received,
                    retweets_received: p.retweets_received,
                    replies_received: p.replies_received,
                    impressions: p.impressions,
                    performance_score: p.performance_score,
                })
                .collect();

            match aggregate_thread_metrics(&converted) {
                Some(a) => a,
                None => continue,
            }
        };

        match update_entry_analytics(&full_path, &perf.tweet_id, &analytics, percentiles) {
            Ok(UpdateResult::Updated) => {
                if !is_thread {
                    summary.tweets_synced += 1;
                } else {
                    summary.threads_synced += 1;
                }
            }
            Ok(UpdateResult::EntryNotFound) => {
                summary.entries_not_found += 1;
            }
            Ok(UpdateResult::FileNotFound) => {
                summary.files_not_found += 1;
            }
            Err(e) => {
                tracing::warn!(
                    tweet_id = %perf.tweet_id,
                    path = %full_path.display(),
                    error = %e,
                    "Forge sync: file write failed"
                );
                summary.files_not_found += 1;
            }
        }
    }

    Ok(summary)
}
