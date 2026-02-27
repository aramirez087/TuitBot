//! Thread block types and validation for structured thread composition.
//!
//! Provides the `ThreadBlock` struct for representing individual tweets
//! within a thread, along with validation and storage serialization.

use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::length::{tweet_weighted_len, MAX_TWEET_CHARS};

/// Maximum number of media attachments per block.
pub const MAX_MEDIA_PER_BLOCK: usize = 4;

/// A single tweet block within a thread.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThreadBlock {
    /// Client-generated stable UUID for this block.
    pub id: String,
    /// The tweet text content.
    pub text: String,
    /// Local media file paths attached to this block.
    #[serde(default)]
    pub media_paths: Vec<String>,
    /// Zero-based ordering index within the thread.
    pub order: u32,
}

/// Versioned wrapper for serializing thread blocks to the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThreadBlocksPayload {
    /// Schema version (currently 1).
    pub version: u8,
    /// Ordered list of thread blocks.
    pub blocks: Vec<ThreadBlock>,
}

/// Validation errors for thread block payloads.
#[derive(Debug, thiserror::Error)]
pub enum ThreadBlockError {
    #[error("thread blocks must not be empty")]
    EmptyBlocks,
    #[error("thread must contain at least 2 blocks")]
    SingleBlock,
    #[error("duplicate block ID: {id}")]
    DuplicateBlockId { id: String },
    #[error("block order must be a contiguous sequence starting at 0")]
    NonContiguousOrder {
        expected: Vec<u32>,
        actual: Vec<u32>,
    },
    #[error("block {block_id} has empty text")]
    EmptyBlockText { block_id: String },
    #[error("block {block_id}: text exceeds {max} characters (length: {length})")]
    BlockTextTooLong {
        block_id: String,
        length: usize,
        max: usize,
    },
    #[error("block {block_id}: too many media attachments ({count}, max {max})")]
    TooManyMedia {
        block_id: String,
        count: usize,
        max: usize,
    },
    #[error("block at index {index} has an empty ID")]
    InvalidBlockId { index: usize },
}

impl ThreadBlockError {
    /// Return the user-facing error message for API responses.
    pub fn api_message(&self) -> String {
        self.to_string()
    }
}

/// Validate a slice of thread blocks.
///
/// Checks:
/// 1. Non-empty blocks array
/// 2. At least 2 blocks for a thread
/// 3. All block IDs are non-empty
/// 4. All block IDs are unique
/// 5. Order fields form contiguous 0..N sequence
/// 6. Each block's text is non-empty after trim
/// 7. Each block's text is within MAX_TWEET_CHARS
/// 8. Each block has at most MAX_MEDIA_PER_BLOCK media entries
pub fn validate_thread_blocks(blocks: &[ThreadBlock]) -> Result<(), ThreadBlockError> {
    if blocks.is_empty() {
        return Err(ThreadBlockError::EmptyBlocks);
    }
    if blocks.len() < 2 {
        return Err(ThreadBlockError::SingleBlock);
    }

    // Validate block IDs are non-empty.
    for (i, block) in blocks.iter().enumerate() {
        if block.id.trim().is_empty() {
            return Err(ThreadBlockError::InvalidBlockId { index: i });
        }
    }

    // Check for duplicate IDs.
    let mut seen_ids = HashSet::with_capacity(blocks.len());
    for block in blocks {
        if !seen_ids.insert(&block.id) {
            return Err(ThreadBlockError::DuplicateBlockId {
                id: block.id.clone(),
            });
        }
    }

    // Validate contiguous order starting at 0.
    let mut actual_orders: Vec<u32> = blocks.iter().map(|b| b.order).collect();
    actual_orders.sort_unstable();
    let expected_orders: Vec<u32> = (0..blocks.len() as u32).collect();
    if actual_orders != expected_orders {
        return Err(ThreadBlockError::NonContiguousOrder {
            expected: expected_orders,
            actual: actual_orders,
        });
    }

    // Per-block validation.
    for block in blocks {
        if block.text.trim().is_empty() {
            return Err(ThreadBlockError::EmptyBlockText {
                block_id: block.id.clone(),
            });
        }
        let weighted_len = tweet_weighted_len(&block.text);
        if weighted_len > MAX_TWEET_CHARS {
            return Err(ThreadBlockError::BlockTextTooLong {
                block_id: block.id.clone(),
                length: weighted_len,
                max: MAX_TWEET_CHARS,
            });
        }
        if block.media_paths.len() > MAX_MEDIA_PER_BLOCK {
            return Err(ThreadBlockError::TooManyMedia {
                block_id: block.id.clone(),
                count: block.media_paths.len(),
                max: MAX_MEDIA_PER_BLOCK,
            });
        }
    }

    Ok(())
}

/// Serialize thread blocks to the versioned JSON format for database storage.
pub fn serialize_blocks_for_storage(blocks: &[ThreadBlock]) -> String {
    let payload = ThreadBlocksPayload {
        version: 1,
        blocks: blocks.to_vec(),
    };
    serde_json::to_string(&payload).expect("ThreadBlocksPayload serialization cannot fail")
}

/// Attempt to deserialize thread blocks from stored content.
///
/// Returns `Some(blocks)` if content is a versioned blocks payload.
/// Returns `None` if content is a legacy format (plain string or string array).
pub fn deserialize_blocks_from_content(content: &str) -> Option<Vec<ThreadBlock>> {
    let parsed: serde_json::Value = serde_json::from_str(content).ok()?;
    if let Some(obj) = parsed.as_object() {
        if obj.contains_key("blocks") {
            let payload: ThreadBlocksPayload = serde_json::from_str(content).ok()?;
            return Some(payload.blocks);
        }
    }
    None
}

impl fmt::Display for ThreadBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ThreadBlock({}, order={})", self.id, self.order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(id: &str, text: &str, order: u32) -> ThreadBlock {
        ThreadBlock {
            id: id.to_string(),
            text: text.to_string(),
            media_paths: vec![],
            order,
        }
    }

    fn make_block_with_media(id: &str, text: &str, order: u32, media: Vec<&str>) -> ThreadBlock {
        ThreadBlock {
            id: id.to_string(),
            text: text.to_string(),
            media_paths: media.into_iter().map(String::from).collect(),
            order,
        }
    }

    #[test]
    fn valid_two_block_thread() {
        let blocks = vec![
            make_block("a", "First tweet", 0),
            make_block("b", "Second tweet", 1),
        ];
        assert!(validate_thread_blocks(&blocks).is_ok());
    }

    #[test]
    fn empty_blocks_rejected() {
        let blocks: Vec<ThreadBlock> = vec![];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::EmptyBlocks));
    }

    #[test]
    fn single_block_rejected() {
        let blocks = vec![make_block("a", "Only tweet", 0)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::SingleBlock));
    }

    #[test]
    fn duplicate_ids_rejected() {
        let blocks = vec![
            make_block("same", "First", 0),
            make_block("same", "Second", 1),
        ];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::DuplicateBlockId { .. }));
    }

    #[test]
    fn non_contiguous_order_rejected() {
        let blocks = vec![make_block("a", "First", 0), make_block("b", "Second", 2)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::NonContiguousOrder { .. }));
    }

    #[test]
    fn order_not_starting_at_zero_rejected() {
        let blocks = vec![make_block("a", "First", 1), make_block("b", "Second", 2)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::NonContiguousOrder { .. }));
    }

    #[test]
    fn empty_text_rejected() {
        let blocks = vec![make_block("a", "  ", 0), make_block("b", "Second", 1)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::EmptyBlockText { .. }));
    }

    #[test]
    fn text_over_limit_rejected() {
        let long_text = "a".repeat(281);
        let blocks = vec![make_block("a", &long_text, 0), make_block("b", "Short", 1)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::BlockTextTooLong { .. }));
    }

    #[test]
    fn too_many_media_rejected() {
        let blocks = vec![
            make_block_with_media(
                "a",
                "Text",
                0,
                vec!["1.jpg", "2.jpg", "3.jpg", "4.jpg", "5.jpg"],
            ),
            make_block("b", "Second", 1),
        ];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::TooManyMedia { .. }));
    }

    #[test]
    fn four_media_accepted() {
        let blocks = vec![
            make_block_with_media("a", "Text", 0, vec!["1.jpg", "2.jpg", "3.jpg", "4.jpg"]),
            make_block("b", "Second", 1),
        ];
        assert!(validate_thread_blocks(&blocks).is_ok());
    }

    #[test]
    fn empty_block_id_rejected() {
        let blocks = vec![make_block("", "First", 0), make_block("b", "Second", 1)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::InvalidBlockId { .. }));
    }

    #[test]
    fn url_weighted_length_respected() {
        // 260 chars of text + a URL = 260 + 23 = 283, over 280
        let padding = "a".repeat(260);
        let text = format!("{padding} https://example.com");
        let blocks = vec![make_block("a", &text, 0), make_block("b", "Short", 1)];
        let err = validate_thread_blocks(&blocks).unwrap_err();
        assert!(matches!(err, ThreadBlockError::BlockTextTooLong { .. }));
    }

    #[test]
    fn url_within_limit_accepted() {
        // 250 chars + URL = 250 + 23 = 273, under 280
        let padding = "a".repeat(250);
        let text = format!("{padding} https://example.com/{}", "x".repeat(76));
        let blocks = vec![make_block("a", &text, 0), make_block("b", "Short", 1)];
        assert!(validate_thread_blocks(&blocks).is_ok());
    }

    #[test]
    fn serialize_and_deserialize_roundtrip() {
        let blocks = vec![
            make_block_with_media("uuid-1", "First tweet", 0, vec!["photo.jpg"]),
            make_block("uuid-2", "Second tweet", 1),
        ];

        let serialized = serialize_blocks_for_storage(&blocks);
        let deserialized = deserialize_blocks_from_content(&serialized);

        assert_eq!(deserialized, Some(blocks));
    }

    #[test]
    fn deserialize_legacy_string_array_returns_none() {
        let legacy = r#"["tweet 1","tweet 2"]"#;
        assert_eq!(deserialize_blocks_from_content(legacy), None);
    }

    #[test]
    fn deserialize_plain_string_returns_none() {
        assert_eq!(deserialize_blocks_from_content("just a tweet"), None);
    }

    #[test]
    fn deserialize_invalid_json_returns_none() {
        assert_eq!(deserialize_blocks_from_content("{not valid"), None);
    }

    #[test]
    fn out_of_order_blocks_accepted_if_contiguous() {
        // Blocks supplied with order [1, 0] — orders are contiguous {0,1}
        let blocks = vec![
            make_block("a", "Second but order 1", 1),
            make_block("b", "First but order 0", 0),
        ];
        assert!(validate_thread_blocks(&blocks).is_ok());
    }
}
