# Core Contract — Automatic Vault Context Primitives

Documents the new API surface added in Session 02 for composer RAG support.

## New Methods

### `BusinessProfile::draft_context_keywords() -> Vec<String>`

**Location:** `crates/tuitbot-core/src/config/types.rs`

**Purpose:** Single source of truth for the merged keyword set used across draft workflows, composer RAG resolution, and engagement scoring.

**Assembly order:** `product_keywords` → `competitor_keywords` → `effective_industry_topics()`

**Semantics:**
- Returns an owned `Vec<String>` (all downstream consumers need owned data).
- Does NOT dedup. When `industry_topics` is empty, `effective_industry_topics()` falls back to `product_keywords`, causing them to appear twice. This preserves the exact behavior of the original manual assembly in `draft.rs` and `engagement.rs`. Downstream consumers (RAG retrieval SQL, scoring engine) tolerate duplicates.
- Deterministic ordering: product first, then competitor, then industry.

**Call sites after Session 02:**
- `workflow/draft.rs` — autopilot draft step
- `context/engagement.rs` — engagement recommendation engine
- (Future) `routes/assist.rs` — composer RAG resolver (Session 03)

### `ContentGenerator::improve_draft_with_context(draft, tone_cue, rag_context) -> Result<GenerationOutput, LlmError>`

**Location:** `crates/tuitbot-core/src/content/generator/mod.rs`

**Purpose:** Rewrite/improve a draft tweet with optional RAG context injected into the system prompt, enabling the composer improve endpoint to benefit from vault context.

**Prompt layout:**
```
System prompt:
  You are {product_name}'s social media voice. {description}.
  {voice_section}
  {persona_section}
  {rag_section}           <-- Automatic vault context (new, from rag_context param)

  Task: Rewrite and improve the draft tweet below.
  Keep the core message but make it sharper, more engaging, and better-written.
  Tone/style directive: {tone_cue}   <-- User-supplied quick cue (existing)

  Rules: ...
```

**Dual-context design:** The `rag_context` (vault patterns, winning ancestors) occupies the system prompt's context section. The `tone_cue` (user's style directive) occupies the task instruction section. They serve different roles (reference material vs. directive) and cannot conflict.

**Implementation pattern:** Follows the `_inner` delegation pattern used by `generate_tweet`, `generate_reply`, and `generate_thread`. The existing `improve_draft()` delegates to `improve_draft_inner(draft, tone_cue, None)` preserving full backward compatibility.

### `ContentGenerator::business() -> &BusinessProfile`

**Location:** `crates/tuitbot-core/src/content/generator/mod.rs`

**Purpose:** Read-only accessor for the business profile stored in the generator. Server routes (Session 03) need this to call `draft_context_keywords()` on the same profile used for prompt generation.

**Design choice:** Getter over `pub` field to preserve encapsulation. Server routes should not mutate the business profile on an existing generator instance.

## Backward Compatibility

- `improve_draft(draft, tone_cue)` — unchanged signature, delegates to inner with `rag_context: None`. All existing callers are source-compatible.
- `draft.rs` and `engagement.rs` — refactored from 3-line manual assembly to `draft_context_keywords()`. Produces identical keyword vectors. Pure refactor, no semantic change.
- No HTTP payload changes. No frontend changes.

## Test Coverage

| Test | What it verifies |
|------|-----------------|
| `draft_context_keywords_empty_profile` | Default profile returns empty vec |
| `draft_context_keywords_only_product` | Fallback duplication documented |
| `draft_context_keywords_all_keyword_types` | Correct ordering: product → competitor → industry |
| `draft_context_keywords_deduplication_not_applied` | Overlapping keywords preserved |
| `improve_draft_success` | Basic improvement works |
| `improve_draft_with_tone_cue` | Tone cue accepted |
| `improve_draft_with_context_success` | RAG context accepted |
| `improve_draft_with_context_none_matches_base` | None context matches base behavior |
| `improve_draft_with_context_injects_rag_in_prompt` | RAG block appears in system prompt (captured) |
| `improve_draft_with_context_no_rag_when_none` | No RAG content when context is None (captured) |
