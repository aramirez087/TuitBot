# PRDV1-06: Bilingual + Brand Voice + QA Gates

## Goal

Add product-grade content quality controls so every draft/reply includes a QA
report, enforces brand constraints, and respects bilingual behavior.

## Already built (do not redo)

- Basic brand context fields exist in config (`brand_voice`, `reply_style`, `content_style`).
- Banned phrase checks and semantic dedupe foundations already exist.

## Gaps to implement

1. Brand Voice Profile (structured, per account)
   - tone
   - emoji policy
   - length constraints
   - forbidden words
   - claims rules
   - link policy (allowlist/denylist + UTM rules)
2. Bilingual policy
   - detect source language (`es`/`en` minimum for v1)
   - respond in same language by default (configurable override)
   - glossary for non-translatable product terms
3. QA report artifact for every draft
   - hard flags
   - soft flags
   - recommendations
   - score summary
4. Override workflow
   - hard flag requires explicit editor override and audit note

## Primary code touchpoints

- `crates/tuitbot-core/src/config/mod.rs`
- `crates/tuitbot-core/src/safety/mod.rs`
- `crates/tuitbot-core/src/safety/dedup.rs`
- `crates/tuitbot-core/src/content/generator.rs`
- `crates/tuitbot-server/src/routes/content.rs`
- `crates/tuitbot-server/src/routes/discovery.rs`
- `crates/tuitbot-server/src/routes/approval.rs`
- `dashboard/src/routes/(app)/settings/+page.svelte`
- `dashboard/src/routes/(app)/approval/+page.svelte`

## Implementation tasks

1. Add structured config models.
   - `brand_voice_profile`
   - `language_policy`
   - `link_policy`
   - `glossary_terms`
2. Build QA evaluator pipeline.
   - language match check
   - forbidden phrases and claims checks
   - semantic dedupe threshold
   - link policy and UTM enforcement
3. Persist QA report with draft/approval items.
   - include hard/soft flag arrays and score.
4. Enforce hard flags.
   - block publish/approve unless edited or explicit override with note.
5. Add dashboard surfaces.
   - show QA report in compose/approval views.
   - provide override action with reason capture.
6. Add tests.
   - bilingual detection and same-language reply behavior.
   - glossary preservation tests.
   - hard vs soft flag gating tests.

## Acceptance criteria

- Every draft has a QA report payload.
- Hard flags block publishing until override/edit.
- Reply language follows configured policy and source language defaults.
- Glossary terms remain unmodified when translation would break brand/product terms.

## Verification commands

```bash
cargo test -p tuitbot-core safety
cargo test -p tuitbot-core content
cargo test -p tuitbot-server content
```

## Out of scope

- New X tool creation (PRDV1-04).
- Multi-account/agency permissions (PRDV1-09).
