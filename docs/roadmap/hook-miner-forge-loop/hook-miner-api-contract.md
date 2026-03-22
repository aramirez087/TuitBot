# Hook Miner API Contract — Implemented

**Date:** 2026-03-22
**Session:** 03
**Status:** Implemented

---

## Endpoint

```
POST /api/assist/angles
```

Requires authentication (bearer token or session cookie). Account-scoped.

---

## Request

```json
{
  "topic": "Growth metrics for Q3",
  "accepted_neighbor_ids": [42, 57, 63],
  "session_id": "sel-abc-123",
  "selected_node_ids": [10, 20]
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `topic` | string | yes | The content topic to generate angles for. |
| `accepted_neighbor_ids` | int[] | yes | Node IDs of accepted neighbor notes (from the graph suggestion step). |
| `session_id` | string | no | Selection session ID for vault selection context. |
| `selected_node_ids` | int[] | no | Explicit node IDs for vault context (fallback if no session). |

---

## Response (Success)

```json
{
  "angles": [
    {
      "angle_type": "story",
      "seed_text": "We grew 45% but our costs told a different story...",
      "char_count": 52,
      "evidence": [
        {
          "evidence_type": "data_point",
          "citation_text": "Revenue grew 45% in Q3",
          "source_node_id": 42,
          "source_note_title": "Q3 Metrics",
          "source_heading_path": "# Revenue"
        },
        {
          "evidence_type": "contradiction",
          "citation_text": "Operating costs increased 60%",
          "source_node_id": 57,
          "source_note_title": "Cost Analysis"
        }
      ],
      "confidence": "high",
      "rationale": "Tension between growth and cost creates a compelling narrative arc."
    }
  ],
  "topic": "Growth metrics for Q3",
  "vault_citations": [
    {
      "chunk_id": 101,
      "node_id": 42,
      "heading_path": "# Revenue",
      "source_path": "notes/q3-metrics.md",
      "source_title": "Q3 Metrics",
      "snippet": "Revenue grew 45% in Q3",
      "retrieval_boost": 1.0
    }
  ]
}
```

---

## Response (Fallback)

When evidence is insufficient, the endpoint returns an empty `angles` array with a `fallback_reason`:

```json
{
  "angles": [],
  "fallback_reason": "insufficient_evidence",
  "topic": "Growth metrics for Q3",
  "vault_citations": []
}
```

### Fallback Reasons

| Reason | Trigger |
|--------|---------|
| `no_neighbors_accepted` | Empty `accepted_neighbor_ids` passed validation but yielded no vault chunks. |
| `insufficient_evidence` | Fewer than 2 evidence items survived validation. |
| `low_evidence_quality` | Average evidence confidence below 0.3. |
| `all_angles_filtered` | Angle generation succeeded but all angles had empty evidence after mapping. |

---

## Evidence Mining Pipeline

```
Neighbor notes
    │
    ▼
┌─────────────────────┐
│ Regex Pre-filter     │  Scan for: percentages, dollar amounts,
│ (pure, no LLM)      │  multipliers, ISO dates, count+unit patterns
└─────────┬───────────┘
          │ candidates
          ▼
┌─────────────────────┐
│ LLM Extraction      │  System prompt with topic + neighbor snippets
│ (temp: 0.3)         │  Returns JSON array of evidence items
└─────────┬───────────┘
          │ raw evidence
          ▼
┌─────────────────────┐
│ Validation           │  1. Reject invalid node_ids
│                      │  2. Truncate citations > 120 chars
│                      │  3. Reject confidence < 0.1
│                      │  4. Deduplicate by (type, node_id)
└─────────┬───────────┘
          │ validated evidence
          ▼
┌─────────────────────┐
│ Threshold Gates      │  MIN_EVIDENCE_COUNT = 2
│                      │  MIN_EVIDENCE_QUALITY = 0.3
└─────────┬───────────┘
          │ (pass)
          ▼
┌─────────────────────┐
│ Angle Generation     │  Up to 3 angles: story, listicle, hot_take
│ (temp: 0.8)         │  Evidence-constrained, voice/persona aware
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Parse & Filter       │  Map evidence IDs back to items
│                      │  Drop angles with empty evidence
│                      │  Cap at 3 angles
└─────────────────────┘
```

---

## Privacy Rules

- Only `citation_text` (max 120 chars), `source_note_title`, and `heading_path` are surfaced — never raw note bodies.
- All DB queries are scoped by `account_id` from `AccountContext`.
- Neighbor content is fetched via `get_chunks_for_nodes_with_context` with account isolation.
- Snippet text sent to the LLM is truncated to 500 chars per neighbor.

---

## Evidence Types

| Type | Description | Extraction cue |
|------|-------------|---------------|
| `contradiction` | Opposing claims across different notes | Conflicting assertions on the same topic |
| `data_point` | A specific statistic, metric, or quantified claim | Numbers, percentages, dollar amounts |
| `aha_moment` | A non-obvious insight or surprising connection | Unexpected correlations, counterintuitive findings |

## Angle Types

| Type | Description | Best when |
|------|-------------|-----------|
| `story` | Narrative-driven, grounded in observed events | Evidence includes contradictions or temporal sequence |
| `listicle` | Structured list format | Multiple data points or aha moments |
| `hot_take` | Bold opinion backed by evidence | Strong contradiction or surprising data |

---

## Constants (in `tuitbot-core`)

| Constant | Value | Location |
|----------|-------|----------|
| `MIN_EVIDENCE_COUNT` | 2 | `content::angles` |
| `MIN_EVIDENCE_QUALITY` | 0.3 | `content::angles` |
| `MAX_CITATION_CHARS` | 120 | `content::evidence` |
| `MIN_CONFIDENCE_FLOOR` | 0.1 | `content::evidence` |

---

## Backward Compatibility

- `POST /api/assist/hooks` is completely untouched.
- No changes to `HookOption`, `HookGenerationOutput`, or any existing endpoint.
- The new endpoint is purely additive.
