# Session 01 — Priority Backlog

> Prioritized engineering backlog mapped to sessions 02–10. Priority: P0 (critical), P1 (important), P2 (medium), P3 (nice-to-have).

## Backlog

| # | Priority | Item | Severity | Impact | Session | Category |
|---|----------|------|----------|--------|---------|----------|
| 1 | **P0** | Add `unlike_tweet()` to `XApiClient` trait + `XApiHttpClient` impl | High | Completes likes CRUD — currently can like but not unlike | 04 | API coverage |
| 2 | **P0** | Add `get_followers()` / `get_following()` list endpoints | High | Enables audience analysis + relationship mapping | 04 | API coverage |
| 3 | **P0** | Add `get_user_by_id()` single user lookup | High | Resolves `author_id` from tweet data to full user profile | 04 | API coverage |
| 4 | **P0** | Add batch user lookup (`get_users_by_ids()`, `get_users_by_usernames()`) | High | Efficient bulk user resolution (up to 100 per call) | 04 | API coverage |
| 5 | **P1** | Add bookmarks CRUD (`bookmark_tweet`, `unbookmark_tweet`, `get_bookmarks`) | Medium | New API category — useful for content curation workflows | 04 | API coverage |
| 6 | **P1** | Add `get_liked_tweets()` for a user | Medium | Completes likes category — enables like history analysis | 04 | API coverage |
| 7 | **P1** | Decouple api_client_lane tools from `AppState` / DB | High | Enables standalone X client MCP without DB/LLM deps | 03 | Architecture |
| 8 | **P1** | Extract read-only X API tools into independent module | High | 7 tools need only `XApiClient` — zero DB dependency | 03 | Architecture |
| 9 | **P1** | Make policy recording injectable (trait or optional) | Medium | Unblocks mutation tool decoupling from DB | 03 | Architecture |
| 10 | **P1** | Decouple `ArcProvider` wrapper from full `AppState` | Medium | Content tools should need only `LlmProvider` + `BusinessProfile` | 03 | Architecture |
| 11 | **P2** | Define JSON schema / tool manifest for all 52 tools | Medium | Contract hardening — enables validation, documentation generation | 05 | DX |
| 12 | **P2** | Add automatic retry with exponential backoff for rate limits | Medium | Reliability — currently rate-limited calls return error immediately | 06 | Reliability |
| 13 | **P2** | Add cursor-based auto-pagination helper | Medium | DX — agents currently must manually pass `pagination_token` | 06 | DX |
| 14 | **P2** | Add media support to `quote_tweet()` | Low | Currently text-only — `media_ids` param accepted but not forwarded | 05 | API coverage |
| 15 | **P2** | Verify `quote_tweet()` test coverage (default trait returns error) | Medium | Risk: mock-based tests may not exercise `XApiHttpClient` override | 02 | Testing |
| 16 | **P2** | Add integration test harness for X API mutations | Medium | Current tests are unit-level with mocks — no end-to-end validation | 06 | Testing |
| 17 | **P3** | Add full-archive search (`GET /tweets/search/all`) | Low | Requires Academic/Pro tier — niche use case | 04 | API coverage |
| 18 | **P3** | Extract workflow_lane tools into plugin system | Medium | Long-term architecture — enables third-party tool extensions | 07 | Architecture |
| 19 | **P3** | Add WebSocket-based real-time tool execution events | Low | DX — enables live progress tracking for composite tools | 08 | DX |
| 20 | **P3** | Add tool dependency graph visualization | Low | Documentation — helps agents understand tool relationships | 09 | DX |

## Session Mapping

### Session 02 — Response Envelope Hardening
- Verify `quote_tweet()` test coverage (#15)
- Standardize response envelope across all tools
- Add error code taxonomy

### Session 03 — Decoupling & Architecture
- Decouple api_client_lane from AppState (#7, #8)
- Make policy recording injectable (#9)
- Decouple ArcProvider from AppState (#10)

### Session 04 — API Coverage Completion
- `unlike_tweet()` (#1)
- `get_followers()` / `get_following()` (#2)
- `get_user_by_id()` + batch lookups (#3, #4)
- Bookmarks CRUD (#5)
- `get_liked_tweets()` (#6)
- Full-archive search if Pro tier available (#17)

### Session 05 — Contract & Schema
- JSON schema / tool manifest (#11)
- Media support for `quote_tweet()` (#14)
- Tool parameter validation framework

### Session 06 — Reliability & DX
- Retry with exponential backoff (#12)
- Auto-pagination helper (#13)
- Integration test harness (#16)

### Session 07 — Plugin Architecture
- Extract workflow_lane to plugins (#18)
- Define plugin API surface

### Session 08 — Real-time & Events
- WebSocket tool execution events (#19)
- Progress tracking for composite tools

### Session 09 — Documentation & Observability
- Tool dependency graph visualization (#20)
- Auto-generated tool documentation from schemas

### Session 10 — Polish & Release
- Performance benchmarks
- Load testing
- Release candidate preparation
