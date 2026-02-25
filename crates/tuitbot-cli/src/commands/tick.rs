//! Implementation of the `tuitbot tick` command.
//!
//! Runs each enabled automation loop once and exits. Designed for
//! integration with external schedulers (cron, systemd timers, launchd).
//! Acquires a process lock to prevent concurrent ticks, respects schedule
//! gates and rate limits, and outputs a structured JSON summary.

use std::sync::Arc;
use std::time::{Duration, Instant};

use fs2::FileExt;
use serde::Serialize;
use tokio_util::sync::CancellationToken;

use tuitbot_core::automation::{
    run_posting_queue_with_approval, AnalyticsLoop, ContentLoop, DiscoveryLoop, MentionsLoop,
    PostExecutor, TargetLoop, ThreadLoop,
};
use tuitbot_core::config::{Config, OperatingMode};

use super::{OutputFormat, TickArgs};
use crate::deps::RuntimeDeps;

// ============================================================================
// JSON output types
// ============================================================================

#[derive(Serialize)]
struct TickOutput {
    success: bool,
    tier: String,
    schedule_active: bool,
    dry_run: bool,
    approval_mode: bool,
    duration_ms: u64,
    loops: LoopResults,
    errors: Vec<LoopErrorJson>,
}

#[derive(Serialize)]
struct LoopResults {
    analytics: LoopOutcome,
    discovery: LoopOutcome,
    mentions: LoopOutcome,
    target: LoopOutcome,
    content: LoopOutcome,
    thread: LoopOutcome,
}

#[derive(Serialize)]
#[serde(tag = "status")]
enum LoopOutcome {
    #[serde(rename = "completed")]
    Completed { detail: String },
    #[serde(rename = "skipped")]
    Skipped { reason: String },
    #[serde(rename = "failed")]
    Failed { error: String },
}

#[derive(Serialize)]
struct LoopErrorJson {
    loop_name: String,
    error: String,
}

/// Which loops to run, resolved from CLI args + tier capabilities.
struct LoopFilter {
    analytics: bool,
    discovery: bool,
    mentions: bool,
    target: bool,
    content: bool,
    thread: bool,
}

impl LoopFilter {
    fn from_args(args: &TickArgs) -> Self {
        match &args.loops {
            Some(names) => Self {
                analytics: names.iter().any(|n| n == "analytics"),
                discovery: names.iter().any(|n| n == "discovery"),
                mentions: names.iter().any(|n| n == "mentions"),
                target: names.iter().any(|n| n == "target"),
                content: names.iter().any(|n| n == "content"),
                thread: names.iter().any(|n| n == "thread"),
            },
            None => Self {
                analytics: true,
                discovery: true,
                mentions: true,
                target: true,
                content: true,
                thread: true,
            },
        }
    }
}

// ============================================================================
// Execute
// ============================================================================

/// Execute the `tuitbot tick` command.
pub async fn execute(
    config: &Config,
    args: TickArgs,
    output_format: OutputFormat,
) -> anyhow::Result<()> {
    let start = Instant::now();
    let filter = LoopFilter::from_args(&args);

    // 1. Acquire process lock.
    let lock_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".tuitbot")
        .join("tuitbot.lock");

    // Ensure parent directory exists.
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&lock_path)?;

    if lock_file.try_lock_exclusive().is_err() {
        anyhow::bail!(
            "Another tuitbot tick process is running (lock: {})",
            lock_path.display()
        );
    }

    // 2. Initialize dependencies.
    let mut deps = RuntimeDeps::init(config, args.dry_run).await?;

    // 3. Check schedule gate.
    let schedule_active = if args.ignore_schedule {
        true
    } else {
        deps.active_schedule
            .as_ref()
            .map_or(true, |s| s.is_active())
    };

    if !schedule_active {
        let output = TickOutput {
            success: true,
            tier: deps.tier.to_string(),
            schedule_active: false,
            dry_run: args.dry_run,
            approval_mode: config.approval_mode,
            duration_ms: start.elapsed().as_millis() as u64,
            loops: LoopResults {
                analytics: LoopOutcome::Skipped {
                    reason: "outside active hours".to_string(),
                },
                discovery: LoopOutcome::Skipped {
                    reason: "outside active hours".to_string(),
                },
                mentions: LoopOutcome::Skipped {
                    reason: "outside active hours".to_string(),
                },
                target: LoopOutcome::Skipped {
                    reason: "outside active hours".to_string(),
                },
                content: LoopOutcome::Skipped {
                    reason: "outside active hours".to_string(),
                },
                thread: LoopOutcome::Skipped {
                    reason: "outside active hours".to_string(),
                },
            },
            errors: Vec::new(),
        };

        print_output(&output, output_format);
        return Ok(());
    }

    // 4. Spawn posting queue consumer with zero delay.
    let cancel = CancellationToken::new();
    let post_rx = deps.post_rx.take().expect("post_rx not yet consumed");
    let queue_cancel = cancel.clone();
    let queue_handle = tokio::spawn({
        let executor = deps.post_executor.clone() as Arc<dyn PostExecutor>;
        let approval_queue = deps.approval_queue.clone();
        async move {
            run_posting_queue_with_approval(
                post_rx,
                executor,
                approval_queue,
                Duration::ZERO,
                Duration::ZERO,
                queue_cancel,
            )
            .await;
        }
    });

    // 5. Run enabled loops sequentially.
    let mut errors: Vec<LoopErrorJson> = Vec::new();
    let is_composer = config.mode == OperatingMode::Composer;

    // --- Analytics (runs in both modes) ---
    let analytics_outcome = run_analytics(&deps, &filter, &mut errors).await;

    // --- Discovery (dry_run in composer mode) ---
    let discovery_outcome = if is_composer {
        LoopOutcome::Skipped {
            reason: "disabled in composer mode".to_string(),
        }
    } else {
        run_discovery(&deps, &filter, config, &mut errors).await
    };

    // --- Mentions (autopilot only) ---
    let mentions_outcome = if is_composer {
        LoopOutcome::Skipped {
            reason: "disabled in composer mode".to_string(),
        }
    } else {
        run_mentions(&deps, &filter, &mut errors).await
    };

    // --- Target (autopilot only) ---
    let target_outcome = if is_composer {
        LoopOutcome::Skipped {
            reason: "disabled in composer mode".to_string(),
        }
    } else {
        run_target(&deps, &filter, &mut errors).await
    };

    // --- Content (autopilot only) ---
    let content_outcome = if is_composer {
        LoopOutcome::Skipped {
            reason: "disabled in composer mode".to_string(),
        }
    } else {
        run_content(&deps, &filter, config, &mut errors).await
    };

    // --- Thread (autopilot only) ---
    let thread_outcome = if is_composer {
        LoopOutcome::Skipped {
            reason: "disabled in composer mode".to_string(),
        }
    } else {
        run_thread(&deps, &filter, config, &mut errors).await
    };

    // 6. Cancel posting queue and await drain (30s timeout).
    cancel.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(30), queue_handle).await;

    // 7. Close DB pool.
    deps.pool.close().await;

    // 8. Output summary.
    let output = TickOutput {
        success: errors.is_empty(),
        tier: deps.tier.to_string(),
        schedule_active,
        dry_run: args.dry_run,
        approval_mode: config.approval_mode,
        duration_ms: start.elapsed().as_millis() as u64,
        loops: LoopResults {
            analytics: analytics_outcome,
            discovery: discovery_outcome,
            mentions: mentions_outcome,
            target: target_outcome,
            content: content_outcome,
            thread: thread_outcome,
        },
        errors,
    };

    print_output(&output, output_format);

    // 9. Exit code: the process exits 0 on Ok, 1 via anyhow::bail.
    if !output.success {
        // Don't bail — we still want the output. Process exits 0 since
        // loop-level failures are captured in the JSON output.
    }

    Ok(())
}

// ============================================================================
// Per-loop runners
// ============================================================================

async fn run_analytics(
    deps: &RuntimeDeps,
    filter: &LoopFilter,
    errors: &mut Vec<LoopErrorJson>,
) -> LoopOutcome {
    if !filter.analytics {
        return LoopOutcome::Skipped {
            reason: "not in --loops filter".to_string(),
        };
    }

    if !deps.capabilities.mentions {
        return LoopOutcome::Skipped {
            reason: "requires Basic/Pro tier".to_string(),
        };
    }

    let analytics_loop = AnalyticsLoop::new(
        deps.profile_adapter.clone(),
        deps.profile_adapter.clone(),
        deps.analytics_storage.clone(),
    );

    match analytics_loop.run_iteration().await {
        Ok(summary) => LoopOutcome::Completed {
            detail: format!(
                "followers={}, replies_measured={}, tweets_measured={}",
                summary.follower_count, summary.replies_measured, summary.tweets_measured
            ),
        },
        Err(e) => {
            let msg = e.to_string();
            errors.push(LoopErrorJson {
                loop_name: "analytics".to_string(),
                error: msg.clone(),
            });
            LoopOutcome::Failed { error: msg }
        }
    }
}

async fn run_discovery(
    deps: &RuntimeDeps,
    filter: &LoopFilter,
    config: &Config,
    errors: &mut Vec<LoopErrorJson>,
) -> LoopOutcome {
    if !filter.discovery {
        return LoopOutcome::Skipped {
            reason: "not in --loops filter".to_string(),
        };
    }

    if !deps.capabilities.discovery {
        return LoopOutcome::Skipped {
            reason: "requires Basic/Pro tier".to_string(),
        };
    }

    if deps.keywords.is_empty() {
        return LoopOutcome::Skipped {
            reason: "no keywords configured".to_string(),
        };
    }

    let discovery_loop = DiscoveryLoop::new(
        deps.searcher.clone(),
        deps.scorer.clone(),
        deps.reply_gen.clone(),
        deps.safety.clone(),
        deps.loop_storage.clone(),
        deps.post_sender.clone(),
        deps.keywords.clone(),
        config.scoring.threshold as f32,
        deps.target_loop_config.dry_run,
    );

    match discovery_loop.run_once(None).await {
        Ok((_results, summary)) => LoopOutcome::Completed {
            detail: format!(
                "found={}, qualifying={}, replied={}, skipped={}, failed={}",
                summary.tweets_found,
                summary.qualifying,
                summary.replied,
                summary.skipped,
                summary.failed
            ),
        },
        Err(e) => {
            let msg = e.to_string();
            errors.push(LoopErrorJson {
                loop_name: "discovery".to_string(),
                error: msg.clone(),
            });
            LoopOutcome::Failed { error: msg }
        }
    }
}

async fn run_mentions(
    deps: &RuntimeDeps,
    filter: &LoopFilter,
    errors: &mut Vec<LoopErrorJson>,
) -> LoopOutcome {
    if !filter.mentions {
        return LoopOutcome::Skipped {
            reason: "not in --loops filter".to_string(),
        };
    }

    if !deps.capabilities.mentions {
        return LoopOutcome::Skipped {
            reason: "requires Basic/Pro tier".to_string(),
        };
    }

    let mentions_loop = MentionsLoop::new(
        deps.mentions_fetcher.clone(),
        deps.reply_gen.clone(),
        deps.safety.clone(),
        deps.post_sender.clone(),
        deps.target_loop_config.dry_run,
    );

    let storage: Arc<dyn tuitbot_core::automation::LoopStorage> = deps.loop_storage.clone();
    match mentions_loop.run_once(None, None, &storage).await {
        Ok((results, _since_id)) => {
            let replied = results
                .iter()
                .filter(|r| matches!(r, tuitbot_core::automation::MentionResult::Replied { .. }))
                .count();
            let skipped = results
                .iter()
                .filter(|r| matches!(r, tuitbot_core::automation::MentionResult::Skipped { .. }))
                .count();
            let failed = results
                .iter()
                .filter(|r| matches!(r, tuitbot_core::automation::MentionResult::Failed { .. }))
                .count();
            LoopOutcome::Completed {
                detail: format!(
                    "total={}, replied={}, skipped={}, failed={}",
                    results.len(),
                    replied,
                    skipped,
                    failed
                ),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            errors.push(LoopErrorJson {
                loop_name: "mentions".to_string(),
                error: msg.clone(),
            });
            LoopOutcome::Failed { error: msg }
        }
    }
}

async fn run_target(
    deps: &RuntimeDeps,
    filter: &LoopFilter,
    errors: &mut Vec<LoopErrorJson>,
) -> LoopOutcome {
    if !filter.target {
        return LoopOutcome::Skipped {
            reason: "not in --loops filter".to_string(),
        };
    }

    if !deps.capabilities.mentions {
        return LoopOutcome::Skipped {
            reason: "requires Basic/Pro tier".to_string(),
        };
    }

    if deps.target_loop_config.accounts.is_empty() {
        return LoopOutcome::Skipped {
            reason: "no target accounts configured".to_string(),
        };
    }

    let target_loop = TargetLoop::new(
        deps.target_adapter.clone(),
        deps.target_adapter.clone(),
        deps.reply_gen.clone(),
        deps.safety.clone(),
        deps.target_storage.clone(),
        deps.post_sender.clone(),
        deps.target_loop_config.clone(),
    );

    match target_loop.run_iteration().await {
        Ok(results) => {
            let replied = results
                .iter()
                .filter(|r| matches!(r, tuitbot_core::automation::TargetResult::Replied { .. }))
                .count();
            let skipped = results
                .iter()
                .filter(|r| matches!(r, tuitbot_core::automation::TargetResult::Skipped { .. }))
                .count();
            let failed = results
                .iter()
                .filter(|r| matches!(r, tuitbot_core::automation::TargetResult::Failed { .. }))
                .count();
            LoopOutcome::Completed {
                detail: format!(
                    "total={}, replied={}, skipped={}, failed={}",
                    results.len(),
                    replied,
                    skipped,
                    failed
                ),
            }
        }
        Err(e) => {
            let msg = e.to_string();
            errors.push(LoopErrorJson {
                loop_name: "target".to_string(),
                error: msg.clone(),
            });
            LoopOutcome::Failed { error: msg }
        }
    }
}

async fn run_content(
    deps: &RuntimeDeps,
    filter: &LoopFilter,
    config: &Config,
    errors: &mut Vec<LoopErrorJson>,
) -> LoopOutcome {
    if !filter.content {
        return LoopOutcome::Skipped {
            reason: "not in --loops filter".to_string(),
        };
    }

    if config.business.industry_topics.is_empty() {
        return LoopOutcome::Skipped {
            reason: "no industry topics configured".to_string(),
        };
    }

    let content_loop = ContentLoop::new(
        deps.tweet_gen.clone(),
        deps.content_safety.clone(),
        deps.content_storage.clone(),
        config.business.industry_topics.clone(),
        config.intervals.content_post_window_seconds,
        deps.target_loop_config.dry_run,
    )
    .with_topic_scorer(deps.topic_scorer.clone());

    match content_loop.run_once(None).await {
        tuitbot_core::automation::ContentResult::Posted { topic, content } => {
            LoopOutcome::Completed {
                detail: format!("topic='{}', chars={}", topic, content.len()),
            }
        }
        tuitbot_core::automation::ContentResult::TooSoon {
            elapsed_secs,
            window_secs,
        } => LoopOutcome::Skipped {
            reason: format!(
                "too soon since last tweet ({}s / {}s window)",
                elapsed_secs, window_secs
            ),
        },
        tuitbot_core::automation::ContentResult::RateLimited => LoopOutcome::Skipped {
            reason: "daily tweet limit reached".to_string(),
        },
        tuitbot_core::automation::ContentResult::NoTopics => LoopOutcome::Skipped {
            reason: "no topics configured".to_string(),
        },
        tuitbot_core::automation::ContentResult::Failed { error } => {
            errors.push(LoopErrorJson {
                loop_name: "content".to_string(),
                error: error.clone(),
            });
            LoopOutcome::Failed { error }
        }
    }
}

async fn run_thread(
    deps: &RuntimeDeps,
    filter: &LoopFilter,
    config: &Config,
    errors: &mut Vec<LoopErrorJson>,
) -> LoopOutcome {
    if !filter.thread {
        return LoopOutcome::Skipped {
            reason: "not in --loops filter".to_string(),
        };
    }

    if config.business.industry_topics.is_empty() {
        return LoopOutcome::Skipped {
            reason: "no industry topics configured".to_string(),
        };
    }

    let thread_loop = ThreadLoop::new(
        deps.thread_gen.clone(),
        deps.content_safety.clone(),
        deps.content_storage.clone(),
        deps.thread_poster.clone(),
        config.business.industry_topics.clone(),
        config.intervals.thread_interval_seconds,
        deps.target_loop_config.dry_run,
    );

    match thread_loop.run_once(None, None).await {
        tuitbot_core::automation::ThreadResult::Posted {
            topic, tweet_count, ..
        } => LoopOutcome::Completed {
            detail: format!("topic='{}', tweets={}", topic, tweet_count),
        },
        tuitbot_core::automation::ThreadResult::TooSoon {
            elapsed_secs,
            interval_secs,
        } => LoopOutcome::Skipped {
            reason: format!(
                "too soon since last thread ({}s / {}s interval)",
                elapsed_secs, interval_secs
            ),
        },
        tuitbot_core::automation::ThreadResult::RateLimited => LoopOutcome::Skipped {
            reason: "weekly thread limit reached".to_string(),
        },
        tuitbot_core::automation::ThreadResult::NoTopics => LoopOutcome::Skipped {
            reason: "no topics configured".to_string(),
        },
        tuitbot_core::automation::ThreadResult::ValidationFailed { error } => {
            errors.push(LoopErrorJson {
                loop_name: "thread".to_string(),
                error: error.clone(),
            });
            LoopOutcome::Failed { error }
        }
        tuitbot_core::automation::ThreadResult::PartialFailure {
            tweets_posted,
            total_tweets,
            error,
            ..
        } => {
            let detail = format!(
                "partial: {}/{} tweets posted, error: {}",
                tweets_posted, total_tweets, error
            );
            errors.push(LoopErrorJson {
                loop_name: "thread".to_string(),
                error: detail.clone(),
            });
            LoopOutcome::Failed { error: detail }
        }
        tuitbot_core::automation::ThreadResult::Failed { error } => {
            errors.push(LoopErrorJson {
                loop_name: "thread".to_string(),
                error: error.clone(),
            });
            LoopOutcome::Failed { error }
        }
    }
}

// ============================================================================
// Output
// ============================================================================

fn print_output(output: &TickOutput, format: OutputFormat) {
    if format.is_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(output).expect("serialization cannot fail")
        );
    } else {
        print_text_output(output);
    }
}

fn print_text_output(output: &TickOutput) {
    eprintln!(
        "tuitbot tick  tier={}  schedule={}  dry_run={}  approval_mode={}  duration={}ms",
        output.tier,
        if output.schedule_active {
            "active"
        } else {
            "inactive"
        },
        output.dry_run,
        output.approval_mode,
        output.duration_ms,
    );
    eprintln!();

    let loop_entries = [
        ("analytics", &output.loops.analytics),
        ("discovery", &output.loops.discovery),
        ("mentions", &output.loops.mentions),
        ("target", &output.loops.target),
        ("content", &output.loops.content),
        ("thread", &output.loops.thread),
    ];

    for (name, outcome) in &loop_entries {
        let (status, detail) = match outcome {
            LoopOutcome::Completed { detail } => ("OK", detail.as_str()),
            LoopOutcome::Skipped { reason } => ("SKIP", reason.as_str()),
            LoopOutcome::Failed { error } => ("FAIL", error.as_str()),
        };
        eprintln!("  {:<12} {:<6} {}", name, status, detail);
    }

    if !output.errors.is_empty() {
        eprintln!();
        eprintln!("Errors:");
        for err in &output.errors {
            eprintln!("  {}: {}", err.loop_name, err.error);
        }
    }

    eprintln!();
    eprintln!(
        "Result: {}",
        if output.success { "success" } else { "failure" }
    );
}
