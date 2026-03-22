# Hook Miner API & Type Contract

**Date:** 2026-03-22
**Session:** 02
**Status:** Locked for implementation (S03-S05)

---

## 1. Angle Taxonomy

Three evidence-based angle types replace the five style-based hook types.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngleType {
    Story,
    Listicle,
    HotTake,
}
```

| Variant | Definition | When to use | Example seed text |
|---------|-----------|-------------|-------------------|
| `Story` | A narrative angle rooted in a personal experience, anecdote, or timeline drawn from the source notes. | Evidence includes a sequence of events, a before/after, or a lived experience. | "I spent 3 months migrating from X to Y. Here's what nobody tells you about..." |
| `Listicle` | A structured angle that surfaces multiple discrete takeaways from the evidence. | Evidence yields 2+ distinct data points, tips, or observations. | "3 things I learned about caching after reading [note title]:" |
| `HotTake` | A provocative angle built on a contradiction, counterintuitive finding, or strong opinion supported by evidence. | Evidence contains a contradiction, a data point that defies conventional wisdom, or a polarizing claim. | "Everyone says you should X. But [data point from note] suggests the opposite." |

The LLM prompt instructs: generate exactly one angle per type when evidence supports it. If evidence only supports 2 types, return 2. If only 1, return 1. Never pad with unsupported angles.

---

## 2. Evidence Taxonomy

Three evidence categories define what the extraction engine looks for in accepted neighbors.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Contradiction,
    DataPoint,
    AhaMoment,
}
```

| Variant | Definition | Extraction rule |
|---------|-----------|-----------------|
| `Contradiction` | Two claims in the evidence set that conflict or create tension. | LLM identifies opposing statements across 2+ neighbor snippets, or between the selection text and a neighbor. |
| `DataPoint` | A specific number, percentage, metric, date, or measurable fact. | Regex pre-filter for numeric patterns (`\d+%`, `\$\d+`, `\d+x`, dates) + LLM confirmation of relevance to the topic. |
| `AhaMoment` | A non-obvious insight, connection, or reframe that the user's notes surface. | LLM identifies a surprising implication or novel synthesis across neighbor content that the user may not have noticed. |

---

## 3. Core Rust Types

These types live in `tuitbot-core`. File placement: `crates/tuitbot-core/src/content/angles.rs` (new module).

```rust
use serde::{Deserialize, Serialize};

/// A single piece of evidence extracted from an accepted graph neighbor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_type: EvidenceType,
    /// Extracted citation text from the neighbor snippet. Max 120 chars.
    pub citation_text: String,
    /// The node_id of the source neighbor.
    pub source_node_id: i64,
    /// Human-readable title of the source note.
    pub source_note_title: String,
    /// Optional heading path within the source note.
    pub source_heading_path: Option<String>,
    /// LLM-assigned confidence that this evidence is meaningful. 0.0-1.0.
    pub confidence: f64,
}

/// A mined angle backed by evidence from the user's vault graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinedAngle {
    pub angle_type: AngleType,
    /// The angle's opening line. Max 280 chars (tweet-length).
    pub seed_text: String,
    pub char_count: usize,
    /// 1-3 evidence items supporting this angle.
    pub evidence: Vec<EvidenceItem>,
    /// "high" or "medium". Based on evidence count and confidence.
    pub confidence: String,
    /// 1-sentence rationale for why this angle was chosen. Shown as tooltip.
    pub rationale: String,
}

/// Output of the angle mining pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngleMiningOutput {
    /// 0-3 mined angles. 0 means fallback should be triggered.
    pub angles: Vec<MinedAngle>,
    /// Non-None when angles is empty. Values: "no_neighbors_accepted",
    /// "insufficient_evidence", "low_evidence_quality".
    pub fallback_reason: Option<String>,
    /// Aggregate quality score across all extracted evidence. 0.0-1.0.
    pub evidence_quality_score: f64,
    pub usage: TokenUsage,
    pub model: String,
    pub provider: String,
}
```

### Confidence assignment rules

| Condition | Confidence |
|-----------|-----------|
| Angle has 2-3 evidence items AND avg evidence confidence >= 0.6 | `"high"` |
| All other cases (1 evidence item, or avg confidence < 0.6) | `"medium"` |

---

## 4. API Endpoint

### `POST /api/assist/angles`

New endpoint in `crates/tuitbot-server/src/routes/assist/angles.rs`. Sits alongside the existing `hooks.rs`.

### Request

```rust
#[derive(Deserialize)]
pub struct AssistAnglesRequest {
    /// The topic derived from the user's selection text, note title, or heading.
    pub topic: String,
    /// node_ids of accepted GraphSuggestionCards.
    pub accepted_neighbor_ids: Vec<i64>,
    /// Obsidian selection session ID (for selection context).
    #[serde(default)]
    pub session_id: Option<String>,
    /// For From Vault flow (not used in V1 angle mining, reserved).
    #[serde(default)]
    pub selected_node_ids: Option<Vec<i64>>,
}
```

### Response

```rust
#[derive(Serialize)]
pub struct AssistAnglesResponse {
    pub angles: Vec<MinedAngleDto>,
    /// Non-null when evidence is insufficient. Client shows fallback UX.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub topic: String,
    /// Reuse existing VaultCitation type for provenance.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub vault_citations: Vec<VaultCitation>,
}

#[derive(Serialize)]
pub struct MinedAngleDto {
    pub angle_type: String,     // "story" | "listicle" | "hot_take"
    pub seed_text: String,
    pub char_count: usize,
    pub evidence: Vec<EvidenceItemDto>,
    pub confidence: String,     // "high" | "medium"
    pub rationale: String,
}

#[derive(Serialize)]
pub struct EvidenceItemDto {
    pub evidence_type: String,  // "contradiction" | "data_point" | "aha_moment"
    pub citation_text: String,
    pub source_node_id: i64,
    pub source_note_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_heading_path: Option<String>,
}
```

### Error Responses

| Status | Condition | Body |
|--------|-----------|------|
| 400 | `accepted_neighbor_ids` is empty | `{ "error": "At least one accepted neighbor is required" }` |
| 400 | `topic` is empty | `{ "error": "Topic is required" }` |
| 500 | LLM call failed | `{ "error": "Angle mining failed: {details}" }` |
| 500 | Evidence extraction failed | `{ "error": "Evidence extraction failed: {details}" }` |

---

## 5. Frontend TypeScript Types

These types live in `dashboard/src/lib/api/types.ts`.

```typescript
export interface MinedAngle {
    angle_type: 'story' | 'listicle' | 'hot_take';
    seed_text: string;
    char_count: number;
    evidence: EvidenceItem[];
    confidence: 'high' | 'medium';
    rationale: string;
}

export interface EvidenceItem {
    evidence_type: 'contradiction' | 'data_point' | 'aha_moment';
    citation_text: string;
    source_node_id: number;
    source_note_title: string;
    source_heading_path: string | null;
}

export interface AssistAnglesResponse {
    angles: MinedAngle[];
    fallback_reason: string | null;
    topic: string;
    vault_citations?: VaultCitation[];
}

export interface AssistAnglesRequest {
    topic: string;
    accepted_neighbor_ids: number[];
    session_id?: string;
    selected_node_ids?: number[];
}
```

---

## 6. Fallback Logic

### Constants

```rust
const MIN_EVIDENCE_QUALITY: f64 = 0.3;
const MIN_EVIDENCE_COUNT: usize = 2;
```

These constants live in the extraction engine (`tuitbot-core`), not in the server layer.

### Server-side pseudocode

```
fn generate_mined_angles(topic, accepted_neighbor_ids, selection_context, voice, persona):

    if accepted_neighbor_ids.is_empty():
        return AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("no_neighbors_accepted"),
            evidence_quality_score: 0.0,
            ..
        }

    neighbors = fetch_neighbor_content(accepted_neighbor_ids)
    evidence_items = extract_evidence(neighbors, selection_context)

    if evidence_items.len() < MIN_EVIDENCE_COUNT:
        return AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("insufficient_evidence"),
            evidence_quality_score: avg_confidence(evidence_items),
            ..
        }

    quality_score = avg_confidence(evidence_items)
    if quality_score < MIN_EVIDENCE_QUALITY:
        return AngleMiningOutput {
            angles: vec![],
            fallback_reason: Some("low_evidence_quality"),
            evidence_quality_score: quality_score,
            ..
        }

    angles = llm_generate_angles(topic, evidence_items, voice, persona)
    angles = angles.filter(|a| !a.evidence.is_empty())  // drop empty-evidence angles

    return AngleMiningOutput {
        angles,
        fallback_reason: if angles.is_empty() { Some("all_angles_filtered") } else { None },
        evidence_quality_score: quality_score,
        ..
    }
```

### Client-side routing pseudocode

```typescript
async function handleGenerate() {
    if (synthesisEnabled && acceptedNeighbors.size > 0) {
        // Angle mining path
        angleLoading = true;
        try {
            const result = await api.assist.angles({
                topic,
                accepted_neighbor_ids: [...acceptedNeighbors.keys()],
                session_id: sessionId,
            });

            if (result.fallback_reason !== null || result.angles.length === 0) {
                // Show fallback state (NOT HookPicker immediately)
                showFallback = true;
                fallbackReason = result.fallback_reason;
            } else {
                // Show angle cards
                angleOptions = result.angles;
            }
        } catch (e) {
            angleError = e.message;
        } finally {
            angleLoading = false;
        }
    } else {
        // Generic hook path (unchanged)
        handleGenerateHooks();  // existing function
    }
}
```

### Fallback action handlers

```typescript
function handleUseGenericHooks() {
    // Transition from fallback to HookPicker
    showFallback = false;
    handleGenerateHooks();  // calls POST /api/assist/hooks
}

function handleBackToNeighbors() {
    // Return to GraphSuggestionCards
    showFallback = false;
    angleOptions = null;
    // acceptedNeighbors, dismissedNodeIds, synthesisEnabled preserved
}
```

---

## 7. How Selected Angle Feeds Into Draft Generation

When the user selects an angle and clicks "Use this angle":

```typescript
async function handleAngleSelected(angle: MinedAngle, format: 'tweet' | 'thread') {
    const nodeIds = selection.resolved_node_id ? [selection.resolved_node_id] : [];

    // Add accepted neighbor node IDs for RAG context
    const neighborProv = [];
    for (const [nid, { neighbor }] of acceptedNeighbors) {
        if (!nodeIds.includes(nid)) nodeIds.push(nid);
        neighborProv.push({
            node_id: nid,
            edge_type: neighbor.reason,
            edge_label: neighbor.reason_label,
        });
    }

    await ongenerate(
        nodeIds,
        format,
        [angle.seed_text],          // highlights = angle seed text
        angle.angle_type,            // hookStyle = angle type
        neighborProv.length > 0 ? neighborProv : undefined
    );
}
```

This calls the same `ongenerate` callback with the same signature. The existing compose -> approve -> publish pipeline is untouched. Only the first step (what populates `highlights` and `hookStyle`) changes.

### Parameter mapping

| ongenerate param | HookPicker (current) | AngleCards (new) |
|-----------------|---------------------|------------------|
| `nodeIds` | `[resolved_node_id]` + accepted neighbor IDs | Same |
| `format` | `outputFormat` | Same |
| `highlights` | `[hook.text]` | `[angle.seed_text]` |
| `hookStyle` | `hook.style` (e.g., `"question"`) | `angle.angle_type` (e.g., `"story"`) |
| `neighborProvenance` | accepted neighbor provenance | Same |

---

## 8. Backward Compatibility Guarantees

| Component | Status |
|-----------|--------|
| `POST /api/assist/hooks` | Unchanged. Remains the primary endpoint for generic hooks. |
| `HookPicker.svelte` | Unchanged. Used for generic fallback and From Vault chunk-selection path. |
| `GraphSuggestionCards.svelte` | Unchanged. Accept/dismiss flow feeds into angle mining. |
| `VaultFooter.svelte` | Unchanged. "Generate" button behavior changes in `VaultSelectionReview` only. |
| `ongenerate` callback signature | Unchanged. AngleCards passes the same parameter types. |
| `api.assist.hooks()` client function | Unchanged. Remains available for fallback. |
| Approval queue `action_type` | Unchanged. Angles produce the same `"tweet"` / `"thread_tweet"` items. |
| Provenance propagation | Unchanged. `neighborProvenance` parameter is already supported. |

---

## 9. Provenance Extension

When an angle is selected, its evidence items create additional provenance records:

| Field | Value |
|-------|-------|
| `entity_type` | `"approval_queue"` (same as current hook flow) |
| `node_id` | `evidence_item.source_node_id` |
| `chunk_id` | Resolved from the neighbor's `best_chunk_id` |
| `source_path` | Resolved from the neighbor's note path |
| `heading_path` | `evidence_item.source_heading_path` |
| `snippet` | `evidence_item.citation_text` |
| `edge_type` | The neighbor's `reason` (e.g., `"backlink"`, `"shared_tag"`) |
| `edge_label` | The neighbor's `reason_label` |

This uses the existing `insert_links_for()` function. No schema changes needed.

---

## 10. Evidence Extraction Pipeline (Internal)

This section specifies the extraction strategy for implementors. It is not user-facing.

### Step 1: Fetch neighbor content

For each `accepted_neighbor_id`, load:
- `node_id`, `note_title`, `heading_path`
- Best chunk content (from `best_chunk_id` on the graph neighbor)
- Full snippet (up to 500 chars around the best chunk)

### Step 2: Regex pre-filter for data points

Before LLM extraction, scan all neighbor content for numeric patterns:
- Percentages: `\d+(\.\d+)?%`
- Dollar amounts: `\$[\d,]+(\.\d+)?`
- Multipliers: `\d+(\.\d+)?x`
- Dates: `\d{4}-\d{2}-\d{2}` or common date formats
- Counts: `\d+\s+(users|customers|downloads|revenue|etc.)`

Matches are tagged as candidate `DataPoint` evidence and passed to the LLM for relevance confirmation.

### Step 3: LLM evidence extraction

Single LLM call with structured output. The prompt receives:
- The selection text (topic context)
- All neighbor snippets with their node_ids and titles
- Candidate data points from regex pre-filter

The LLM returns a JSON array of `EvidenceItem` objects, each with:
- `evidence_type` (must be one of the three enum values)
- `citation_text` (must be a direct quote or close paraphrase, max 120 chars)
- `source_node_id` (must reference one of the provided neighbors)
- `confidence` (0.0-1.0)

### Step 4: Validation and filtering

- Reject evidence items that reference node_ids not in the accepted set
- Reject evidence items with `citation_text` > 120 chars (truncate to 120)
- Reject evidence items with `confidence` < 0.1 (noise floor)
- Deduplicate by `(evidence_type, source_node_id)` — keep highest confidence

---

## 11. Token Budget

| Component | Budget |
|-----------|--------|
| Evidence extraction LLM call | Max 2000 input tokens (neighbor content) + 500 output tokens |
| Angle generation LLM call | Max 1000 input tokens (topic + evidence) + 800 output tokens |
| Total per angle mining request | ~4300 tokens max |

These budgets constrain the number of neighbors that can be included. If accepted neighbors exceed the input budget, the system includes neighbors in descending score order until the budget is reached.
