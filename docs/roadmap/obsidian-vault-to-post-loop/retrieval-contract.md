# Retrieval Contract — Vault Fragment Retrieval Engine

## Overview

The retrieval engine provides account-scoped vault fragment retrieval for
LLM prompt enrichment. It operates as Tier 2 in the three-tier context model,
sitting between winning ancestors (Tier 1) and cold-start seeds (Tier 3).

## Three-Tier Context Model

| Tier | Source | Signal Type | When Used |
|------|--------|-------------|-----------|
| 1 | Winning ancestors | Behavioral (what worked) | Always attempted |
| 2 | Vault fragments | Knowledge (user expertise) | Always attempted |
| 3 | Content seeds | LLM-extracted hooks | Fallback when no fragments |

### Combination Rules

- **Both ancestors + fragments** → combined prompt with split budgets
- **Ancestors only** → full budget for ancestor patterns
- **Fragments only** → full budget for vault knowledge
- **Neither** → fall back to content seeds
- **None of the above** → empty prompt block (fail-open)

## Retrieval Ranking

### Fragment Retrieval (`retrieve_vault_fragments`)

1. **Selected-note bias** (optional): When `selected_node_ids` is provided,
   chunks from those notes are retrieved first via `get_chunks_for_nodes_with_context`.
2. **Keyword search**: Remaining slots filled via `search_chunks_with_context`
   using LIKE-based matching on `chunk_text`.
3. **Ordering**: Results ordered by `retrieval_boost DESC` (default 1.0,
   range 0.1–5.0). Higher boost = more likely to appear.
4. **Deduplication**: By `chunk_id` — no chunk appears twice even if matched
   by both selected-note and keyword paths.

### Ancestor Retrieval (`retrieve_ancestors`)

1. **Engagement filter**: `engagement_score >= 0.1` (filters bottom ~10%)
2. **Topic matching**: Tweets matched by `topic IN (keywords)`, replies by
   `reply_content LIKE '%keyword%'`
3. **Recency decay**: `weight = engagement_score * exp(-0.693 * days / 14)`
4. **Account scoping**: `WHERE account_id = ?` on both `original_tweets` and
   `replies_sent`

## Prompt Shaping

### Budget Allocation

| Section | Max Chars | Constant |
|---------|-----------|----------|
| Total RAG block | 2000 | `RAG_MAX_CHARS` |
| Ancestors (combined mode) | 800 | `MAX_ANCESTOR_CHARS` |
| Fragments | 1000 | `MAX_FRAGMENT_CHARS` |
| Max fragments | 5 | `MAX_FRAGMENTS` |

### Prompt Format

**Ancestors section:**
```
Winning patterns from your best-performing content:
1. [archetype] (type): "preview..."
Use these patterns as inspiration but don't copy them directly.
```

**Fragments section:**
```
Relevant knowledge from your notes:
1. [heading_path] (from: title): "excerpt..."
Reference these insights to ground your response in your own expertise.
```

### Truncation

- Each entry is added only if it fits within the section budget
- Combined blocks are truncated at `RAG_MAX_CHARS` on a char boundary
- Fragment text previews are capped at 200 chars per entry

## Citation Schema

```rust
pub struct VaultCitation {
    pub chunk_id: i64,        // Stable ID for the content chunk
    pub node_id: i64,         // Parent content node
    pub heading_path: String, // e.g., "# Title > ## Section"
    pub source_path: String,  // Relative file path
    pub source_title: Option<String>,
    pub snippet: String,      // First ~120 chars
    pub retrieval_boost: f64, // Current boost value
}
```

Citations are populated in `DraftContext.vault_citations` whenever fragments
are included. They are not yet exposed in API responses — that is a future
session deliverable.

## Account Isolation Guarantees

Every retrieval query is scoped by `account_id`:

- `get_scored_ancestors()` → `WHERE ot.account_id = ?` / `WHERE rs.account_id = ?`
- `search_chunks_with_context()` → `WHERE cc.account_id = ?`
- `get_chunks_for_nodes_with_context()` → `WHERE cc.account_id = ?`
- `get_seeds_for_context_for()` → `WHERE ds.account_id = ?`

No cross-account data leakage is possible in the retrieval path.

## Guardrails

1. **No content logging**: Chunk text and note content are never logged
2. **Max fragments**: Hard cap at 5 fragments per context build
3. **Char limits**: All prompt sections are budget-capped
4. **Fail-open**: Retrieval errors produce empty context, not failures
5. **Idempotent**: Same inputs produce same retrieval results (no side effects)
