---
work_package_id: "WP05"
subtasks: ["T024", "T025", "T026", "T027", "T028"]
title: "LLM Integration + Content Generator"
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

# Work Package Prompt: WP05 -- LLM Integration + Content Generator

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

- Define a clean `LlmProvider` trait abstraction with typed errors, token usage tracking, and health checking.
- Implement an OpenAI-compatible provider that works for both OpenAI (cloud) and Ollama (local) -- same HTTP implementation, different base URL and API key.
- Implement an Anthropic native provider using the Messages API with its distinct headers and request format.
- Build a provider factory that instantiates the correct provider from configuration.
- Build a `ContentGenerator` that uses any `LlmProvider` to produce replies (3 sentences max, 280 chars), standalone tweets (280 chars), and threads (5-8 tweets, 280 chars each).
- All generated content is validated for length and format before being returned, with retry logic for invalid output.
- No `unwrap()` in any production code path.
- All public items have `///` doc comments.

---

## Context & Constraints

### Reference Documents

- **Constitution**: `.kittify/memory/constitution.md` -- Rust 1.75+, no `unwrap()`, thiserror in library, cargo clippy/fmt/audit gates.
- **Plan**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` -- project structure, module dependency graph (`llm` depends on config and error).
- **Spec**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md` -- FR-007 (human-sounding replies, 3 sentences), FR-008 (multi-provider LLM), FR-010 (educational tweets), FR-011 (threads of 5-8 tweets).
- **Research**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/research.md` -- Section 2 (LLM provider integration, provider implementation map, trait design decisions).
- **CLI Contract**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/contracts/cli-interface.md` -- LLM config section, provider selection.
- **Data Model**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` -- `BusinessProfile` entity used by ContentGenerator.

### Architectural Constraints

- LLM provider code lives in `crates/replyguy-core/src/llm/`.
- Content generator code lives in `crates/replyguy-core/src/content/generator.rs`.
- The `LlmProvider` trait must be object-safe (`Box<dyn LlmProvider>`) so the factory can return different implementations.
- Only two HTTP implementations are needed: OpenAI-compatible (covers OpenAI + Ollama) and Anthropic native. This is a deliberate decision from research.md.
- Error types use `thiserror` and must integrate with the `LlmError` variant defined in WP01's `error.rs`.
- The `ContentGenerator` depends on `BusinessProfile` from the config types (WP01) to inject product context into prompts.

### Dependencies

- **WP01** must be complete: Cargo workspace, error types, config types (including `LlmConfig`, `BusinessProfile`).

---

## Subtasks & Detailed Guidance

### Subtask T024 -- LlmProvider Trait + Types

- **Purpose**: Define the core LLM abstraction types and trait that all providers implement, establishing a consistent interface for content generation regardless of the backing provider.
- **Steps**:
  1. Create `crates/replyguy-core/src/llm/mod.rs`.
  2. Define the `LlmError` enum (if not already covered by WP01's error types; otherwise, use or extend that):
     ```rust
     #[derive(Debug, thiserror::Error)]
     pub enum LlmError {
         #[error("HTTP request failed: {0}")]
         Request(#[from] reqwest::Error),

         #[error("API error (status {status}): {message}")]
         Api { status: u16, message: String },

         #[error("Rate limited, retry after {retry_after_secs} seconds")]
         RateLimited { retry_after_secs: u64 },

         #[error("Failed to parse LLM response: {0}")]
         Parse(String),

         #[error("LLM provider not configured")]
         NotConfigured,
     }
     ```
  3. Define supporting types:
     ```rust
     #[derive(Debug, Clone)]
     pub struct TokenUsage {
         pub input_tokens: u32,
         pub output_tokens: u32,
     }

     #[derive(Debug, Clone)]
     pub struct LlmResponse {
         pub text: String,
         pub usage: TokenUsage,
         pub model: String,
     }

     #[derive(Debug, Clone)]
     pub struct GenerationParams {
         pub max_tokens: u32,
         pub temperature: f32,
         pub system_prompt: Option<String>,
     }

     impl Default for GenerationParams {
         fn default() -> Self {
             Self {
                 max_tokens: 512,
                 temperature: 0.7,
                 system_prompt: None,
             }
         }
     }
     ```
  4. Define the `LlmProvider` trait:
     ```rust
     #[async_trait::async_trait]
     pub trait LlmProvider: Send + Sync {
         /// Returns the display name of this provider (e.g., "openai", "anthropic", "ollama").
         fn name(&self) -> &str;

         /// Send a completion request to the LLM.
         async fn complete(
             &self,
             system: &str,
             user_message: &str,
             params: &GenerationParams,
         ) -> Result<LlmResponse, LlmError>;

         /// Check if the provider is reachable and configured correctly.
         async fn health_check(&self) -> Result<(), LlmError>;
     }
     ```
  5. Re-export types and trait from `mod.rs`.
- **Files**: `crates/replyguy-core/src/llm/mod.rs`
- **Parallel?**: Yes -- this subtask has no dependencies within WP05 and can start immediately.
- **Notes**:
  - The trait must be object-safe. All methods take `&self` and return concrete types (no associated types that would break `dyn`).
  - `GenerationParams` uses `Default` so callers can do `GenerationParams { temperature: 0.9, ..Default::default() }`.
  - `TokenUsage` tracks input/output tokens for cost monitoring and logging. Not all providers return this (Ollama may not), so set defaults of 0 when unavailable.
  - The `system` parameter in `complete()` is separate from `GenerationParams.system_prompt` intentionally. The `GenerationParams.system_prompt` is an override; the `system` param is the primary system prompt passed by the caller. If `GenerationParams.system_prompt` is `Some`, use it instead of the `system` parameter.

### Subtask T025 -- OpenAI-Compatible Provider

- **Purpose**: Implement a single provider that works for both OpenAI (cloud API) and Ollama (local) since they share the same chat completions request/response format.
- **Steps**:
  1. Create `crates/replyguy-core/src/llm/openai_compat.rs`.
  2. Define the `OpenAiCompatProvider` struct:
     ```rust
     pub struct OpenAiCompatProvider {
         client: reqwest::Client,
         base_url: String,
         api_key: String,
         model: String,
         provider_name: String,
     }
     ```
  3. Constructor: `new(base_url, api_key, model, provider_name) -> Self`. Build the reqwest client.
  4. Implement `LlmProvider`:
     - `name()`: return `&self.provider_name`.
     - `complete()`: `POST {base_url}/chat/completions` with JSON body:
       ```json
       {
         "model": "<model>",
         "messages": [
           {"role": "system", "content": "<system_prompt>"},
           {"role": "user", "content": "<user_message>"}
         ],
         "max_tokens": <max_tokens>,
         "temperature": <temperature>
       }
       ```
       - Set `Authorization: Bearer {api_key}` header.
       - Parse response: `response.choices[0].message.content` for the text.
       - Parse `response.usage.prompt_tokens` and `response.usage.completion_tokens` for `TokenUsage`. Default to 0 if absent.
       - Parse `response.model` for the model name.
     - Handle error responses:
       - 429: extract `retry-after` header if present (as seconds). Return `LlmError::RateLimited`.
       - 401: return `LlmError::Api` with status and message indicating invalid API key.
       - 500+: return `LlmError::Api` with status and response body.
       - Network errors: return `LlmError::Request`.
     - `health_check()`: call `complete("You are a test assistant.", "Say OK", &GenerationParams { max_tokens: 10, ..Default::default() })`. If it returns successfully, the provider is healthy. Discard the response.
  5. Define internal request/response types for Serde:
     - `ChatCompletionRequest`, `ChatMessage`, `ChatCompletionResponse`, `Choice`, `Usage`.
     - These are private types used only for serialization/deserialization within this module.
- **Files**: `crates/replyguy-core/src/llm/openai_compat.rs`
- **Parallel?**: Depends on T024 (needs the trait). Can proceed in parallel with T026 once T024 is done.
- **Notes**:
  - For OpenAI: `base_url = "https://api.openai.com/v1"`, `api_key = <real_key>`, `provider_name = "openai"`.
  - For Ollama: `base_url = "http://localhost:11434/v1"`, `api_key = "ollama"` (Ollama ignores the key but the endpoint expects the header), `provider_name = "ollama"`.
  - The `messages` array always includes a system message followed by a user message. If the system prompt is empty, still include it with empty content (OpenAI/Ollama handle this fine).
  - Ollama's OpenAI-compatible endpoint may not return `usage` in the response. Handle this by defaulting `TokenUsage` fields to 0 if the `usage` field is null or missing.
  - Keep the response types flexible with `#[serde(default)]` to tolerate field additions or omissions across providers.

### Subtask T026 -- Anthropic Native Provider

- **Purpose**: Implement the Anthropic provider using the native Messages API, which has a different request/response format and authentication mechanism than the OpenAI-compatible endpoint.
- **Steps**:
  1. Create `crates/replyguy-core/src/llm/anthropic.rs`.
  2. Define the `AnthropicProvider` struct:
     ```rust
     pub struct AnthropicProvider {
         client: reqwest::Client,
         api_key: String,
         model: String,
     }
     ```
  3. Constructor: `new(api_key, model) -> Self`. Build the reqwest client.
  4. Implement `LlmProvider`:
     - `name()`: return `"anthropic"`.
     - `complete()`: `POST https://api.anthropic.com/v1/messages` with:
       - Headers:
         - `x-api-key: {api_key}`
         - `anthropic-version: 2023-06-01`
         - `content-type: application/json`
       - JSON body:
         ```json
         {
           "model": "<model>",
           "max_tokens": <max_tokens>,
           "system": "<system_prompt>",
           "messages": [
             {"role": "user", "content": "<user_message>"}
           ],
           "temperature": <temperature>
         }
         ```
       - Note: `max_tokens` is REQUIRED by the Anthropic API (unlike OpenAI where it is optional). Always include it.
       - Note: `system` is a top-level field, not a message in the `messages` array.
       - Parse response: `response.content[0].text` for the text.
       - Parse `response.usage.input_tokens` and `response.usage.output_tokens` for `TokenUsage`.
       - Parse `response.model` for the model name.
     - Handle error responses:
       - 429: Anthropic returns `{"error": {"type": "rate_limit_error", "message": "..."}}`. Extract `retry-after` header if present. Return `LlmError::RateLimited`.
       - 401: invalid API key. Return `LlmError::Api`.
       - 529: Anthropic "overloaded" status. Treat as rate limited with a default retry of 30 seconds.
       - Other errors: return `LlmError::Api` with status and parsed error message.
     - `health_check()`: same pattern as OpenAI -- simple completion with "Say OK".
  5. Define internal Serde types for the Anthropic request/response format:
     - `AnthropicRequest`, `AnthropicMessage`, `AnthropicResponse`, `ContentBlock`, `AnthropicUsage`, `AnthropicError`.
- **Files**: `crates/replyguy-core/src/llm/anthropic.rs`
- **Parallel?**: Yes -- can proceed in parallel with T025 once T024 is done.
- **Notes**:
  - The Anthropic Messages API has key differences from OpenAI:
    - System prompt is a top-level `"system"` field, NOT a message with role "system".
    - `max_tokens` is required, not optional.
    - Response format uses `content` array of blocks, not `choices` array.
    - Error format is `{"error": {"type": "...", "message": "..."}}`, not a simple status message.
  - The `anthropic-version` header must be `2023-06-01`. This is the stable version string.
  - Status 529 ("overloaded") is Anthropic-specific and should be treated as a transient error with retry.
  - If the system prompt is empty, omit the `"system"` field entirely from the request body (Anthropic accepts requests without it).

### Subtask T027 -- Provider Factory

- **Purpose**: Create a factory function that reads the LLM configuration and returns the correct provider instance, abstracting away provider-specific construction details from the rest of the codebase.
- **Steps**:
  1. Create `crates/replyguy-core/src/llm/factory.rs`.
  2. Implement the factory function:
     ```rust
     pub fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
         match config.provider.as_str() {
             "openai" => {
                 let base_url = if config.base_url.is_empty() {
                     "https://api.openai.com/v1".to_string()
                 } else {
                     config.base_url.clone()
                 };
                 let model = if config.model.is_empty() {
                     "gpt-4o-mini".to_string()
                 } else {
                     config.model.clone()
                 };
                 Ok(Box::new(OpenAiCompatProvider::new(
                     base_url,
                     config.api_key.clone(),
                     model,
                     "openai".to_string(),
                 )))
             }
             "ollama" => {
                 let base_url = if config.base_url.is_empty() {
                     "http://localhost:11434/v1".to_string()
                 } else {
                     config.base_url.clone()
                 };
                 let model = if config.model.is_empty() {
                     "llama3.1".to_string()
                 } else {
                     config.model.clone()
                 };
                 Ok(Box::new(OpenAiCompatProvider::new(
                     base_url,
                     "ollama".to_string(),
                     model,
                     "ollama".to_string(),
                 )))
             }
             "anthropic" => {
                 let model = if config.model.is_empty() {
                     "claude-sonnet-4-5-20250514".to_string()
                 } else {
                     config.model.clone()
                 };
                 Ok(Box::new(AnthropicProvider::new(
                     config.api_key.clone(),
                     model,
                 )))
             }
             other => Err(LlmError::NotConfigured),
         }
     }
     ```
  3. Validate essential configuration:
     - For "openai" and "anthropic": `api_key` must be non-empty. Return `LlmError::NotConfigured` if missing.
     - For "ollama": `api_key` is ignored (set to "ollama" internally).
     - For unknown provider names: return `LlmError::NotConfigured` with a message listing valid options.
  4. Support custom `base_url` override for all providers. If `config.base_url` is non-empty, use it instead of the default. This enables pointing OpenAI-compatible clients at custom endpoints (e.g., Azure OpenAI, LiteLLM proxy).
- **Files**: `crates/replyguy-core/src/llm/factory.rs`
- **Parallel?**: Depends on T025 and T026 (needs concrete provider types to construct).
- **Notes**:
  - Default models: `gpt-4o-mini` for OpenAI, `claude-sonnet-4-5-20250514` for Anthropic, `llama3.1` for Ollama. These are reasonable defaults that balance quality and cost.
  - The factory returns `Box<dyn LlmProvider>` so callers are decoupled from the concrete type.
  - Log the constructed provider at `tracing::info!` level: provider name, model, base_url (but NOT the API key).
  - The `base_url` override is particularly useful for testing with wiremock.

### Subtask T028 -- ContentGenerator

- **Purpose**: Build the high-level content generation layer that combines an LLM provider with business context to produce replies, tweets, and threads that meet all format and length requirements.
- **Steps**:
  1. Create `crates/replyguy-core/src/content/generator.rs` (and `crates/replyguy-core/src/content/mod.rs` if it does not exist).
  2. Define the `ContentGenerator` struct:
     ```rust
     pub struct ContentGenerator {
         provider: Box<dyn LlmProvider>,
         business: BusinessProfile,
     }
     ```
  3. Constructor: `new(provider: Box<dyn LlmProvider>, business: BusinessProfile) -> Self`.
  4. Implement `generate_reply(tweet_text: &str, tweet_author: &str) -> Result<String>`:
     - System prompt (build dynamically from `self.business`):
       ```
       You are a helpful community member who uses {product_name} ({product_description}).
       Your target audience is: {target_audience}.
       Product URL: {product_url}

       Rules:
       - Write a reply to the tweet below.
       - Maximum 3 sentences.
       - Be conversational and helpful, not salesy.
       - Only mention {product_name} if it is genuinely relevant to the tweet's topic.
       - Do not use hashtags.
       - Do not use emojis excessively.
       - Sound like a real person, not a bot.
       ```
     - User message: `"Tweet by @{tweet_author}: {tweet_text}"`.
     - Use `GenerationParams { max_tokens: 200, temperature: 0.8, ..Default::default() }`.
     - Validate the output: must be <= 280 characters. If too long, retry once with an appended instruction: `"Important: Your reply MUST be under 280 characters. Be more concise."`. If still too long, truncate at the last complete sentence boundary that fits within 280 characters.
     - Trim whitespace from the output.
     - Return the validated reply text.
  5. Implement `generate_tweet(topic: &str) -> Result<String>`:
     - System prompt:
       ```
       You are {product_name}'s social media voice. {product_description}.
       Your audience: {target_audience}.

       Rules:
       - Write a single educational tweet about the topic below.
       - Maximum 280 characters.
       - Be informative and engaging.
       - Do not use hashtags.
       - Do not mention {product_name} directly unless it is central to the topic.
       ```
     - User message: `"Write a tweet about: {topic}"`.
     - Use `GenerationParams { max_tokens: 150, temperature: 0.8, ..Default::default() }`.
     - Validate: must be <= 280 characters. Same retry/truncation strategy as `generate_reply`.
  6. Implement `generate_thread(topic: &str) -> Result<Vec<String>>`:
     - System prompt:
       ```
       You are {product_name}'s social media voice. {product_description}.
       Your audience: {target_audience}.

       Rules:
       - Write an educational thread of 5 to 8 tweets about the topic below.
       - Separate each tweet with a line containing only "---".
       - Each tweet must be under 280 characters.
       - The first tweet should hook the reader.
       - The last tweet should include a call to action or summary.
       - Be informative, not promotional.
       - Do not use hashtags.
       ```
     - User message: `"Write a thread about: {topic}"`.
     - Use `GenerationParams { max_tokens: 1500, temperature: 0.7, ..Default::default() }`.
     - Parse the output: split by `"---"`, trim each part, filter out empty strings.
     - Validate:
       - Count must be 5-8 tweets. If outside range, retry (max 2 retries).
       - Each tweet must be <= 280 characters. If any tweet exceeds, retry.
     - If all retries fail, return an error with a descriptive message.
  7. Add a private helper `fn validate_length(text: &str, max_chars: usize) -> bool`.
  8. Add a private helper `fn truncate_at_sentence(text: &str, max_chars: usize) -> String` that finds the last period, exclamation mark, or question mark within the limit and truncates there.
- **Files**: `crates/replyguy-core/src/content/generator.rs`, `crates/replyguy-core/src/content/mod.rs`
- **Parallel?**: Depends on T024 (LlmProvider trait) and WP01 (BusinessProfile config type).
- **Notes**:
  - The system prompts are critical to output quality. They should be tunable but these defaults provide a solid starting point.
  - The 280-character limit is X's current tweet limit. This is validated before returning, never after posting.
  - Thread parsing is the most fragile part. LLMs may not always use the exact `"---"` delimiter. Consider also splitting on lines that look like numbered items (`"1/8"`, `"1."`) as a fallback parser.
  - The retry logic for threads (max 2 retries) prevents infinite loops when the LLM consistently produces malformed output.
  - `generate_reply` uses a slightly higher temperature (0.8) for more natural-sounding replies. `generate_thread` uses 0.7 for more coherent multi-tweet output.
  - The `BusinessProfile` struct must be available from WP01's config types. If it uses different field names, adapt the prompts accordingly.

---

## Test Strategy

- **Unit tests** for `GenerationParams::default()`: verify default values (512 tokens, 0.7 temperature).
- **Unit tests** for `validate_length` and `truncate_at_sentence` helpers.
- **Unit tests** for thread parsing: test splitting by `"---"`, edge cases (extra whitespace, empty sections, no delimiter).
- **Mock provider tests**: Create a `MockLlmProvider` that returns canned responses. Test:
  - `generate_reply` with response under 280 chars (success).
  - `generate_reply` with response over 280 chars (retry, then truncate).
  - `generate_tweet` with valid output.
  - `generate_thread` with valid 6-tweet thread.
  - `generate_thread` with malformed output (wrong count, missing delimiters) to verify retry logic.
- **Integration tests** with wiremock for OpenAI-compat and Anthropic providers: mock the HTTP endpoints, verify correct request format, headers, and response parsing.
- **Test commands**:
  ```bash
  cargo test -p replyguy-core --lib llm
  cargo test -p replyguy-core --lib content
  cargo test -p replyguy-core --test integration -- llm
  ```

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| LLM output format unpredictable | High | Medium | Validate all output (length, format) before returning. Retry with stricter prompts. Truncate as last resort. |
| Anthropic API differences from OpenAI | Medium | Medium | Separate implementation (T026) with Anthropic-specific request/response types. Test both providers independently. |
| Ollama may not be running locally | Medium | Low | `health_check()` detects unreachable Ollama. `replyguy test` reports the issue clearly before `run`. |
| Thread delimiter not consistent | High | Medium | Primary parser splits on `"---"`. Fallback parser detects numbered items. Retry on parse failure. |
| API key exposed in logs | Low | High | Never log API keys. Log provider name and model only. The factory logs construction without the key. |
| Token usage not returned by Ollama | Medium | Low | Default `TokenUsage` fields to 0 when `usage` is null or missing in the response. |

---

## Review Guidance

- Verify the `LlmProvider` trait is object-safe (can be used as `Box<dyn LlmProvider>`).
- Verify both OpenAI-compat and Anthropic providers handle all documented error codes (429, 401, 500/529).
- Verify the Anthropic request format matches the Messages API: `system` as top-level field, `max_tokens` required, `content` array response.
- Verify the factory correctly maps provider names to implementations with sensible defaults.
- Verify `ContentGenerator` system prompts incorporate all `BusinessProfile` fields.
- Verify all generated content is validated for 280-character limit before being returned.
- Verify thread parsing handles edge cases (extra whitespace, missing delimiters, wrong count).
- Verify retry logic has bounded retries (1 for replies/tweets, 2 for threads) to prevent infinite loops.
- Verify no API keys are logged anywhere in the codebase.
- Verify no `unwrap()` calls in any production code path.
- Verify all public types and functions have `///` doc comments.

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
