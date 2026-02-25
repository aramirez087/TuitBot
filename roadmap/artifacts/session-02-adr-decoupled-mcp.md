# ADR: Decoupled MCP Architecture

**Status:** Accepted
**Date:** 2026-02-25
**Session:** 02 — Target Architecture and Boundaries

## Context

The tuitbot-mcp crate (8,200 lines, 60+ tools) is a well-structured MCP server, but all tool modules live in a flat `tools/` namespace with implicit coupling to `AppState`, `DbPool`, and concrete `XApiClient`. This makes the MCP tools non-reusable outside TuitBot and prevents swapping backends (e.g., scraper vs. official API) without modifying tool code.

Session 01 identified 17 API-client-lane tools, 25 workflow-lane tools, and 10 platform-lane tools. The API-client tools should be usable by any MCP consumer, not just TuitBot.

## Decision

Introduce a **four-layer architecture** inside `tuitbot-mcp`:

### 1. Contract Layer (`contract/`)
- **Owns:** `ToolResponse`, `ToolError`, `ToolMeta` (response envelope), `ProviderError` (error taxonomy)
- **Depends on:** `serde`, `serde_json` — no TuitBot types
- **Invariant:** Any MCP server can import and use these types

### 2. Provider Layer (`provider/`)
- **Owns:** `SocialReadProvider` trait (read operations), concrete `XApiProvider` adapter
- **Depends on:** `contract::ProviderError`, `tuitbot_core::x_api` types (data structs only)
- **Invariant:** New backends implement `SocialReadProvider` without touching kernel or tool code

### 3. Kernel Layer (`kernel/`)
- **Owns:** Provider-agnostic tool implementations (`read::get_tweet`, `read::search_tweets`, etc.)
- **Depends on:** `contract` envelope, `provider::SocialReadProvider` — **never** `AppState` or `DbPool`
- **Invariant:** Kernel tools are testable with a mock provider, no database or network

### 4. Workflow Layer (existing `tools/`)
- **Owns:** TuitBot-specific tools (analytics, approval queue, content gen, composites)
- **Depends on:** `AppState`, `DbPool`, `LlmProvider`, kernel (for refactored reads)
- **Invariant:** Workflow tools may use kernel functions but kernel never reaches into workflow

## Boundary Rules

| From → To       | contract | provider | kernel | workflow |
|------------------|----------|----------|--------|----------|
| **contract**     | —        | ✗        | ✗      | ✗        |
| **provider**     | ✓        | —        | ✗      | ✗        |
| **kernel**       | ✓        | ✓        | —      | ✗        |
| **workflow**     | ✓        | ✓        | ✓      | —        |

## Consequences

**Positive:**
- Read tools no longer depend on workflow-only services (`DbPool`, `Config`, `LlmProvider`)
- Contract types are reusable by non-TuitBot consumers
- Provider trait enables future scraper backend without kernel changes
- Kernel tools are testable with zero infrastructure (no SQLite, no network)
- Clear dependency DAG prevents accidental coupling

**Negative:**
- Bridge code in `x_actions/read.rs` extracts `x_client` from `AppState` and wraps it in `XApiProvider` before calling kernel — one extra layer of indirection
- Core data types (`Tweet`, `User`, `SearchResponse`) still flow from `tuitbot_core` through all layers — full decoupling of data types deferred to a future session

## Alternatives Considered

1. **Separate crate for contract types** — cleaner boundary but premature; internal modules suffice for now
2. **Provider returns `serde_json::Value`** — fully decoupled data but loses type safety; rejected
3. **Kernel owns `AppState`** — would reduce bridge code but defeats the purpose of decoupling
