use super::super::accounts::DEFAULT_ACCOUNT_ID;
use super::super::DbPool;
use crate::error::StorageError;

/// A daily follower snapshot.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FollowerSnapshot {
    pub snapshot_date: String,
    pub follower_count: i64,
    pub following_count: i64,
    pub tweet_count: i64,
}

/// Upsert a follower snapshot for today for a specific account.
pub async fn upsert_follower_snapshot_for(
    pool: &DbPool,
    account_id: &str,
    follower_count: i64,
    following_count: i64,
    tweet_count: i64,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO follower_snapshots (account_id, snapshot_date, follower_count, following_count, tweet_count) \
         VALUES (?, date('now'), ?, ?, ?) \
         ON CONFLICT(snapshot_date) DO UPDATE SET \
         account_id = excluded.account_id, \
         follower_count = excluded.follower_count, \
         following_count = excluded.following_count, \
         tweet_count = excluded.tweet_count",
    )
    .bind(account_id)
    .bind(follower_count)
    .bind(following_count)
    .bind(tweet_count)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Upsert a follower snapshot for today.
pub async fn upsert_follower_snapshot(
    pool: &DbPool,
    follower_count: i64,
    following_count: i64,
    tweet_count: i64,
) -> Result<(), StorageError> {
    upsert_follower_snapshot_for(
        pool,
        DEFAULT_ACCOUNT_ID,
        follower_count,
        following_count,
        tweet_count,
    )
    .await
}

/// Get recent follower snapshots for a specific account, ordered by date descending.
pub async fn get_follower_snapshots_for(
    pool: &DbPool,
    account_id: &str,
    limit: u32,
) -> Result<Vec<FollowerSnapshot>, StorageError> {
    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT snapshot_date, follower_count, following_count, tweet_count \
         FROM follower_snapshots \
         WHERE account_id = ? \
         ORDER BY snapshot_date DESC \
         LIMIT ?",
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows
        .into_iter()
        .map(|r| FollowerSnapshot {
            snapshot_date: r.0,
            follower_count: r.1,
            following_count: r.2,
            tweet_count: r.3,
        })
        .collect())
}

/// Get recent follower snapshots, ordered by date descending.
pub async fn get_follower_snapshots(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<FollowerSnapshot>, StorageError> {
    get_follower_snapshots_for(pool, DEFAULT_ACCOUNT_ID, limit).await
}
