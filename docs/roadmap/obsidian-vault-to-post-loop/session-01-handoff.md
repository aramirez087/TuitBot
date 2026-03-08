# Session 01 Handoff — Planning & Charter

## What Changed

Five planning documents created under `docs/roadmap/obsidian-vault-to-post-loop/`:

| Document | Purpose |
|----------|---------|
| `charter.md` | Problem statement, non-negotiable defaults, 8 design decisions, scope, success criteria |
| `product-model.md` | 9 entity definitions, lifecycle diagram, re-chunking strategy, account isolation model |
| `ux-blueprint.md` | UX for onboarding, settings health, vault page, From Vault composer, reply citations, navigation |
| `implementation-plan.md` | Sessions 02–13 with concrete files, defaults, and verification for each |
| `session-01-handoff.md` | This file |

No source code was modified.

## What Remains

12 implementation sessions (02–13):

| Session | Scope | Layer |
|---------|-------|-------|
| 02 | Fragment chunking pipeline | Backend |
| 03 | Provenance schema & storage | Backend |
| 04 | Chunk-level RAG retrieval | Backend |
| 05 | Citation metadata in responses | Backend |
| 06 | Account isolation audit | Backend |
| 07 | Draft provenance wiring | Backend |
| 08 | Loop-back learning | Backend |
| 09 | Vault health API | Backend |
| 10 | From Vault composer API | Backend |
| 11 | Vault health dashboard | Frontend |
| 12 | From Vault composer & citations UI | Frontend |
| 13 | Integration testing & documentation | Full stack |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Heading-based chunking produces bad results for flat notes | Medium | Low | Fallback to paragraph splitting for notes without headings |
| Keyword retrieval returns irrelevant chunks | Medium | Medium | Bounded to max 3 chunks; retrieval_boost learning self-corrects |
| Migration breaks existing installs | Low | High | All new columns nullable; additive-only migrations |
| Loop-back boost creates filter bubble | Low | Medium | Bounded range [0.1, 5.0]; recency decay still applies; cold-start seeds remain |
| Account isolation audit reveals deep refactoring | Medium | Medium | Session 06 is dedicated; DEFAULT_ACCOUNT_ID replacement is mechanical |
| File size limits exceeded by extended modules | Low | Low | Chunker is small (~150 lines); storage/mod.rs can be split if needed |

## Decisions Made

1. **No embeddings** — keyword-based retrieval with `retrieval_boost` learning
   provides meaningful improvement over seed-only. Embedding infrastructure
   deferred to a future epic.
2. **No separate provenance table** — nullable FKs on `approval_queue` are
   simpler and sufficient at current scale.
3. **Single migration directory** — `migrations-server/` does not exist in
   this repo. All migrations go in `migrations/`.
4. **Node status lifecycle extended** — `pending` → `chunked` → `processed`
   (was `pending` → `processed`). Seed worker queries `chunked` instead of
   `pending`.
5. **Citations always included** — Response structs always include a
   `citations` field (empty array when no citations). Simpler than optional.

## Inputs for Session 02

- `charter.md` — design decisions D1, D2.
- `product-model.md` — Fragment entity definition, chunking algorithm.
- `implementation-plan.md` — Session 02 section with exact files, defaults.
- Current migration numbering: latest is `20260228000020`. Next:
  `20260309000021`.
- Key files to modify:
  - `crates/tuitbot-core/src/storage/watchtower/mod.rs` (280+ lines, has room)
  - `crates/tuitbot-core/src/automation/watchtower/mod.rs` (857 lines — near
    limit; chunker is a separate file)
  - `crates/tuitbot-core/src/automation/seed_worker.rs` (340 lines)
