//! CRUD operations for target account monitoring.
//!
//! Manages the `target_accounts` and `target_tweets` tables for
//! relationship-based engagement with specific accounts.

pub mod mutations;
pub mod queries;

#[cfg(test)]
mod tests;

pub use mutations::{
    count_target_replies_today, count_target_replies_today_for, deactivate_target_account,
    deactivate_target_account_for, mark_target_tweet_replied, mark_target_tweet_replied_for,
    record_target_reply, record_target_reply_for, store_target_tweet, store_target_tweet_for,
    upsert_target_account, upsert_target_account_for,
};
pub use queries::{
    compute_frequency, get_active_target_accounts, get_active_target_accounts_for,
    get_enriched_target_accounts, get_enriched_target_accounts_for, get_target_account,
    get_target_account_by_username, get_target_account_by_username_for, get_target_account_for,
    get_target_stats, get_target_stats_for, get_target_timeline, get_target_timeline_for,
    target_tweet_exists, target_tweet_exists_for, EnrichedTargetAccount, TargetAccount,
    TargetStats, TargetTimelineItem,
};
