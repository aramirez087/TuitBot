---
work_package_id: "WP04"
subtasks: ["T018", "T019", "T020", "T021", "T022", "T023"]
title: "X API Client + Authentication"
phase: "Phase 0 - Foundation"
lane: "planned"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
dependencies: ["WP01"]
history:
  - timestamp: "2026-02-21T22:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP04 -- X API Client + Authentication

## IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately (right below this notice).
- **You must address all feedback** before your work is complete. Feedback items are your implementation TODO list.
- **Mark as acknowledged**: When you understand the feedback and begin addressing it, update `review_status: acknowledged` in the frontmatter.
- **Report progress**: As you address each feedback item, update the Activity Log explaining what you changed.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** -- Reviewers add detailed feedback here when work needs changes. Implementation must address every item listed below before returning for re-review.

*[This section is empty initially. Reviewers will populate it if the work is returned from review. If you see feedback here, treat each item as a must-do before completion.]*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ` ```python `, ` ```bash `

---

## Objectives & Success Criteria

- Implement all X API v2 request/response types with full Serde support.
- Build a trait-based X API client (`XApiClient`) with a reqwest-backed implementation that supports search, mentions, posting, and replying.
- Implement OAuth 2.0 PKCE authentication with two modes: manual code entry and local browser callback.
- Implement transparent token management: storage, loading, and automatic refresh before expiry.
- Implement API tier detection that adapts the agent's behavior to the user's X API plan.
- All API error states (429 rate limited, 401 auth expired, 403 forbidden) are handled with typed errors, not panics.
- No `unwrap()` in any production code path.
- All public items have `///` doc comments.

---

## Context & Constraints

### Reference Documents

- **Constitution**: `.kittify/memory/constitution.md` -- Rust 1.75+, no `unwrap()`, thiserror in library, cargo clippy/fmt/audit gates.
- **Plan**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` -- project structure, module dependency graph, architecture.
- **Spec**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md` -- FR-001 (OAuth PKCE), FR-004 (search), FR-009 (mentions), FR-020 (callback host/port), FR-025 (token storage), FR-027/FR-028 (tier detection).
- **Research**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/research.md` -- Section 1 (X API v2 endpoints, rate limits, tiers, PKCE flow, token management).
- **CLI Contract**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/contracts/cli-interface.md` -- `replyguy auth` command behavior.
- **Data Model**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` -- entity definitions for DiscoveredTweet, ReplySent.
- **Tasks**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/tasks.md` -- WP04 summary, subtask list, dependencies.

### Architectural Constraints

- All code lives in `crates/replyguy-core/src/x_api/`.
- `XApiClient` is a trait so tests can use mock implementations (wiremock or manual mocks).
- Error types use `thiserror` and must integrate with the `XApiError` variant defined in WP01's `error.rs`.
- Token storage path defaults to `~/.replyguy/tokens.json` with chmod 600.
- The client must parse X API rate limit headers on every response for downstream use by the safety module.
- OAuth 2.0 PKCE is the only supported authentication method (no API key auth, no OAuth 1.0a).

### Dependencies

- **WP01** must be complete: Cargo workspace, error types, config types (including `XApiConfig`, `AuthConfig`), and CLI skeleton.

---

## Subtasks & Detailed Guidance

### Subtask T018 -- X API Types

- **Purpose**: Define all X API v2 request and response types as Rust structs with Serde derives, providing the type foundation for every other subtask in this work package.
- **Steps**:
  1. Create `crates/replyguy-core/src/x_api/types.rs`.
  2. Define the following structs, all deriving `Debug, Clone, Serialize, Deserialize`:
     - `Tweet` -- fields: `id: String`, `text: String`, `author_id: String`, `created_at: String`, `public_metrics: PublicMetrics`, `conversation_id: Option<String>`.
     - `PublicMetrics` -- fields: `retweet_count: u64`, `reply_count: u64`, `like_count: u64`, `quote_count: u64`, `impression_count: u64`, `bookmark_count: u64`.
     - `User` -- fields: `id: String`, `username: String`, `name: String`, `public_metrics: UserMetrics`.
     - `UserMetrics` -- fields: `followers_count: u64`, `following_count: u64`, `tweet_count: u64`.
     - `SearchResponse` -- fields: `data: Vec<Tweet>`, `includes: Option<Includes>`, `meta: SearchMeta`.
     - `Includes` -- fields: `users: Vec<User>`.
     - `SearchMeta` -- fields: `newest_id: Option<String>`, `oldest_id: Option<String>`, `result_count: u32`, `next_token: Option<String>`.
     - `MentionResponse` -- same structure as `SearchResponse` (consider a type alias or identical struct).
     - `PostTweetRequest` -- fields: `text: String`, `reply: Option<ReplyTo>`.
     - `ReplyTo` -- fields: `in_reply_to_tweet_id: String`.
     - `PostTweetResponse` -- fields: `data: PostedTweet`.
     - `PostedTweet` -- fields: `id: String`, `text: String`.
  3. Use `#[serde(default)]` on optional or defaultable fields where the X API may omit them.
  4. Add `///` doc comments on every public struct and field explaining what it maps to in the X API v2 response.
- **Files**: `crates/replyguy-core/src/x_api/types.rs`
- **Parallel?**: Yes -- this subtask has no dependencies within WP04 and can start immediately.
- **Notes**:
  - `PublicMetrics` fields like `impression_count` and `bookmark_count` may not always be present depending on tweet type. Use `#[serde(default)]` for these.
  - `MentionResponse` has the same structure as `SearchResponse`. You can use a type alias (`pub type MentionResponse = SearchResponse;`) or define it as a separate struct for clarity -- either approach is acceptable.
  - Tweet IDs are strings, not integers, because X API v2 returns them as strings and some IDs exceed `i64` range.

### Subtask T019 -- X API Client Trait + Reqwest Implementation

- **Purpose**: Define the `XApiClient` trait as the abstraction boundary for all X API operations, and implement it using reqwest with proper error handling and rate limit header parsing.
- **Steps**:
  1. Create `crates/replyguy-core/src/x_api/mod.rs` with the `XApiClient` trait:
     ```rust
     #[async_trait::async_trait]
     pub trait XApiClient: Send + Sync {
         async fn search_tweets(&self, query: &str, max_results: u32, since_id: Option<&str>) -> Result<SearchResponse>;
         async fn get_mentions(&self, user_id: &str, since_id: Option<&str>) -> Result<MentionResponse>;
         async fn post_tweet(&self, text: &str) -> Result<PostedTweet>;
         async fn reply_to_tweet(&self, text: &str, in_reply_to_id: &str) -> Result<PostedTweet>;
         async fn get_tweet(&self, tweet_id: &str) -> Result<Tweet>;
         async fn get_me(&self) -> Result<User>;
     }
     ```
  2. Create `crates/replyguy-core/src/x_api/client.rs` with `XApiHttpClient`:
     - Fields: `client: reqwest::Client`, `base_url: String` (default: `https://api.x.com/2`), `access_token: Arc<RwLock<String>>`.
     - Constructor: `new(access_token: String) -> Self` -- builds reqwest client with appropriate default headers.
  3. Implement all trait methods:
     - `search_tweets`: `GET /tweets/search/recent?query={query}&max_results={max_results}&tweet.fields=public_metrics,created_at,author_id,conversation_id&expansions=author_id&user.fields=username,public_metrics`. Include `since_id` parameter if provided.
     - `get_mentions`: `GET /users/{user_id}/mentions` with same tweet.fields, expansions, and user.fields. Include `since_id` if provided.
     - `post_tweet`: `POST /tweets` with JSON body `PostTweetRequest { text, reply: None }`.
     - `reply_to_tweet`: `POST /tweets` with JSON body `PostTweetRequest { text, reply: Some(ReplyTo { in_reply_to_tweet_id }) }`.
     - `get_tweet`: `GET /tweets/{tweet_id}?tweet.fields=public_metrics,created_at,author_id,conversation_id&expansions=author_id&user.fields=username,public_metrics`.
     - `get_me`: `GET /users/me?user.fields=username,public_metrics`.
  4. On every response, parse rate limit headers:
     - `x-rate-limit-remaining` -- remaining requests in the current window.
     - `x-rate-limit-reset` -- UTC epoch second when the window resets.
     - Store these in a `RateLimitInfo` struct and log them at `tracing::debug!` level.
  5. Map HTTP errors to typed `XApiError` variants:
     - 429 -> `XApiError::RateLimited { reset_at: u64 }` (extract from `x-rate-limit-reset` header).
     - 401 -> `XApiError::AuthExpired`.
     - 403 -> `XApiError::Forbidden { message: String }` (may indicate tier restriction).
     - Other 4xx/5xx -> `XApiError::Api { status: u16, message: String }`.
     - Network/connection errors -> `XApiError::Network { source: reqwest::Error }`.
  6. Set `Authorization: Bearer {access_token}` on all requests.
- **Files**: `crates/replyguy-core/src/x_api/mod.rs`, `crates/replyguy-core/src/x_api/client.rs`
- **Parallel?**: Depends on T018 (needs types). Can proceed alongside T020/T021/T022 once T018 is done.
- **Notes**:
  - Use `Arc<RwLock<String>>` for the access token so the token manager (T022) can update it transparently after a refresh without requiring a new client instance.
  - The base URL should be configurable (stored in config) for testing with wiremock.
  - Consider adding a helper method `fn build_request(&self, method, path, query_params) -> RequestBuilder` to reduce duplication across methods.
  - All `Result` types should use the crate's error type from WP01.

### Subtask T020 -- OAuth 2.0 PKCE Manual Code Entry Flow

- **Purpose**: Implement the "manual" mode of OAuth 2.0 PKCE authentication where the user copies an authorization URL, visits it in their browser, and pastes the resulting authorization code back into the CLI.
- **Steps**:
  1. Create `crates/replyguy-core/src/x_api/auth.rs`.
  2. Implement PKCE code challenge generation:
     - Generate `code_verifier`: random string of 43-128 characters from the set `[A-Za-z0-9\-._~]`. Use `rand::thread_rng()` with `OsRng` fallback for cryptographic quality.
     - Compute `code_challenge`: `base64url_no_pad(sha256(code_verifier))`. Use the `sha2` crate for SHA-256 and `base64` crate with URL-safe no-pad encoding.
  3. Build the authorization URL:
     ```
     https://x.com/i/oauth2/authorize?
       response_type=code&
       client_id={client_id}&
       redirect_uri=http://localhost/callback&
       scope=tweet.read+tweet.write+users.read+offline.access&
       state={random_state}&
       code_challenge={code_challenge}&
       code_challenge_method=S256
     ```
  4. Generate a random `state` parameter (16+ bytes, hex-encoded) for CSRF protection.
  5. Implement the manual flow function:
     - Print the authorization URL to stdout with clear instructions.
     - Prompt the user: "Paste the authorization code from the callback URL:".
     - Read the code from stdin.
  6. Exchange the authorization code for tokens:
     - `POST https://api.x.com/2/oauth2/token` with form-encoded body:
       - `grant_type=authorization_code`
       - `client_id={client_id}`
       - `redirect_uri=http://localhost/callback`
       - `code_verifier={code_verifier}`
       - `code={authorization_code}`
     - Content-Type: `application/x-www-form-urlencoded`.
     - Parse response: `access_token`, `refresh_token`, `expires_in`, `token_type`, `scope`.
  7. Return a `Tokens` struct (defined in T022) with computed `expires_at = now + expires_in`.
- **Files**: `crates/replyguy-core/src/x_api/auth.rs`
- **Parallel?**: Can proceed in parallel with T021 (different functions in same file). Depends on WP01 config types for `client_id`.
- **Notes**:
  - The `redirect_uri` for manual mode is `http://localhost/callback` -- this is a convention for manual copy-paste flows where no server is actually listening.
  - Do not use the `oauth2` crate for this implementation. The X API PKCE flow is straightforward enough to implement with reqwest directly, and avoiding the crate gives more control over error handling.
  - Validate that the pasted code is non-empty and does not contain whitespace.
  - Log the token exchange at `tracing::info!` level (but never log the actual token values -- only log success/failure and expiration time).

### Subtask T021 -- OAuth 2.0 PKCE Local Callback Mode

- **Purpose**: Implement the "local_callback" mode where the CLI starts a temporary HTTP server, opens the user's browser to the authorization URL, and automatically captures the callback with the authorization code.
- **Steps**:
  1. Add to `crates/replyguy-core/src/x_api/auth.rs`.
  2. Implement a temporary HTTP server:
     - Bind to `{callback_host}:{callback_port}` (from config, defaults: `127.0.0.1:8080`).
     - Use `tokio::net::TcpListener` and manually parse the HTTP request (or use a minimal HTTP parsing approach). Avoid pulling in a full web framework.
     - Listen for a single `GET /callback?code=XXX&state=YYY` request.
  3. Build the authorization URL (same as T020 but with `redirect_uri=http://{callback_host}:{callback_port}/callback`).
  4. Open the browser:
     - Use `open::that(url)` (the `open` crate) to open the authorization URL in the default browser.
     - If browser open fails, fall back to printing the URL with instructions to open manually.
  5. Handle the callback:
     - Parse the query string from the GET request to extract `code` and `state` parameters.
     - Validate that `state` matches the state generated in step 3.
     - If state mismatch, respond with an error HTML page and return an error.
     - If valid, respond with a success HTML page: "Authentication successful! You can close this tab."
     - Shut down the temporary server.
  6. Exchange the code for tokens using the same token exchange function from T020 (share the exchange logic).
  7. Add a timeout (e.g., 120 seconds) for the callback. If no callback arrives, shut down the server and return an error.
- **Files**: `crates/replyguy-core/src/x_api/auth.rs`
- **Parallel?**: Yes -- can proceed in parallel with T020. Both are separate functions that share only the code exchange logic.
- **Notes**:
  - The callback host and port must come from `auth.callback_host` and `auth.callback_port` in the config (FR-020).
  - The HTTP server only needs to handle one request. After receiving the callback, it should shut down immediately.
  - Keep the HTTP parsing minimal. The request will be `GET /callback?code=XXX&state=YYY HTTP/1.1\r\n...`. Parse the first line, extract the query string.
  - The success HTML response should be a simple, self-contained HTML page with no external dependencies.
  - Consider factoring the common PKCE logic (verifier generation, challenge computation, URL building, token exchange) into shared helper functions used by both T020 and T021.

### Subtask T022 -- Token Management

- **Purpose**: Implement persistent token storage, loading, and automatic refresh so that API calls transparently use valid tokens without requiring manual re-authentication.
- **Steps**:
  1. Define the `Tokens` struct in `crates/replyguy-core/src/x_api/auth.rs`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize)]
     pub struct Tokens {
         pub access_token: String,
         pub refresh_token: String,
         pub expires_at: chrono::DateTime<chrono::Utc>,
         pub scopes: Vec<String>,
     }
     ```
  2. Implement `TokenManager` struct:
     - Fields: `tokens: Arc<RwLock<Tokens>>`, `client_id: String`, `http_client: reqwest::Client`.
     - `save_tokens(tokens: &Tokens, path: &Path) -> Result<()>`:
       - Serialize tokens to JSON.
       - Write to the specified path (default `~/.replyguy/tokens.json`).
       - Create parent directories if they do not exist (`std::fs::create_dir_all`).
       - Set file permissions to 600 (`std::os::unix::fs::PermissionsExt` on Unix; on Windows, skip permission setting with a logged warning).
     - `load_tokens(path: &Path) -> Result<Option<Tokens>>`:
       - Return `Ok(None)` if the file does not exist.
       - Deserialize from JSON. Return an error if the file exists but is malformed.
     - `refresh_if_needed(&self) -> Result<()>`:
       - Read current tokens (acquire read lock).
       - If `expires_at` is within 5 minutes of now, perform a refresh.
       - `POST https://api.x.com/2/oauth2/token` with form body: `grant_type=refresh_token`, `refresh_token={refresh_token}`, `client_id={client_id}`.
       - Parse new `access_token`, `refresh_token`, `expires_in`.
       - Compute new `expires_at`.
       - Acquire write lock and update the stored tokens.
       - Save updated tokens to disk.
     - `get_access_token(&self) -> Result<String>`:
       - Call `refresh_if_needed()` first.
       - Return the current `access_token`.
  3. Integrate with `XApiHttpClient`:
     - The client should call `token_manager.get_access_token()` before each request to ensure the token is fresh.
     - Alternatively, the client holds an `Arc<RwLock<String>>` for the access token, and the token manager updates it in place after refresh.
- **Files**: `crates/replyguy-core/src/x_api/auth.rs`
- **Parallel?**: Depends on T020 or T021 for the `Tokens` struct context. Can start once the struct is defined.
- **Notes**:
  - Token refresh is a critical section. Use `RwLock` (not `Mutex`) so multiple concurrent reads of the access token do not block each other. Only refresh acquires a write lock.
  - If the refresh token itself fails (e.g., revoked), return `XApiError::AuthExpired` so the caller can prompt re-authentication.
  - The 5-minute refresh window is deliberate: access tokens expire every 2 hours, and refreshing 5 minutes early prevents edge cases where a token expires mid-request.
  - On Windows, `std::os::unix::fs::PermissionsExt` is not available. Use `#[cfg(unix)]` to conditionally set permissions and log a warning on non-Unix platforms.

### Subtask T023 -- API Tier Detection

- **Purpose**: Detect the user's X API tier at startup so the agent can adapt its behavior -- enabling or disabling loops based on available capabilities.
- **Steps**:
  1. Create `crates/replyguy-core/src/x_api/tier.rs`.
  2. Define the `ApiTier` enum:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub enum ApiTier {
         Free,
         Basic,
         Pro,
     }
     ```
  3. Implement `detect_tier(client: &dyn XApiClient) -> ApiTier`:
     - Attempt `client.search_tweets("test", 1, None)`.
     - If the result is `Err(XApiError::Forbidden { .. })` or a specific X API error code indicating the search endpoint is not available -> return `ApiTier::Free`.
     - If the result is `Ok(_)` or any error other than forbidden -> return `ApiTier::Basic` (treat Pro same as Basic for our purposes).
     - If the result is `Err(XApiError::RateLimited { .. })` -> return `ApiTier::Basic` (rate limited implies the endpoint exists).
     - If the result is `Err(XApiError::AuthExpired)` -> propagate the error (do not infer tier from auth failure).
  4. Implement `ApiTier::capabilities(&self) -> TierCapabilities`:
     ```rust
     pub struct TierCapabilities {
         pub search_available: bool,
         pub mentions_available: bool,
         pub posting_available: bool,
         pub discovery_loop_enabled: bool,
     }
     ```
     - Free: `search_available: false, mentions_available: false, posting_available: true, discovery_loop_enabled: false`.
     - Basic/Pro: all `true`.
  5. Log the detected tier and capabilities at `tracing::info!` level, including which loops will be enabled/disabled (FR-028).
- **Files**: `crates/replyguy-core/src/x_api/tier.rs`
- **Parallel?**: Depends on T019 (needs `XApiClient` trait). Can proceed once T019 is complete.
- **Notes**:
  - The detection query "test" is intentionally minimal to avoid wasting rate limit quota.
  - Free tier does not have access to the search endpoint at all. Basic tier ($200/mo) has access to search with rate limits.
  - According to research.md, Free tier also does not have mentions endpoint access. Update `TierCapabilities` accordingly.
  - If detection fails due to network issues (not auth, not forbidden), log a warning and default to `ApiTier::Free` as the safe fallback.
  - Consider caching the detected tier for the session lifetime -- no need to re-detect during a run.

---

## Test Strategy

- **Unit tests** for all type deserialization: create JSON fixtures matching X API v2 responses and verify they deserialize correctly into Rust types.
- **Unit tests** for PKCE: verify `code_verifier` length and character set, verify `code_challenge` matches expected SHA-256/base64url output for known inputs.
- **Integration tests** with `wiremock` for the HTTP client: mock search, mentions, post, and error responses. Verify correct URL construction, headers, query parameters, and error mapping.
- **Integration tests** for token refresh: mock the token endpoint, verify refresh triggers within the 5-minute window, verify write-lock serialization.
- **Integration tests** for tier detection: mock search returning 200 (Basic), 403 (Free), and 429 (Basic) to verify detection logic.
- **Test commands**:
  ```bash
  cargo test -p replyguy-core --lib x_api
  cargo test -p replyguy-core --test integration -- x_api
  ```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| X API rate limits during testing | High | Medium | Use wiremock for all HTTP tests. Never hit the live X API in CI. |
| Token refresh race conditions | Medium | High | Use `RwLock` around `TokenManager`. Write lock only during refresh. Multiple readers can proceed concurrently. |
| PKCE code_verifier entropy | Low | High | Use `rand::thread_rng()` with `OsRng` fallback for cryptographic quality randomness. |
| X API response format changes | Low | Medium | Pin type definitions to X API v2 docs. Use `#[serde(default)]` on optional fields to tolerate additions. |
| Local callback port conflict | Medium | Low | Make host and port configurable. Log a clear error if bind fails, suggesting the user change the port. |
| OAuth state parameter mismatch | Low | Medium | Generate cryptographically random state, validate on callback. Return clear error on mismatch. |

---

## Review Guidance

- Verify all API response types match the [X API v2 documentation](https://developer.x.com/en/docs/x-api/tweets/search/api-reference/get-tweets-search-recent) field names and types.
- Verify the OAuth PKCE flow works with both manual and local_callback modes. Confirm the code_verifier meets RFC 7636 requirements (43-128 chars, unreserved characters only).
- Verify tier detection handles all edge cases: success (Basic), 403 (Free), 429 (Basic), network error (fallback to Free), auth error (propagate).
- Verify token refresh uses the 5-minute pre-expiry window correctly.
- Verify no `unwrap()` calls in any production code path.
- Verify all public types and functions have `///` doc comments.
- Verify rate limit headers (`x-rate-limit-remaining`, `x-rate-limit-reset`) are parsed on every API response.
- Verify token file permissions are set to 600 on Unix and gracefully skipped on Windows.

---

## Activity Log

> **CRITICAL**: Activity log entries MUST be in chronological order (oldest first, newest last).

### How to Add Activity Log Entries

**When adding an entry**:
1. Scroll to the bottom of this file (Activity Log section below "Valid lanes")
2. **APPEND the new entry at the END** (do NOT prepend or insert in middle)
3. Use exact format: `- YYYY-MM-DDTHH:MM:SSZ -- agent_id -- lane=<lane> -- <action>`
4. Timestamp MUST be current time in UTC (check with `date -u "+%Y-%m-%dT%H:%M:%SZ"`)
5. Lane MUST match the frontmatter `lane:` field exactly
6. Agent ID should identify who made the change (claude-sonnet-4-5, codex, etc.)

**Format**:
```
- YYYY-MM-DDTHH:MM:SSZ -- <agent_id> -- lane=<lane> -- <brief action description>
```

**Example (correct chronological order)**:
```
- 2026-01-12T10:00:00Z -- system -- lane=planned -- Prompt created
- 2026-01-12T10:30:00Z -- claude -- lane=doing -- Started implementation
- 2026-01-12T11:00:00Z -- codex -- lane=for_review -- Implementation complete, ready for review
- 2026-01-12T11:30:00Z -- claude -- lane=done -- Review passed, all tests passing
```

**Common mistakes (DO NOT DO THIS)**:
- Adding new entry at the top (breaks chronological order)
- Using future timestamps (causes acceptance validation to fail)
- Lane mismatch: frontmatter says `lane: "done"` but log entry says `lane=doing`
- Inserting in middle instead of appending to end

**Why this matters**: The acceptance system reads the LAST activity log entry as the current state. If entries are out of order, acceptance will fail even when the work is complete.

**Initial entry**:
- 2026-02-21T22:00:00Z -- system -- lane=planned -- Prompt created.

---

### Updating Lane Status

To change a work package's lane, either:

1. **Edit directly**: Change the `lane:` field in frontmatter AND append activity log entry (at the end)
2. **Use CLI**: `spec-kitty agent tasks move-task <WPID> --to <lane> --note "message"` (recommended)

The CLI command updates both frontmatter and activity log automatically.

**Valid lanes**: `planned`, `doing`, `for_review`, `done`

### Optional Phase Subdirectories

For large features, organize prompts under `tasks/` to keep bundles grouped while maintaining lexical ordering.
