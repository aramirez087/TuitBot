# Retrieval Ranking Specification

## Algorithm: Reciprocal Rank Fusion (RRF)

The evidence endpoint blends results from up to three heterogeneous signal sources using RRF, a rank-based fusion method that requires no score normalization.

### Formula

For each candidate chunk `c` that appears in one or more result lists:

```
rrf_score(c) = Σ  1 / (k + rank_i(c))
               i∈lists
```

Where:
- `k = 60` (constant from the original RRF paper, Cormack et al. 2009)
- `rank_i(c)` = 1-indexed rank of chunk `c` in result list `i`
- The sum runs over all lists in which chunk `c` appears

### Signal Sources

| Source | Input | Ranking Criterion | Headroom |
|--------|-------|-------------------|----------|
| **Semantic** | Query embedding vs chunk embeddings | Cosine similarity (ascending distance) | `limit` |
| **Keyword** | Whitespace-split query tokens | `LIKE '%token%'` match, ordered by `retrieval_boost DESC` | `limit * 2` |
| **Graph** | Selected node IDs | Chunks from those nodes, ordered by `retrieval_boost DESC` | `limit * 2` |

### Worked Example

Query: "install CLI tool"
Limit: 5

**Semantic results** (from embedding search):
1. chunk_42 (distance=0.05)
2. chunk_17 (distance=0.12)
3. chunk_88 (distance=0.18)

**Keyword results** (from LIKE search):
1. chunk_17 (boost=2.0)
2. chunk_55 (boost=1.5)
3. chunk_42 (boost=1.0)
4. chunk_99 (boost=0.8)

**RRF scores** (k=60):

| Chunk | Semantic Rank | Keyword Rank | RRF Score | Match Reason |
|-------|--------------|--------------|-----------|--------------|
| chunk_42 | 1 | 3 | 1/61 + 1/63 = 0.01639 + 0.01587 = **0.03226** | hybrid |
| chunk_17 | 2 | 1 | 1/62 + 1/61 = 0.01613 + 0.01639 = **0.03252** | hybrid |
| chunk_88 | 3 | — | 1/63 = **0.01587** | semantic |
| chunk_55 | — | 2 | 1/62 = **0.01613** | keyword |
| chunk_99 | — | 4 | 1/64 = **0.01563** | keyword |

**Final ranking** (sorted by RRF score desc):
1. chunk_17 (0.03252, hybrid)
2. chunk_42 (0.03226, hybrid)
3. chunk_55 (0.01613, keyword)
4. chunk_88 (0.01587, semantic)
5. chunk_99 (0.01563, keyword)

## MatchReason Classification

Each result is classified based on which signal lists it appeared in:

| Appears In | MatchReason |
|------------|-------------|
| Semantic only | `semantic` |
| Keyword only | `keyword` |
| Graph only | `graph` |
| Any 2+ lists | `hybrid` |

## Score Interpretation

RRF scores are **relative**, not absolute. Key properties:
- Scores are always positive (> 0)
- Maximum single-list score: `1/(k+1)` ≈ 0.0164
- Maximum multi-list score: `3/(k+1)` ≈ 0.0492 (appearing rank 1 in all three lists)
- Scores are useful for ordering, not for threshold-based filtering
- The frontend should show relevance badges (High/Medium/Low) rather than raw scores

## Fallback Behavior Matrix

| Condition | Behavior | Results |
|-----------|----------|---------|
| Embedding provider not configured | Skip semantic search, keyword-only | `match_reason: keyword` |
| Semantic index empty | Skip semantic search, keyword-only | `match_reason: keyword` |
| Embedding API call fails | Log warning, fall back to keyword | `match_reason: keyword` |
| Embedding dimension mismatch | Log warning, fall back to keyword | `match_reason: keyword` |
| No keyword matches | Semantic-only results (if available) | `match_reason: semantic` |
| No results at all | Empty results array | `results: []` |
| mode=semantic, no provider | Empty results | `results: []` |
| mode=keyword | Skip semantic entirely | `match_reason: keyword` |

In all fallback cases, `index_status.freshness_pct` in the response tells the frontend the state of the index, enabling degraded-state UI.

## Performance Budget

| Operation | Target Latency | Notes |
|-----------|---------------|-------|
| Query embedding | <50ms (OpenAI), <20ms (Ollama) | Single short text, network bound |
| Semantic search (brute-force) | <10ms for 50K vectors | Linear scan with SIMD-friendly ops |
| Keyword search (SQLite LIKE) | <20ms for 50K chunks | Indexed by account_id |
| RRF fusion | <1ms | In-memory HashMap merge |
| **Total evidence query** | **<100ms** | Sum of above, dominated by embedding |

## Index Freshness

The `freshness_pct` field indicates what percentage of chunks have up-to-date embeddings:
- **100%**: All chunks are embedded with the current model
- **0%**: No embeddings exist (provider not configured or never ran)
- **Partial**: Indexer is catching up after content changes

The frontend can use this to show:
- Green indicator: freshness ≥ 90%
- Yellow indicator: 50% ≤ freshness < 90%
- Red indicator: freshness < 50%
- "Indexing unavailable" when `provider_configured: false`
