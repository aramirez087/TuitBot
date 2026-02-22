//! Implementation of the `tuitbot run` command.
//!
//! The main entry point for autonomous operation. Initializes all
//! dependencies, detects API tier, creates adapter structs, spawns
//! automation loops, and runs until a shutdown signal is received.

use std::sync::Arc;
use std::time::Duration;

use tuitbot_core::automation::{
    run_posting_queue_with_approval, scheduler_from_config, status_reporter::run_status_reporter,
    AnalyticsLoop, ContentLoop, DiscoveryLoop, MentionsLoop, PostExecutor, Runtime, TargetLoop,
    ThreadLoop,
};
use tuitbot_core::config::Config;
use tuitbot_core::startup::format_startup_banner;

use crate::deps::RuntimeDeps;

/// Execute the `tuitbot run` command.
///
/// Startup sequence:
/// 1. Initialize all shared dependencies via `RuntimeDeps`
/// 2. Print startup banner
/// 3. Spawn automation loops based on tier
/// 4. Run until shutdown
pub async fn execute(config: &Config, status_interval: u64) -> anyhow::Result<()> {
    // 1. Initialize all shared dependencies.
    let mut deps = RuntimeDeps::init(config, false).await?;

    // 2. Apply status_interval override.
    let effective_interval = if status_interval > 0 {
        status_interval
    } else {
        config.logging.status_interval_seconds
    };

    // 3. Print startup banner (always visible, even in default mode).
    let banner = format_startup_banner(deps.tier, &deps.capabilities, effective_interval);
    eprintln!("{banner}");

    // 4. Create runtime and spawn tasks.
    let mut runtime = Runtime::new();
    let min_delay = Duration::from_secs(config.limits.min_action_delay_seconds);
    let max_delay = Duration::from_secs(config.limits.max_action_delay_seconds);

    // Spawn posting queue consumer.
    let cancel = runtime.cancel_token();
    let post_rx = deps.post_rx.take().expect("post_rx not yet consumed");
    runtime.spawn("posting-queue", {
        let executor = deps.post_executor.clone() as Arc<dyn PostExecutor>;
        let approval_queue = deps.approval_queue.clone();
        async move {
            run_posting_queue_with_approval(
                post_rx,
                executor,
                approval_queue,
                min_delay,
                max_delay,
                cancel,
            )
            .await;
        }
    });

    // --- Content loop (all tiers) ---
    {
        let content_loop = ContentLoop::new(
            deps.tweet_gen.clone(),
            deps.content_safety.clone(),
            deps.content_storage.clone(),
            config.business.industry_topics.clone(),
            config.intervals.content_post_window_seconds,
            false,
        )
        .with_topic_scorer(deps.topic_scorer.clone());

        let cancel = runtime.cancel_token();
        let scheduler = scheduler_from_config(
            config.intervals.content_post_window_seconds,
            config.limits.min_action_delay_seconds,
            config.limits.max_action_delay_seconds,
        );
        let schedule = deps.active_schedule.clone();
        runtime.spawn("content-loop", async move {
            content_loop.run(cancel, scheduler, schedule).await;
        });
    }

    // --- Thread loop (all tiers) ---
    {
        let thread_loop = ThreadLoop::new(
            deps.thread_gen.clone(),
            deps.content_safety.clone(),
            deps.content_storage.clone(),
            deps.thread_poster.clone(),
            config.business.industry_topics.clone(),
            config.intervals.thread_interval_seconds,
            false,
        );

        let cancel = runtime.cancel_token();
        let scheduler = scheduler_from_config(
            config.intervals.thread_interval_seconds,
            config.limits.min_action_delay_seconds,
            config.limits.max_action_delay_seconds,
        );
        let schedule = deps.active_schedule.clone();
        runtime.spawn("thread-loop", async move {
            thread_loop.run(cancel, scheduler, schedule).await;
        });
    }

    // --- Tier-gated loops (Basic/Pro only) ---
    if deps.capabilities.discovery {
        // Discovery loop
        let discovery_loop = DiscoveryLoop::new(
            deps.searcher.clone(),
            deps.scorer.clone(),
            deps.reply_gen.clone(),
            deps.safety.clone(),
            deps.loop_storage.clone(),
            deps.post_sender.clone(),
            deps.keywords.clone(),
            config.scoring.threshold as f32,
            false,
        );

        let cancel = runtime.cancel_token();
        let scheduler = scheduler_from_config(
            config.intervals.discovery_search_seconds,
            config.limits.min_action_delay_seconds,
            config.limits.max_action_delay_seconds,
        );
        let schedule = deps.active_schedule.clone();
        runtime.spawn("discovery-loop", async move {
            discovery_loop.run(cancel, scheduler, schedule).await;
        });
    }

    if deps.capabilities.mentions {
        // Mentions loop
        let mentions_loop = MentionsLoop::new(
            deps.mentions_fetcher.clone(),
            deps.reply_gen.clone(),
            deps.safety.clone(),
            deps.post_sender.clone(),
            false,
        );

        let cancel = runtime.cancel_token();
        let scheduler = scheduler_from_config(
            config.intervals.mentions_check_seconds,
            config.limits.min_action_delay_seconds,
            config.limits.max_action_delay_seconds,
        );
        let schedule = deps.active_schedule.clone();
        let storage_clone = deps.loop_storage.clone();
        runtime.spawn("mentions-loop", async move {
            mentions_loop
                .run(cancel, scheduler, schedule, storage_clone)
                .await;
        });

        // Target loop
        let target_loop = TargetLoop::new(
            deps.target_adapter.clone(),
            deps.target_adapter.clone(),
            deps.reply_gen.clone(),
            deps.safety.clone(),
            deps.target_storage.clone(),
            deps.post_sender.clone(),
            deps.target_loop_config.clone(),
        );

        let cancel = runtime.cancel_token();
        let scheduler = scheduler_from_config(
            config.intervals.mentions_check_seconds,
            config.limits.min_action_delay_seconds,
            config.limits.max_action_delay_seconds,
        );
        let schedule = deps.active_schedule.clone();
        runtime.spawn("target-loop", async move {
            target_loop.run(cancel, scheduler, schedule).await;
        });

        // Analytics loop (no schedule gate -- analytics runs 24/7)
        let analytics_loop = AnalyticsLoop::new(
            deps.profile_adapter.clone(),
            deps.profile_adapter.clone(),
            deps.analytics_storage.clone(),
        );

        let cancel = runtime.cancel_token();
        let scheduler = scheduler_from_config(3600, 0, 0);
        runtime.spawn("analytics-loop", async move {
            analytics_loop.run(cancel, scheduler).await;
        });
    }

    // --- Status reporter ---
    if effective_interval > 0 {
        let scheduler = scheduler_from_config(effective_interval, 0, 0);
        let cancel = runtime.cancel_token();
        let status_querier = deps.status_querier.clone();
        runtime.spawn("status-reporter", async move {
            run_status_reporter(status_querier, scheduler, cancel).await;
        });
    }

    tracing::info!(
        tasks = runtime.task_count(),
        "All automation loops spawned, running until shutdown"
    );

    // 5. Run until shutdown signal.
    runtime.run_until_shutdown().await;

    tracing::info!("Shutdown complete.");
    Ok(())
}
