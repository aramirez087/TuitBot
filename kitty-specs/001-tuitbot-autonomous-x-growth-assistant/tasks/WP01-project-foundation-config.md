---
work_package_id: WP01
title: Project Foundation + Configuration
lane: "done"
dependencies: []
base_branch: main
base_commit: 1c81600044b3605a8b3d395b41f5b5948763e94c
created_at: '2026-02-22T00:22:10.910094+00:00'
subtasks:
- T001
- T002
- T003
- T004
- T005
- T006
phase: Phase 0 - Foundation
assignee: ''
agent: "claude-opus"
shell_pid: "35288"
review_status: "approved"
reviewed_by: "Alexander Ramirez"
history:
- timestamp: '2026-02-21T22:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# Work Package Prompt: WP01 -- Project Foundation + Configuration

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

- **Cargo workspace compiles**: `cargo build --workspace` succeeds with zero warnings under `cargo clippy -- -D warnings`.
- **CLI help works**: `cargo run --bin replyguy -- --help` prints usage matching the contract in `contracts/cli-interface.md`.
- **Config loads and validates**: A valid `config.toml` is loaded, env var overrides apply, and validation catches all invalid configurations (returning a complete list of errors, not just the first).
- **Error types defined**: All five error enums (`ConfigError`, `XApiError`, `LlmError`, `StorageError`, `ScoringError`) compile with `thiserror` derives and cover every variant specified in the spec.
- **Subcommand stubs**: All 8 subcommands (`run`, `auth`, `test`, `discover`, `mentions`, `post`, `thread`, `score`) parse correctly and print "not implemented yet" when invoked.
- **Version flag**: `cargo run --bin replyguy -- --version` prints the version from `Cargo.toml`.

## Context & Constraints

- **Spec**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/spec.md` -- FR-002, FR-003, FR-018, FR-019, FR-021, FR-023.
- **Plan**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/plan.md` -- Project Structure, Architecture, Module Dependency Graph.
- **CLI contract**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/contracts/cli-interface.md` -- all commands, global options, config structure, env var overrides.
- **Data model**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/data-model.md` -- BusinessProfile and ScoringWeights config entities.
- **Quickstart**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/quickstart.md` -- repository structure, key dependencies table.
- **Research**: `kitty-specs/001-replyguy-autonomous-x-growth-assistant/research.md` -- Section 5 (Error Handling), Section 6 (Configuration Layering), Section 7 (Crate Structure).
- **Constitution**: `.kittify/memory/constitution.md` -- No `unwrap()` in production code, `cargo clippy -D warnings`, `cargo fmt --check`, public APIs have `///` doc comments.
- **Rust edition**: 2021, minimum Rust 1.75+.
- **No dependencies on other work packages**: WP01 is the starting package.

## Subtasks & Detailed Guidance

### Subtask T001 -- Create Cargo Workspace

- **Purpose**: Establish the two-crate workspace structure that all subsequent work packages depend on. `replyguy-core` holds all business logic as a library; `replyguy-cli` is a thin binary that depends on it.
- **Steps**:
  1. Create the workspace root `Cargo.toml` at the repository root:
     ```toml
     [workspace]
     members = ["crates/replyguy-core", "crates/replyguy-cli"]
     resolver = "2"
     ```
  2. Create directory structure:
     ```
     crates/
       replyguy-core/
         Cargo.toml
         src/
           lib.rs
       replyguy-cli/
         Cargo.toml
         src/
           main.rs
     ```
  3. Create `crates/replyguy-core/Cargo.toml`:
     ```toml
     [package]
     name = "replyguy-core"
     version = "0.1.0"
     edition = "2021"
     rust-version = "1.75"
     description = "Core library for ReplyGuy autonomous X growth assistant"

     [dependencies]
     tokio = { version = "1", features = ["full"] }
     reqwest = { version = "0.12", features = ["json"] }
     serde = { version = "1", features = ["derive"] }
     serde_json = "1"
     toml = "0.8"
     sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }
     tracing = "0.1"
     oauth2 = "4"
     thiserror = "2"
     chrono = { version = "0.4", features = ["serde"] }
     rand = "0.8"
     async-trait = "0.1"
     ```
  4. Create `crates/replyguy-cli/Cargo.toml`:
     ```toml
     [package]
     name = "replyguy-cli"
     version = "0.1.0"
     edition = "2021"
     rust-version = "1.75"
     description = "CLI for ReplyGuy autonomous X growth assistant"

     [[bin]]
     name = "replyguy"
     path = "src/main.rs"

     [dependencies]
     replyguy-core = { path = "../replyguy-core" }
     clap = { version = "4", features = ["derive"] }
     anyhow = "1"
     tracing-subscriber = { version = "0.3", features = ["env-filter"] }
     tokio = { version = "1", features = ["full"] }
     ```
  5. In `crates/replyguy-core/src/lib.rs`, add module declarations (initially commented out or empty) and a placeholder `pub fn version() -> &'static str`.
  6. In `crates/replyguy-cli/src/main.rs`, add a minimal `fn main()` that prints "replyguy" so the build succeeds.
  7. Run `cargo build --workspace` to verify everything compiles.
- **Files**:
  - `Cargo.toml` (workspace root)
  - `crates/replyguy-core/Cargo.toml`
  - `crates/replyguy-core/src/lib.rs`
  - `crates/replyguy-cli/Cargo.toml`
  - `crates/replyguy-cli/src/main.rs`
- **Parallel?**: No -- this must complete before all other subtasks.
- **Notes**: Pin dependency versions to avoid breaking changes. Use `resolver = "2"` in the workspace for proper feature resolution. The `[[bin]]` section in replyguy-cli sets the binary name to `replyguy` (not `replyguy-cli`).

### Subtask T002 -- Define Error Types

- **Purpose**: Establish typed error enums so all modules have well-defined error boundaries. The library uses `thiserror` for structured errors; the binary uses `anyhow` for ergonomic reporting.
- **Steps**:
  1. Create `crates/replyguy-core/src/error.rs`.
  2. Define `ConfigError` with variants:
     - `MissingField { field: String }` -- a required config field is absent.
     - `InvalidValue { field: String, message: String }` -- a field has an unacceptable value.
     - `FileNotFound { path: String }` -- the config file does not exist at the specified path.
     - `ParseError { source: toml::de::Error }` -- TOML deserialization failed.
  3. Define `XApiError` with variants:
     - `RateLimited { retry_after: Option<u64> }` -- X API returned 429.
     - `AuthExpired` -- OAuth token is expired and refresh failed.
     - `AccountRestricted { message: String }` -- account is suspended or limited.
     - `Network { source: reqwest::Error }` -- network-level failure.
     - `ApiError { status: u16, message: String }` -- any other X API error.
  4. Define `LlmError` with variants:
     - `ProviderUnreachable { provider: String, source: reqwest::Error }` -- cannot reach the LLM endpoint.
     - `RateLimited { provider: String }` -- LLM provider rate limit hit.
     - `ParseFailure { message: String }` -- LLM response could not be parsed.
     - `NotConfigured` -- no LLM provider configured.
  5. Define `StorageError` with variants:
     - `Connection { source: sqlx::Error }` -- failed to connect to SQLite.
     - `Migration { source: sqlx::migrate::MigrateError }` -- migration failed.
     - `Query { source: sqlx::Error }` -- a query failed.
  6. Define `ScoringError` with variants:
     - `InvalidTweetData { message: String }` -- tweet data is missing or malformed for scoring.
  7. All enums derive `thiserror::Error` and `Debug`. Each variant uses `#[error("...")]` with a human-readable message.
  8. Add `pub mod error;` to `lib.rs` and `pub use error::*;`.
- **Files**:
  - `crates/replyguy-core/src/error.rs`
  - `crates/replyguy-core/src/lib.rs` (add module declaration)
- **Parallel?**: No -- should complete after T001 so the crate exists.
- **Notes**: Do NOT use `#[from]` on error variants where you want to add context (e.g., `LlmError::ProviderUnreachable` includes a `provider` field alongside the source). Use `#[source]` attribute for the `source` field where appropriate. Follow constitution: no `unwrap()`, add `///` doc comments to each enum and variant.

### Subtask T003 -- Define Config Types and Defaults

- **Purpose**: Define the strongly-typed configuration struct hierarchy that mirrors the `config.toml` structure from `contracts/cli-interface.md`. Provide built-in defaults so users only need to supply credentials and business profile.
- **Steps**:
  1. Create `crates/replyguy-core/src/config/mod.rs` and `crates/replyguy-core/src/config/defaults.rs`.
  2. In `config/mod.rs`, define the top-level `Config` struct with `#[derive(Debug, Clone, Deserialize)]` (from serde) containing these sections:
     - `x_api: XApiConfig` -- fields: `client_id: String`, `client_secret: Option<String>`.
     - `auth: AuthConfig` -- fields: `mode: String` (default "manual"), `callback_host: String` (default "127.0.0.1"), `callback_port: u16` (default 8080).
     - `business: BusinessProfile` -- fields: `product_name: String`, `product_description: String`, `product_url: Option<String>`, `target_audience: String`, `product_keywords: Vec<String>`, `competitor_keywords: Vec<String>`, `industry_topics: Vec<String>`.
     - `scoring: ScoringConfig` -- fields: `threshold: u32` (default 70), `keyword_relevance_max: f32` (default 40.0), `follower_count_max: f32` (default 20.0), `recency_max: f32` (default 15.0), `engagement_rate_max: f32` (default 25.0).
     - `limits: LimitsConfig` -- fields: `max_replies_per_day: u32` (default 20), `max_tweets_per_day: u32` (default 4), `max_threads_per_week: u32` (default 1), `min_action_delay_seconds: u64` (default 30), `max_action_delay_seconds: u64` (default 120).
     - `intervals: IntervalsConfig` -- fields: `mentions_check_seconds: u64` (default 300), `discovery_search_seconds: u64` (default 600), `content_post_window_seconds: u64` (default 14400), `thread_interval_seconds: u64` (default 604800).
     - `llm: LlmConfig` -- fields: `provider: String`, `api_key: Option<String>`, `model: String`, `base_url: Option<String>`.
     - `storage: StorageConfig` -- fields: `db_path: String` (default "~/.replyguy/replyguy.db"), `retention_days: u32` (default 90).
     - `logging: LoggingConfig` -- fields: `status_interval_seconds: u64` (default 0).
  3. Use `#[serde(default)]` on each section field so missing TOML sections use defaults. Use `#[serde(default = "default_fn")]` on individual fields where defaults differ from Rust's zero values.
  4. In `config/defaults.rs`, define `impl Default` for each sub-struct with the default values listed above.
  5. Add `pub mod config;` to `lib.rs`.
- **Files**:
  - `crates/replyguy-core/src/config/mod.rs`
  - `crates/replyguy-core/src/config/defaults.rs`
  - `crates/replyguy-core/src/lib.rs` (add module declaration)
- **Parallel?**: Yes -- can proceed in parallel with T004 once T001 is done.
- **Notes**: The `XApiConfig` and `LlmConfig` sections have no useful defaults for credentials (they must be provided by the user). Use `Option<String>` for optional fields and plain `String` for required ones. Match the TOML key names exactly to `contracts/cli-interface.md` (e.g., `max_replies_per_day`, not `maxRepliesPerDay`). All structs should derive `Debug, Clone, serde::Deserialize`. Consider also deriving `serde::Serialize` for future config export use cases.

### Subtask T004 -- Implement Config Loading with Layering

- **Purpose**: Implement the three-layer configuration loading: TOML file -> env var overlay -> CLI flag overrides, matching the precedence order in `contracts/cli-interface.md` (CLI flags > env vars > config.toml > built-in defaults).
- **Steps**:
  1. In `config/mod.rs`, implement `Config::load(config_path: Option<&str>) -> Result<Config, ConfigError>`:
     a. Determine config file path: use `config_path` argument if provided, else check `REPLYGUY_CONFIG` env var, else default to `~/.replyguy/config.toml`. Expand `~` to the user's home directory.
     b. Read the TOML file. If the file does not exist and the path was explicitly provided (via arg or env var), return `ConfigError::FileNotFound`. If using the default path and the file does not exist, use all defaults (allows running with only env vars).
     c. Deserialize with `toml::from_str::<Config>(&contents)`, mapping parse errors to `ConfigError::ParseError`.
  2. Implement `Config::apply_env_overrides(&mut self)`:
     a. Read all env vars with prefix `REPLYGUY_`.
     b. Map double underscores to nested keys. For example, `REPLYGUY_LLM__API_KEY` maps to `self.llm.api_key`.
     c. Handle the following env var mappings explicitly (no need for a generic reflection-based approach):
        - `REPLYGUY_X_API__CLIENT_ID` -> `x_api.client_id`
        - `REPLYGUY_X_API__CLIENT_SECRET` -> `x_api.client_secret`
        - `REPLYGUY_AUTH__MODE` -> `auth.mode`
        - `REPLYGUY_AUTH__CALLBACK_HOST` -> `auth.callback_host`
        - `REPLYGUY_AUTH__CALLBACK_PORT` -> `auth.callback_port`
        - `REPLYGUY_BUSINESS__PRODUCT_NAME` -> `business.product_name`
        - `REPLYGUY_BUSINESS__PRODUCT_DESCRIPTION` -> `business.product_description`
        - `REPLYGUY_BUSINESS__PRODUCT_URL` -> `business.product_url`
        - `REPLYGUY_BUSINESS__TARGET_AUDIENCE` -> `business.target_audience`
        - `REPLYGUY_BUSINESS__PRODUCT_KEYWORDS` -> `business.product_keywords` (comma-separated)
        - `REPLYGUY_BUSINESS__COMPETITOR_KEYWORDS` -> `business.competitor_keywords` (comma-separated)
        - `REPLYGUY_BUSINESS__INDUSTRY_TOPICS` -> `business.industry_topics` (comma-separated)
        - `REPLYGUY_SCORING__THRESHOLD` -> `scoring.threshold`
        - `REPLYGUY_LIMITS__MAX_REPLIES_PER_DAY` -> `limits.max_replies_per_day`
        - `REPLYGUY_LIMITS__MAX_TWEETS_PER_DAY` -> `limits.max_tweets_per_day`
        - `REPLYGUY_LIMITS__MAX_THREADS_PER_WEEK` -> `limits.max_threads_per_week`
        - `REPLYGUY_LIMITS__MIN_ACTION_DELAY_SECONDS` -> `limits.min_action_delay_seconds`
        - `REPLYGUY_LIMITS__MAX_ACTION_DELAY_SECONDS` -> `limits.max_action_delay_seconds`
        - `REPLYGUY_INTERVALS__MENTIONS_CHECK_SECONDS` -> `intervals.mentions_check_seconds`
        - `REPLYGUY_INTERVALS__DISCOVERY_SEARCH_SECONDS` -> `intervals.discovery_search_seconds`
        - `REPLYGUY_INTERVALS__CONTENT_POST_WINDOW_SECONDS` -> `intervals.content_post_window_seconds`
        - `REPLYGUY_INTERVALS__THREAD_INTERVAL_SECONDS` -> `intervals.thread_interval_seconds`
        - `REPLYGUY_LLM__PROVIDER` -> `llm.provider`
        - `REPLYGUY_LLM__API_KEY` -> `llm.api_key`
        - `REPLYGUY_LLM__MODEL` -> `llm.model`
        - `REPLYGUY_LLM__BASE_URL` -> `llm.base_url`
        - `REPLYGUY_STORAGE__DB_PATH` -> `storage.db_path`
        - `REPLYGUY_STORAGE__RETENTION_DAYS` -> `storage.retention_days`
        - `REPLYGUY_LOGGING__STATUS_INTERVAL_SECONDS` -> `logging.status_interval_seconds`
     d. For numeric fields, parse the string and return `ConfigError::InvalidValue` if parsing fails.
     e. For `Vec<String>` fields (keywords, topics), split the env var value on commas and trim whitespace.
  3. The full loading sequence in `Config::load`:
     a. Load from TOML file (or use defaults).
     b. Call `apply_env_overrides()`.
     c. Return the config (CLI overrides are applied later by the binary crate, since they come from Clap).
- **Files**:
  - `crates/replyguy-core/src/config/mod.rs`
- **Parallel?**: Yes -- can proceed in parallel with T003 (both edit `config/mod.rs`, but T003 defines types while T004 adds loading logic; coordinate or combine).
- **Notes**: Use `std::env::var()` for env var reads. Handle `~` expansion for paths using `dirs::home_dir()` or a manual replacement. Do not use a config library like `config-rs` -- keep it simple with manual layering per the research decision in `research.md` Section 6. Test each layer independently: TOML-only, env-only, combined.

### Subtask T005 -- Implement Config Validation

- **Purpose**: Validate the loaded configuration and return ALL errors found (not just the first), so users can fix everything in one pass.
- **Steps**:
  1. In `config/mod.rs`, implement `Config::validate(&self) -> Result<(), Vec<ConfigError>>`:
     a. Collect all errors into a `Vec<ConfigError>`.
     b. Validate business profile:
        - `business.product_name` must not be empty -> `ConfigError::MissingField { field: "business.product_name" }`.
        - At least one of `product_keywords` or `competitor_keywords` must be non-empty -> `ConfigError::MissingField { field: "business.product_keywords or business.competitor_keywords" }`.
     c. Validate LLM provider:
        - `llm.provider` must be one of `"openai"`, `"anthropic"`, `"ollama"` -> `ConfigError::InvalidValue { field: "llm.provider", message: "must be openai, anthropic, or ollama" }`.
        - If provider is `"openai"` or `"anthropic"`, `llm.api_key` must be `Some` and non-empty.
     d. Validate auth mode:
        - `auth.mode` must be `"manual"` or `"local_callback"` -> `ConfigError::InvalidValue`.
     e. Validate scoring:
        - `scoring.threshold` must be in range 0..=100 -> `ConfigError::InvalidValue`.
     f. Validate limits:
        - All rate limit values (`max_replies_per_day`, `max_tweets_per_day`, `max_threads_per_week`) must be > 0 -> `ConfigError::InvalidValue`.
        - `min_action_delay_seconds` must be <= `max_action_delay_seconds`.
     g. If the errors vec is empty, return `Ok(())`. Otherwise, return `Err(errors)`.
  2. Provide a `Config::load_and_validate(config_path: Option<&str>) -> Result<Config, Vec<ConfigError>>` convenience method that calls `load()` then `validate()`.
- **Files**:
  - `crates/replyguy-core/src/config/mod.rs`
- **Parallel?**: No -- depends on T003 (config types) and T004 (config loading).
- **Notes**: Validation runs after all layers are applied. The binary crate will call `validate()` after applying CLI overrides. Return `Vec<ConfigError>` so the CLI can display all issues at once (important for UX per FR-019). Consider using a helper macro or function to reduce repetition in validation checks.

### Subtask T006 -- Create CLI Skeleton with Clap

- **Purpose**: Define the top-level CLI app with all 8 subcommands, global flags, and stub implementations. This is the user-facing entry point that must match `contracts/cli-interface.md` exactly.
- **Steps**:
  1. Create `crates/replyguy-cli/src/main.rs` with the Clap-derived top-level parser:
     ```rust
     use clap::Parser;

     #[derive(Parser)]
     #[command(name = "replyguy")]
     #[command(version)]
     #[command(about = "Autonomous X growth assistant")]
     struct Cli {
         /// Path to config.toml
         #[arg(short = 'c', long, default_value = "~/.replyguy/config.toml")]
         config: String,

         /// Enable verbose logging (debug level)
         #[arg(short, long)]
         verbose: bool,

         /// Suppress all output except errors
         #[arg(short, long)]
         quiet: bool,

         #[command(subcommand)]
         command: Commands,
     }
     ```
  2. Define the `Commands` enum with all 8 subcommands:
     ```rust
     #[derive(clap::Subcommand)]
     enum Commands {
         /// Start the autonomous agent
         Run(commands::RunArgs),
         /// Authenticate with X API
         Auth(commands::AuthArgs),
         /// Validate configuration and connectivity
         Test(commands::TestArgs),
         /// Run discovery loop once
         Discover(commands::DiscoverArgs),
         /// Check and reply to mentions
         Mentions(commands::MentionsArgs),
         /// Generate and post an original tweet
         Post(commands::PostArgs),
         /// Generate and post an educational thread
         Thread(commands::ThreadArgs),
         /// Score a specific tweet
         Score(commands::ScoreArgs),
     }
     ```
  3. Create `crates/replyguy-cli/src/commands/mod.rs` with all subcommand arg structs:
     - `RunArgs`: `--status-interval <SECONDS>` (u64, default 0).
     - `AuthArgs`: `--mode <MODE>` (Optional string, possible values: manual, local_callback).
     - `TestArgs`: no extra args.
     - `DiscoverArgs`: `--dry-run` (bool flag), `--limit <N>` (u32, default 50).
     - `MentionsArgs`: `--dry-run` (bool flag), `--limit <N>` (u32, default 20).
     - `PostArgs`: `--dry-run` (bool flag), `--topic <TOPIC>` (Optional string).
     - `ThreadArgs`: `--dry-run` (bool flag), `--topic <TOPIC>` (Optional string), `--count <N>` (Optional u32).
     - `ScoreArgs`: `tweet_id` (positional String argument).
  4. In `main()`, set up a `#[tokio::main]` async main, initialize `tracing_subscriber` based on verbose/quiet flags:
     - `--quiet`: error level only.
     - default (neither): warn level.
     - `--verbose`: debug level.
  5. Match on the command and call a stub function for each that prints `"<command> not implemented yet"` and returns `Ok(())`.
  6. Verify: `cargo run --bin replyguy -- --help` matches the contract. `cargo run --bin replyguy -- --version` prints the version. Each subcommand's `--help` shows the correct flags.
- **Files**:
  - `crates/replyguy-cli/src/main.rs`
  - `crates/replyguy-cli/src/commands/mod.rs`
- **Parallel?**: No -- depends on T001 (workspace exists). Can proceed in parallel with T002-T005 if using minimal imports from replyguy-core.
- **Notes**: The `--verbose` and `--quiet` flags are mutually exclusive in practice but Clap does not enforce this by default. Consider using `#[arg(conflicts_with = "quiet")]` on verbose. Match the exact flag names from `contracts/cli-interface.md`: `-c`/`--config`, `-v`/`--verbose`, `-q`/`--quiet`. The `score` subcommand takes a positional argument `<TWEET_ID>`, not a flag. Use `#[command(version)]` to pull the version from `Cargo.toml` automatically.

## Test Strategy

- **Build test**: `cargo build --workspace` must succeed with zero errors.
- **Clippy check**: `cargo clippy --workspace -- -D warnings` must pass.
- **Format check**: `cargo fmt --all --check` must pass.
- **CLI help test**: `cargo run --bin replyguy -- --help` output matches `contracts/cli-interface.md` commands and options.
- **Version test**: `cargo run --bin replyguy -- --version` prints `replyguy 0.1.0`.
- **Config unit tests**: In `config/mod.rs`, add `#[cfg(test)] mod tests` with:
  - Test loading from a valid TOML string.
  - Test that missing optional sections use defaults.
  - Test env var override for a string field, a numeric field, and a comma-separated vec field.
  - Test validation catches missing `product_name`.
  - Test validation catches invalid `llm.provider`.
  - Test validation catches `threshold > 100`.
  - Test validation returns multiple errors at once.
- **Error type test**: Verify each error variant formats its message correctly.
- Run with: `cargo test --workspace`.

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Config layering complexity -- env var overlay may conflict with TOML deserialization defaults | Test each layer independently: TOML-only, env-only, combined. Write unit tests for each env var mapping. |
| Dependency version conflicts between crates | Pin exact minor versions in `Cargo.toml` (e.g., `tokio = { version = "1.40" }`). Run `cargo update` only intentionally. |
| Clap derive macro version incompatibilities | Use `clap = "4"` with the `derive` feature. Test `--help` output manually after any clap update. |
| `~` expansion in paths not working cross-platform | Use `dirs::home_dir()` crate or manual `$HOME` / `%USERPROFILE%` expansion. Test on macOS, Linux, and Windows path conventions. |
| Config validation returning only first error (poor UX) | Explicitly collect all errors into a `Vec` and return them together. Test that multiple simultaneous validation failures all appear. |

## Review Guidance

- **Workspace structure**: Verify that `cargo build --workspace` compiles both crates. Check that the binary name is `replyguy` (not `replyguy-cli`).
- **Error types**: Verify all five error enums match the variants listed in `research.md` Section 5. Check that `#[error("...")]` messages are human-readable. Confirm no `unwrap()` in any production code.
- **Config defaults**: Cross-reference every default value against `contracts/cli-interface.md` Configuration File Contract section. Ensure `#[serde(default)]` is used correctly.
- **Config layering**: Verify precedence order: CLI flags > env vars > TOML > defaults. Test that an env var overrides a TOML value. Test that a missing TOML file with env vars still works.
- **Config validation**: Confirm `validate()` returns ALL errors, not just the first. Test edge cases: threshold = 0 (valid), threshold = 100 (valid), threshold = 101 (invalid).
- **CLI contract compliance**: Compare `cargo run --bin replyguy -- --help` output against `contracts/cli-interface.md` line by line. Check every subcommand's flags, positional args, and defaults.
- **Doc comments**: Verify all public structs, enums, and functions have `///` doc comments per constitution.
- **Quality gates**: Run `cargo clippy --workspace -- -D warnings` and `cargo fmt --all --check`.

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
- 2026-02-22T00:22:11Z – claude-opus – shell_pid=35288 – lane=doing – Assigned agent via workflow command
- 2026-02-22T00:28:50Z – claude-opus – shell_pid=35288 – lane=for_review – Ready for review: Cargo workspace, error types, config system with layered loading/validation, CLI skeleton with all 8 subcommands. 28 tests pass, clippy clean, fmt clean.
- 2026-02-22T00:34:26Z – claude-opus – shell_pid=35288 – lane=done – Security review passed, merged to main. Fixed env var test race condition.
