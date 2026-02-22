//! CRUD operations for per-author interaction tracking.
//!
//! Tracks how many times the agent has replied to each author per day
//! to prevent spam behavior (replying to the same person multiple times).

use super::DbPool;
use crate::error::StorageError;

/// Increment the reply count for an author on today's date.
///
/// Uses `INSERT OR REPLACE` with a composite key of (author_id, date)
/// to atomically create or update the counter.
pub async fn increment_author_interaction(
    pool: &DbPool,
    author_id: &str,
    author_username: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO author_interactions (author_id, author_username, interaction_date, reply_count) \
         VALUES (?, ?, date('now'), 1) \
         ON CONFLICT(author_id, interaction_date) \
         DO UPDATE SET reply_count = reply_count + 1, author_username = excluded.author_username",
    )
    .bind(author_id)
    .bind(author_username)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(())
}

/// Get the number of replies sent to a specific author today.
pub async fn get_author_reply_count_today(
    pool: &DbPool,
    author_id: &str,
) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COALESCE( \
            (SELECT reply_count FROM author_interactions \
             WHERE author_id = ? AND interaction_date = date('now')), \
         0)",
    )
    .bind(author_id)
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_test_db;

    #[tokio::test]
    async fn increment_and_get_author_count() {
        let pool = init_test_db().await.expect("init db");

        let count = get_author_reply_count_today(&pool, "author_1")
            .await
            .expect("get");
        assert_eq!(count, 0);

        increment_author_interaction(&pool, "author_1", "alice")
            .await
            .expect("inc");
        let count = get_author_reply_count_today(&pool, "author_1")
            .await
            .expect("get");
        assert_eq!(count, 1);

        increment_author_interaction(&pool, "author_1", "alice")
            .await
            .expect("inc");
        let count = get_author_reply_count_today(&pool, "author_1")
            .await
            .expect("get");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn different_authors_tracked_separately() {
        let pool = init_test_db().await.expect("init db");

        increment_author_interaction(&pool, "author_1", "alice")
            .await
            .expect("inc");
        increment_author_interaction(&pool, "author_2", "bob")
            .await
            .expect("inc");

        let count1 = get_author_reply_count_today(&pool, "author_1")
            .await
            .expect("get");
        let count2 = get_author_reply_count_today(&pool, "author_2")
            .await
            .expect("get");
        assert_eq!(count1, 1);
        assert_eq!(count2, 1);
    }
}
