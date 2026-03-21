# Hook Generation Contract

## Overview

`POST /api/assist/hooks` generates 5 differentiated tweet hooks from a given topic, optionally grounded in Ghostwriter vault context (selection or indexed nodes). Each hook is a standalone tweet (max 280 chars) tagged with a style and confidence heuristic.

## Endpoint

### `POST /api/assist/hooks`

**Request:**

```json
{
  "topic": "Why testing saves you time",
  "selected_node_ids": [42, 87],
  "session_id": "abc-def-123"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `topic` | string | yes | The subject to generate hooks about |
| `selected_node_ids` | number[] | no | Vault node IDs for RAG context |
| `session_id` | string | no | Ghostwriter selection session ID (takes priority over `selected_node_ids`) |

**Response:**

```json
{
  "hooks": [
    {
      "style": "question",
      "text": "What if your tests could write themselves?",
      "char_count": 42,
      "confidence": "high"
    },
    {
      "style": "contrarian_take",
      "text": "Most devs test too much. Here's why that hurts your product.",
      "char_count": 60,
      "confidence": "high"
    }
  ],
  "topic": "Why testing saves you time",
  "vault_citations": []
}
```

| Field | Type | Description |
|---|---|---|
| `hooks` | HookOptionDto[] | 3-5 differentiated hook options |
| `topic` | string | Echo of the input topic |
| `vault_citations` | VaultCitation[] | Vault citations used for RAG context (omitted when empty) |

### HookOptionDto

| Field | Type | Description |
|---|---|---|
| `style` | string | TweetFormat name (e.g. `question`, `contrarian_take`, `tip`) |
| `text` | string | The hook tweet text (max 280 chars) |
| `char_count` | number | Character count |
| `confidence` | string | `"high"` if <= 240 chars, `"medium"` otherwise |

## Decisions

### D1: Reuse TweetFormat as hook style taxonomy

The existing `TweetFormat` enum (7 variants: `list`, `contrarian_take`, `most_people_think_x`, `storytelling`, `before_after`, `question`, `tip`) defines differentiated tweet structures with prompt fragments. Rather than inventing a new taxonomy, we pick 5 of these per request. A hook IS a tweet — reusing the format system keeps the pipeline composable.

### D2: Single LLM call with structured multi-hook output

All 5 hooks are generated in one LLM call using `STYLE: <name>\nHOOK: <text>\n---` delimiters. This is cheaper (1 call vs 5), faster, and naturally produces differentiated output since the LLM sees all hooks in context.

Fallback: if the LLM returns fewer than 3 hooks, one retry is attempted. Minimum 3 hooks returned. Maximum capped at 5.

### D3: generate_hooks lives in ContentGenerator (tuitbot-core)

Business logic stays in `tuitbot-core` per the server boundary constraint. The server route is a thin adapter that resolves context and delegates.

### D4: Lightweight metadata, not analytics

Each hook carries `style`, `text`, `char_count`, and `confidence`. No engagement prediction or analytics subsystem — that's a future session concern.

### D5: Selection context flows through existing RAG pipeline

When `session_id` is provided:
1. Fetch selection from `vault_selections` (Session 3 infra)
2. If `resolved_node_id` exists, resolve via `build_draft_context_with_selection`
3. If only `selected_text` is available, inject it directly as additional context
4. Falls back to keyword-based RAG when no selection context

New `resolve_selection_rag_context` helper in `rag_helpers.rs` encapsulates this.

### D6: Hook style selection strategy

Every generation includes `question` and `contrarian_take` (strongest hook formats). The remaining 3 are randomly selected from `list`, `most_people_think_x`, `storytelling`, `before_after`, `tip`. This ensures variety across requests while always including the two most engagement-friendly formats.

## Parser Contract

The `parse_hooks_response` function (in `parser.rs`) expects:

```
STYLE: <style_name>
HOOK: <hook text>
---
STYLE: <style_name>
HOOK: <hook text>
---
...
```

Behavior:
- Blocks separated by `---`
- `STYLE:` line is optional; defaults to `"general"` if missing
- `HOOK:` line is required; empty hooks are skipped
- Last block doesn't need a trailing `---`
- Same pattern as `parse_seed_response` in `seed_worker.rs`

## Privacy

Hooks are generated content, not raw vault text. Safe for all deployment modes (Desktop, Self-host, Cloud). The `selected_text` from a selection is only used as LLM prompt context — never returned in the hooks response.

## File Changes

| File | Change |
|---|---|
| `crates/tuitbot-core/src/content/generator/mod.rs` | `HookOption`, `HookGenerationOutput`, `generate_hooks()`, `select_hook_styles()`, `build_hook_options()` |
| `crates/tuitbot-core/src/content/generator/parser.rs` | `parse_hooks_response()` |
| `crates/tuitbot-core/src/content/generator/tests.rs` | 15 new tests for hooks + parser |
| `crates/tuitbot-server/src/routes/assist/mod.rs` | Module directory split (was `assist.rs`) |
| `crates/tuitbot-server/src/routes/assist/hooks.rs` | `POST /api/assist/hooks` handler + 4 tests |
| `crates/tuitbot-server/src/routes/rag_helpers.rs` | `resolve_selection_rag_context()`, `SelectionRagContext` |
| `crates/tuitbot-server/src/lib.rs` | Route registration |
| `dashboard/src/lib/api/types.ts` | `HookOption`, `AssistHooksResponse` |
| `dashboard/src/lib/api/client.ts` | `api.assist.hooks()` |
