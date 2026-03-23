//! CRUD operations for Watchtower ingestion tables.
//!
//! Manages source contexts, content nodes, content chunks, draft seeds,
//! and remote sync connections for the Cold-Start Watchtower RAG pipeline.

pub mod chunks;
pub mod connections;
pub mod edges;
pub mod embeddings;
mod nodes;
mod seeds;
mod sources;
pub mod tags;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_chunks;
#[cfg(test)]
mod tests_embeddings;
#[cfg(test)]
mod tests_graph;
#[cfg(test)]
mod tests_storage;

pub use chunks::*;
pub use connections::*;
pub use edges::*;
pub use embeddings::*;
pub use nodes::*;
pub use seeds::*;
pub use sources::*;
pub use tags::*;

// ============================================================================
// Row types (tuple aliases for sqlx::query_as)
// ============================================================================

/// Row type for source_contexts queries.
type SourceContextRow = (
    i64,
    String,
    String,
    String,
    Option<String>,
    String,
    Option<String>,
    String,
    String,
);

/// Row type for content_nodes queries.
type ContentNodeRow = (
    i64,
    String,
    i64,
    String,
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
);

/// Row type for draft_seeds queries (includes chunk_id).
type DraftSeedRow = (
    i64,
    String,
    i64,
    String,
    Option<String>,
    f64,
    String,
    String,
    Option<String>,
    Option<i64>,
);

/// Row type for content_chunks queries.
type ContentChunkRow = (
    i64,    // id
    String, // account_id
    i64,    // node_id
    String, // heading_path
    String, // chunk_text
    String, // chunk_hash
    i64,    // chunk_index
    f64,    // retrieval_boost
    String, // status
    String, // created_at
    String, // updated_at
);

// ============================================================================
// Row structs
// ============================================================================

/// A registered content source.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceContext {
    pub id: i64,
    pub account_id: String,
    pub source_type: String,
    pub config_json: String,
    pub sync_cursor: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl SourceContext {
    fn from_row(r: SourceContextRow) -> Self {
        Self {
            id: r.0,
            account_id: r.1,
            source_type: r.2,
            config_json: r.3,
            sync_cursor: r.4,
            status: r.5,
            error_message: r.6,
            created_at: r.7,
            updated_at: r.8,
        }
    }
}

/// An ingested content node from a source.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentNode {
    pub id: i64,
    pub account_id: String,
    pub source_id: i64,
    pub relative_path: String,
    pub content_hash: String,
    pub title: Option<String>,
    pub body_text: String,
    pub front_matter_json: Option<String>,
    pub tags: Option<String>,
    pub status: String,
    pub ingested_at: String,
    pub updated_at: String,
}

impl ContentNode {
    fn from_row(r: ContentNodeRow) -> Self {
        Self {
            id: r.0,
            account_id: r.1,
            source_id: r.2,
            relative_path: r.3,
            content_hash: r.4,
            title: r.5,
            body_text: r.6,
            front_matter_json: r.7,
            tags: r.8,
            status: r.9,
            ingested_at: r.10,
            updated_at: r.11,
        }
    }
}

/// A pre-computed draft seed derived from a content node.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DraftSeed {
    pub id: i64,
    pub account_id: String,
    pub node_id: i64,
    pub seed_text: String,
    pub archetype_suggestion: Option<String>,
    pub engagement_weight: f64,
    pub status: String,
    pub created_at: String,
    pub used_at: Option<String>,
    pub chunk_id: Option<i64>,
}

impl DraftSeed {
    fn from_row(r: DraftSeedRow) -> Self {
        Self {
            id: r.0,
            account_id: r.1,
            node_id: r.2,
            seed_text: r.3,
            archetype_suggestion: r.4,
            engagement_weight: r.5,
            status: r.6,
            created_at: r.7,
            used_at: r.8,
            chunk_id: r.9,
        }
    }
}

/// A heading-delimited fragment of a content node.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContentChunk {
    pub id: i64,
    pub account_id: String,
    pub node_id: i64,
    pub heading_path: String,
    pub chunk_text: String,
    pub chunk_hash: String,
    pub chunk_index: i64,
    pub retrieval_boost: f64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl ContentChunk {
    fn from_row(r: ContentChunkRow) -> Self {
        Self {
            id: r.0,
            account_id: r.1,
            node_id: r.2,
            heading_path: r.3,
            chunk_text: r.4,
            chunk_hash: r.5,
            chunk_index: r.6,
            retrieval_boost: r.7,
            status: r.8,
            created_at: r.9,
            updated_at: r.10,
        }
    }
}

/// Result of an upsert operation on a content node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertResult {
    /// A new node was inserted.
    Inserted,
    /// An existing node was updated (content hash changed).
    Updated,
    /// The node was skipped (content hash unchanged).
    Skipped,
}

/// Row type for seeds with their parent node context.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SeedWithContext {
    /// The seed hook text.
    pub seed_text: String,
    /// Title from the parent content node.
    pub source_title: Option<String>,
    /// Suggested archetype for the seed.
    pub archetype_suggestion: Option<String>,
    /// Engagement weight for retrieval ranking.
    pub engagement_weight: f64,
}

/// A content chunk joined with its parent node metadata for retrieval display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChunkWithNodeContext {
    /// The underlying chunk data.
    pub chunk: ContentChunk,
    /// Relative path of the parent content node.
    pub relative_path: String,
    /// Title of the parent content node (may be None for untitled notes).
    pub source_title: Option<String>,
}
