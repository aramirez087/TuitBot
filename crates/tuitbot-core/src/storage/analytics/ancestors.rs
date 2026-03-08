use super::super::DbPool;
use crate::error::StorageError;

/// Update the archetype_vibe classification for a tweet performance record.
pub async fn update_tweet_archetype(
    pool: &DbPool,
    tweet_id: &str,
    archetype_vibe: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE tweet_performance SET archetype_vibe = ? WHERE tweet_id = ?")
        .bind(archetype_vibe)
        .bind(tweet_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Update the archetype_vibe classification for a reply performance record.
pub async fn update_reply_archetype(
    pool: &DbPool,
    reply_id: &str,
    archetype_vibe: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE reply_performance SET archetype_vibe = ? WHERE reply_id = ?")
        .bind(archetype_vibe)
        .bind(reply_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Update the engagement_score for a tweet performance record.
pub async fn update_tweet_engagement_score(
    pool: &DbPool,
    tweet_id: &str,
    score: f64,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE tweet_performance SET engagement_score = ? WHERE tweet_id = ?")
        .bind(score)
        .bind(tweet_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Update the engagement_score for a reply performance record.
pub async fn update_reply_engagement_score(
    pool: &DbPool,
    reply_id: &str,
    score: f64,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE reply_performance SET engagement_score = ? WHERE reply_id = ?")
        .bind(score)
        .bind(reply_id)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;
    Ok(())
}

/// Get the maximum performance_score across all tweets and replies.
///
/// Returns 0.0 if no performance data exists. Used to normalize engagement scores.
pub async fn get_max_performance_score(pool: &DbPool) -> Result<f64, StorageError> {
    let row: (f64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(max_score), 0.0) FROM (\
             SELECT MAX(performance_score) as max_score FROM tweet_performance \
             UNION ALL \
             SELECT MAX(performance_score) as max_score FROM reply_performance\
         )",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Query { source: e })?;

    Ok(row.0)
}

/// Row type returned by the ancestor retrieval query.
#[derive(Debug, Clone)]
pub struct AncestorRow {
    /// "tweet" or "reply"
    pub content_type: String,
    /// The tweet or reply ID.
    pub id: String,
    /// Truncated content preview (up to 120 chars).
    pub content_preview: String,
    /// Archetype classification (may be None if not yet classified).
    pub archetype_vibe: Option<String>,
    /// Normalized engagement score (0.0-1.0).
    pub engagement_score: Option<f64>,
    /// Raw performance score.
    pub performance_score: f64,
    /// When the content was posted (ISO-8601).
    pub posted_at: String,
}

/// Row type returned by the ancestor retrieval UNION query.
type AncestorQueryRow = (
    String,
    String,
    String,
    Option<String>,
    Option<f64>,
    f64,
    String,
);

/// Convert an ancestor query row tuple into an `AncestorRow` struct.
fn ancestor_row_from_tuple(r: AncestorQueryRow) -> AncestorRow {
    AncestorRow {
        content_type: r.0,
        id: r.1,
        content_preview: r.2,
        archetype_vibe: r.3,
        engagement_score: r.4,
        performance_score: r.5,
        posted_at: r.6,
    }
}

/// Query scored ancestors with engagement_score populated, scoped to an account.
///
/// Returns ancestors where `engagement_score >= min_score`, ordered by
/// engagement_score DESC. For topic matching, uses the `topic` column on
/// original_tweets and LIKE-based content matching on replies.
pub async fn get_scored_ancestors(
    pool: &DbPool,
    account_id: &str,
    topic_keywords: &[String],
    min_score: f64,
    limit: u32,
) -> Result<Vec<AncestorRow>, StorageError> {
    if topic_keywords.is_empty() {
        // No keywords: return top ancestors regardless of topic
        let rows: Vec<AncestorQueryRow> = sqlx::query_as(
            "SELECT 'tweet' as content_type, tp.tweet_id, \
                        SUBSTR(ot.content, 1, 120), \
                        tp.archetype_vibe, tp.engagement_score, tp.performance_score, \
                        ot.created_at \
                 FROM tweet_performance tp \
                 JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
                 WHERE ot.account_id = ? \
                   AND tp.engagement_score IS NOT NULL \
                   AND tp.engagement_score >= ? \
                 UNION ALL \
                 SELECT 'reply', rp.reply_id, SUBSTR(rs.reply_content, 1, 120), \
                        rp.archetype_vibe, rp.engagement_score, rp.performance_score, \
                        rs.created_at \
                 FROM reply_performance rp \
                 JOIN replies_sent rs ON rs.reply_tweet_id = rp.reply_id \
                 WHERE rs.account_id = ? \
                   AND rp.engagement_score IS NOT NULL \
                   AND rp.engagement_score >= ? \
                 ORDER BY engagement_score DESC \
                 LIMIT ?",
        )
        .bind(account_id)
        .bind(min_score)
        .bind(account_id)
        .bind(min_score)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

        return Ok(rows.into_iter().map(ancestor_row_from_tuple).collect());
    }

    // Build parameterized IN clause for tweet topics and LIKE clauses for replies.
    // SQLx uses sequential `?` placeholders for SQLite.
    let topic_placeholders: String = (0..topic_keywords.len())
        .map(|_| "?".to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let like_conditions: Vec<String> = (0..topic_keywords.len())
        .map(|_| "rs.reply_content LIKE '%' || ? || '%'".to_string())
        .collect();
    let like_clause = like_conditions.join(" OR ");

    let query_str = format!(
        "SELECT 'tweet' as content_type, tp.tweet_id, \
                SUBSTR(ot.content, 1, 120), \
                tp.archetype_vibe, tp.engagement_score, tp.performance_score, \
                ot.created_at \
         FROM tweet_performance tp \
         JOIN original_tweets ot ON ot.tweet_id = tp.tweet_id \
         WHERE ot.account_id = ? \
           AND tp.engagement_score IS NOT NULL \
           AND tp.engagement_score >= ? \
           AND (ot.topic IN ({topic_placeholders})) \
         UNION ALL \
         SELECT 'reply', rp.reply_id, SUBSTR(rs.reply_content, 1, 120), \
                rp.archetype_vibe, rp.engagement_score, rp.performance_score, \
                rs.created_at \
         FROM reply_performance rp \
         JOIN replies_sent rs ON rs.reply_tweet_id = rp.reply_id \
         WHERE rs.account_id = ? \
           AND rp.engagement_score IS NOT NULL \
           AND rp.engagement_score >= ? \
           AND ({like_clause}) \
         ORDER BY engagement_score DESC \
         LIMIT ?"
    );

    let mut query = sqlx::query_as::<_, AncestorQueryRow>(&query_str);

    // Bind: account_id for tweets, min_score, then topic keywords for IN clause
    query = query.bind(account_id);
    query = query.bind(min_score);
    for kw in topic_keywords {
        query = query.bind(kw);
    }
    // Bind: account_id for replies, min_score, keywords for LIKE clauses, then limit
    query = query.bind(account_id);
    query = query.bind(min_score);
    for kw in topic_keywords {
        query = query.bind(kw);
    }
    query = query.bind(limit);

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Query { source: e })?;

    Ok(rows.into_iter().map(ancestor_row_from_tuple).collect())
}
