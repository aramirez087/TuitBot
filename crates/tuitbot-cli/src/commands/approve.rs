//! Implementation of the `tuitbot approve` command.
//!
//! Interactive review of queued posts when approval_mode is enabled.
//! Shows pending items one at a time and allows approve/reject/skip.

use std::io::{self, BufRead, Write};
use tuitbot_core::config::Config;
use tuitbot_core::storage;

/// Execute the `tuitbot approve` command.
pub async fn execute(config: &Config) -> anyhow::Result<()> {
    if !config.approval_mode {
        eprintln!("Approval mode is not enabled.");
        eprintln!("Set `approval_mode = true` in your config.toml to queue posts for review.");
        return Ok(());
    }

    let pool = storage::init_db(&config.storage.db_path).await?;

    // Expire items older than 24 hours
    let expired = storage::approval_queue::expire_old_items(&pool, 24).await?;
    if expired > 0 {
        eprintln!("Expired {expired} item(s) older than 24 hours.\n");
    }

    let pending = storage::approval_queue::get_pending(&pool).await?;

    if pending.is_empty() {
        eprintln!("No pending items in the approval queue.");
        pool.close().await;
        return Ok(());
    }

    eprintln!("{} pending item(s) to review.\n", pending.len());

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut reviewed = 0u32;
    let mut approved = 0u32;
    let mut rejected = 0u32;

    for (i, item) in pending.iter().enumerate() {
        eprintln!("--- Item {}/{} ---", i + 1, pending.len());
        eprintln!("  Type:    {}", item.action_type);
        if !item.target_tweet_id.is_empty() {
            eprintln!(
                "  Reply to: {} (by {})",
                item.target_tweet_id, item.target_author
            );
        }
        if !item.topic.is_empty() {
            eprintln!("  Topic:   {}", item.topic);
        }
        if !item.archetype.is_empty() {
            eprintln!("  Style:   {}", item.archetype);
        }
        if item.score > 0.0 {
            eprintln!("  Score:   {:.1}", item.score);
        }
        eprintln!("  Created: {}", item.created_at);
        eprintln!();
        eprintln!("  Content:");
        for line in item.generated_content.lines() {
            eprintln!("    {line}");
        }
        eprintln!();
        eprint!("  [y]es / [n]o / [s]kip / [q]uit > ");
        io::stderr().flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        match choice.as_str() {
            "y" | "yes" => {
                storage::approval_queue::update_status(&pool, item.id, "approved").await?;
                eprintln!("  -> Approved\n");
                approved += 1;
                reviewed += 1;
            }
            "n" | "no" => {
                storage::approval_queue::update_status(&pool, item.id, "rejected").await?;
                eprintln!("  -> Rejected\n");
                rejected += 1;
                reviewed += 1;
            }
            "q" | "quit" => {
                eprintln!("  -> Quitting review\n");
                break;
            }
            _ => {
                eprintln!("  -> Skipped\n");
            }
        }
    }

    eprintln!("Review complete: {reviewed} reviewed, {approved} approved, {rejected} rejected.");
    eprintln!(
        "Remaining pending: {}",
        storage::approval_queue::pending_count(&pool).await?
    );

    pool.close().await;
    Ok(())
}
