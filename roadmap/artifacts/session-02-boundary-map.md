# Session 02 — Boundary Map

## Module Layout After Session 02

```
crates/tuitbot-mcp/src/
├── contract/              ← NEW: protocol-level types
│   ├── mod.rs             ← re-exports
│   ├── envelope.rs        ← ToolResponse, ToolError, ToolMeta (+15 tests)
│   └── error.rs           ← ProviderError enum, provider_error_to_response (+4 tests)
│
├── provider/              ← NEW: backend-agnostic trait + adapters
│   ├── mod.rs             ← SocialReadProvider trait
│   └── x_api.rs           ← XApiProvider: dyn XApiClient → SocialReadProvider
│
├── kernel/                ← NEW: provider-agnostic tool implementations
│   ├── mod.rs             ← re-exports
│   ├── read.rs            ← get_tweet, get_user_by_username, search_tweets
│   └── tests.rs           ← 9 tests with MockProvider + ErrorProvider
│
├── tools/                 ← EXISTING: workflow-level tools (TuitBot-specific)
│   ├── response.rs        ← CHANGED: now re-exports from contract::envelope
│   ├── x_actions/
│   │   ├── mod.rs         ← unchanged (error_response, not_configured_response)
│   │   ├── read.rs        ← CHANGED: 3 tools delegate to kernel, 4 unchanged
│   │   ├── write.rs       ← unchanged
│   │   ├── engage.rs      ← unchanged
│   │   ├── media.rs       ← unchanged
│   │   ├── validate.rs    ← unchanged
│   │   └── tests/         ← unchanged (all pass)
│   └── ... (14 other tool modules unchanged)
│
├── lib.rs                 ← CHANGED: added contract, provider, kernel modules
├── server.rs              ← unchanged
├── state.rs               ← unchanged
└── requests.rs            ← unchanged
```

## Dependency Flow

```
contract ←── provider ←── kernel ←── tools/x_actions/read.rs ←── server.rs
                                  ↗
                     tools/* (workflow tools still use AppState directly)
```

## Tools Routed Through Kernel (Session 02)

| Tool | Old Path | New Path |
|------|----------|----------|
| `get_tweet_by_id` | state → XApiClient::get_tweet | state → XApiProvider → kernel::read::get_tweet |
| `get_user_by_username` | state → XApiClient::get_user_by_username | state → XApiProvider → kernel::read::get_user_by_username |
| `search_tweets` | state → XApiClient::search_tweets | state → XApiProvider → kernel::read::search_tweets |

## Tools Not Yet Migrated (Future Sessions)

| Tool | Reason |
|------|--------|
| `get_user_mentions` | Requires authenticated_user_id from AppState |
| `get_user_tweets` | Straightforward; deferred for batch migration |
| `get_home_timeline` | Requires authenticated_user_id from AppState |
| `get_x_usage` | Requires DbPool (storage queries) |
| All write/engage tools | Provider write trait not yet defined |
| All workflow tools | Depend on DbPool, LlmProvider, Config |

## Provider Trait Surface (Session 02)

```rust
#[async_trait]
pub trait SocialReadProvider: Send + Sync {
    async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet, ProviderError>;
    async fn get_user_by_username(&self, username: &str) -> Result<User, ProviderError>;
    async fn search_tweets(&self, query, max_results, since_id, pagination_token) -> Result<SearchResponse, ProviderError>;
}
```

Methods to add in Session 03+: `get_user_mentions`, `get_user_tweets`, `get_home_timeline`, `get_me`.
