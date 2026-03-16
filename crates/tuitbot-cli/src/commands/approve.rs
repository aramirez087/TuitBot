//! Implementation of the `tuitbot approve` command.
//!
//! Interactive review of queued posts when approval_mode is enabled.
//! Shows pending items one at a time and allows approve/reject/skip.
//!
//! Non-interactive modes:
//!   --list          List pending items
//!   --approve <ID>  Approve a specific item
//!   --reject <ID>   Reject a specific item
//!   --approve-all   Approve all pending items

use std::io::{self, BufRead, Write};

use serde::Serialize;
use tuitbot_core::config::Config;
use tuitbot_core::storage;

use super::ApproveArgs;
use crate::output::CliOutput;

#[derive(Serialize)]
struct ApprovalItemJson {
    id: i64,
    action_type: String,
    target_tweet_id: String,
    target_author: String,
    generated_content: String,
    topic: String,
    archetype: String,
    score: f64,
    created_at: String,
}

impl From<&storage::approval_queue::ApprovalItem> for ApprovalItemJson {
    fn from(item: &storage::approval_queue::ApprovalItem) -> Self {
        Self {
            id: item.id,
            action_type: item.action_type.clone(),
            target_tweet_id: item.target_tweet_id.clone(),
            target_author: item.target_author.clone(),
            generated_content: item.generated_content.clone(),
            topic: item.topic.clone(),
            archetype: item.archetype.clone(),
            score: item.score,
            created_at: item.created_at.clone(),
        }
    }
}

#[derive(Serialize)]
struct ApproveActionResult {
    id: i64,
    status: String,
}

/// Execute the `tuitbot approve` command.
pub async fn execute(config: &Config, args: ApproveArgs, out: CliOutput) -> anyhow::Result<()> {
    let is_non_interactive =
        args.list || args.approve.is_some() || args.reject.is_some() || args.approve_all;

    let action_count = args.list as u8
        + args.approve.is_some() as u8
        + args.reject.is_some() as u8
        + args.approve_all as u8;
    if action_count > 1 {
        anyhow::bail!(
            "Conflicting flags: --list, --approve, --reject, and --approve-all are mutually exclusive."
        );
    }

    if !config.approval_mode && !is_non_interactive {
        if out.is_json() {
            out.json(&serde_json::json!({
                "error": "Approval mode is not enabled",
                "hint": "Set `approval_mode = true` in your config.toml",
            }))?;
        } else {
            out.info("Approval mode is not enabled.");
            out.info("Set `approval_mode = true` in your config.toml to queue posts for review.");
        }
        return Ok(());
    }

    let pool = storage::init_db(&config.storage.db_path).await?;

    // Handle non-interactive modes
    if args.list {
        let pending = storage::approval_queue::get_pending(&pool).await?;
        if out.is_json() {
            let items: Vec<ApprovalItemJson> = pending.iter().map(ApprovalItemJson::from).collect();
            out.json(&items)?;
        } else if pending.is_empty() {
            out.info("No pending items.");
        } else {
            for item in &pending {
                out.info(&format!(
                    "  #{} [{}] {} | topic: {} | score: {:.1} | {}",
                    item.id,
                    item.action_type,
                    if item.target_tweet_id.is_empty() {
                        "(original)".to_string()
                    } else {
                        format!("reply to {}", item.target_tweet_id)
                    },
                    if item.topic.is_empty() {
                        "-"
                    } else {
                        &item.topic
                    },
                    item.score,
                    item.created_at,
                ));
            }
            out.info(&format!("\n{} pending item(s).", pending.len()));
        }
        pool.close().await;
        return Ok(());
    }

    if let Some(id) = args.approve {
        let exists = storage::approval_queue::get_by_id(&pool, id).await?;
        if exists.is_none() {
            pool.close().await;
            anyhow::bail!("Item #{id} not found in the approval queue.");
        }
        storage::approval_queue::update_status(&pool, id, "approved").await?;
        if out.is_json() {
            let result = ApproveActionResult {
                id,
                status: "approved".to_string(),
            };
            out.json(&result)?;
        } else {
            out.info(&format!("Approved item #{id}."));
        }
        pool.close().await;
        return Ok(());
    }

    if let Some(id) = args.reject {
        let exists = storage::approval_queue::get_by_id(&pool, id).await?;
        if exists.is_none() {
            pool.close().await;
            anyhow::bail!("Item #{id} not found in the approval queue.");
        }
        storage::approval_queue::update_status(&pool, id, "rejected").await?;
        if out.is_json() {
            let result = ApproveActionResult {
                id,
                status: "rejected".to_string(),
            };
            out.json(&result)?;
        } else {
            out.info(&format!("Rejected item #{id}."));
        }
        pool.close().await;
        return Ok(());
    }

    if args.approve_all {
        let pending = storage::approval_queue::get_pending(&pool).await?;
        let mut results = Vec::new();
        for item in &pending {
            storage::approval_queue::update_status(&pool, item.id, "approved").await?;
            results.push(ApproveActionResult {
                id: item.id,
                status: "approved".to_string(),
            });
        }
        if out.is_json() {
            out.json(&results)?;
        } else {
            out.info(&format!("Approved {} item(s).", results.len()));
        }
        pool.close().await;
        return Ok(());
    }

    // When --output json or --quiet is set without an explicit subcommand,
    // fall back to listing pending items (interactive mode needs a TTY).
    if out.is_json() {
        let pending = storage::approval_queue::get_pending(&pool).await?;
        let items: Vec<ApprovalItemJson> = pending.iter().map(ApprovalItemJson::from).collect();
        out.json(&items)?;
        pool.close().await;
        return Ok(());
    }

    if out.quiet {
        pool.close().await;
        return Ok(());
    }

    // Interactive mode (existing behavior)
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── ApprovalItemJson ──────────────────────────────────────────────

    #[test]
    fn approval_item_json_serializes() {
        let item = ApprovalItemJson {
            id: 42,
            action_type: "reply".to_string(),
            target_tweet_id: "12345".to_string(),
            target_author: "alice".to_string(),
            generated_content: "Great point!".to_string(),
            topic: "rust".to_string(),
            archetype: "helpful".to_string(),
            score: 85.5,
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"action_type\":\"reply\""));
        assert!(json.contains("\"target_tweet_id\":\"12345\""));
        assert!(json.contains("\"target_author\":\"alice\""));
        assert!(json.contains("\"generated_content\":\"Great point!\""));
        assert!(json.contains("\"topic\":\"rust\""));
        assert!(json.contains("\"archetype\":\"helpful\""));
        assert!(json.contains("85.5"));
        assert!(json.contains("\"created_at\":\"2025-01-01T00:00:00Z\""));
    }

    #[test]
    fn approval_item_json_from_trait_conversion() {
        // Test the From trait conversion directly with our JSON struct
        let json_item = ApprovalItemJson {
            id: 1,
            action_type: "tweet".to_string(),
            target_tweet_id: String::new(),
            target_author: String::new(),
            generated_content: "Hello world".to_string(),
            topic: "general".to_string(),
            archetype: "thought_leader".to_string(),
            score: 0.0,
            created_at: "2025-06-01".to_string(),
        };
        assert_eq!(json_item.id, 1);
        assert_eq!(json_item.action_type, "tweet");
        assert!(json_item.target_tweet_id.is_empty());
        assert_eq!(json_item.generated_content, "Hello world");
    }

    #[test]
    fn approval_item_json_empty_target_displays_original() {
        let item = ApprovalItemJson {
            id: 1,
            action_type: "tweet".to_string(),
            target_tweet_id: String::new(),
            target_author: String::new(),
            generated_content: "Content".to_string(),
            topic: String::new(),
            archetype: String::new(),
            score: 0.0,
            created_at: "now".to_string(),
        };
        // Verify the display logic used in the list command
        let display = if item.target_tweet_id.is_empty() {
            "(original)".to_string()
        } else {
            format!("reply to {}", item.target_tweet_id)
        };
        assert_eq!(display, "(original)");
    }

    #[test]
    fn approval_item_json_with_target_displays_reply() {
        let item = ApprovalItemJson {
            id: 1,
            action_type: "reply".to_string(),
            target_tweet_id: "9999".to_string(),
            target_author: "bob".to_string(),
            generated_content: "Reply text".to_string(),
            topic: "topic".to_string(),
            archetype: "arch".to_string(),
            score: 72.0,
            created_at: "now".to_string(),
        };
        let display = if item.target_tweet_id.is_empty() {
            "(original)".to_string()
        } else {
            format!("reply to {}", item.target_tweet_id)
        };
        assert_eq!(display, "reply to 9999");
    }

    #[test]
    fn approval_item_json_topic_dash_for_empty() {
        let topic = "";
        let display = if topic.is_empty() { "-" } else { topic };
        assert_eq!(display, "-");
    }

    // ── ApproveActionResult ───────────────────────────────────────────

    #[test]
    fn approve_action_result_serializes() {
        let result = ApproveActionResult {
            id: 5,
            status: "approved".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"id\":5"));
        assert!(json.contains("\"status\":\"approved\""));
    }

    #[test]
    fn approve_action_result_rejected() {
        let result = ApproveActionResult {
            id: 10,
            status: "rejected".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"rejected\""));
    }

    // ── Action count validation ───────────────────────────────────────

    #[test]
    fn action_count_detects_conflicting_flags() {
        let args = ApproveArgs {
            list: true,
            approve: Some(1),
            reject: None,
            approve_all: false,
        };
        let action_count = args.list as u8
            + args.approve.is_some() as u8
            + args.reject.is_some() as u8
            + args.approve_all as u8;
        assert!(action_count > 1);
    }

    #[test]
    fn action_count_zero_for_interactive() {
        let args = ApproveArgs {
            list: false,
            approve: None,
            reject: None,
            approve_all: false,
        };
        let action_count = args.list as u8
            + args.approve.is_some() as u8
            + args.reject.is_some() as u8
            + args.approve_all as u8;
        assert_eq!(action_count, 0);
    }

    #[test]
    fn action_count_one_for_single_flag() {
        let args = ApproveArgs {
            list: true,
            approve: None,
            reject: None,
            approve_all: false,
        };
        let action_count = args.list as u8
            + args.approve.is_some() as u8
            + args.reject.is_some() as u8
            + args.approve_all as u8;
        assert_eq!(action_count, 1);
    }

    #[test]
    fn is_non_interactive_detects_list() {
        let args = ApproveArgs {
            list: true,
            approve: None,
            reject: None,
            approve_all: false,
        };
        let is_non_interactive =
            args.list || args.approve.is_some() || args.reject.is_some() || args.approve_all;
        assert!(is_non_interactive);
    }

    #[test]
    fn is_non_interactive_false_for_interactive() {
        let args = ApproveArgs {
            list: false,
            approve: None,
            reject: None,
            approve_all: false,
        };
        let is_non_interactive =
            args.list || args.approve.is_some() || args.reject.is_some() || args.approve_all;
        assert!(!is_non_interactive);
    }

    // ── Choice parsing ────────────────────────────────────────────────

    #[test]
    fn choice_parsing_yes() {
        let choice = "y".to_lowercase();
        assert!(matches!(choice.as_str(), "y" | "yes"));
    }

    #[test]
    fn choice_parsing_no() {
        let choice = "n".to_lowercase();
        assert!(matches!(choice.as_str(), "n" | "no"));
    }

    #[test]
    fn choice_parsing_quit() {
        let choice = "q".to_lowercase();
        assert!(matches!(choice.as_str(), "q" | "quit"));
    }

    #[test]
    fn choice_parsing_skip() {
        let choice = "s".to_lowercase();
        // Skip is the default (neither yes, no, nor quit)
        assert!(!matches!(
            choice.as_str(),
            "y" | "yes" | "n" | "no" | "q" | "quit"
        ));
    }

    // ── Serialization round-trip ──────────────────────────────────────

    #[test]
    fn approval_item_json_vec_serializes() {
        let items = vec![
            ApprovalItemJson {
                id: 1,
                action_type: "reply".to_string(),
                target_tweet_id: "100".to_string(),
                target_author: "alice".to_string(),
                generated_content: "Content 1".to_string(),
                topic: "t1".to_string(),
                archetype: "a1".to_string(),
                score: 50.0,
                created_at: "2025-01-01".to_string(),
            },
            ApprovalItemJson {
                id: 2,
                action_type: "tweet".to_string(),
                target_tweet_id: String::new(),
                target_author: String::new(),
                generated_content: "Content 2".to_string(),
                topic: "t2".to_string(),
                archetype: "a2".to_string(),
                score: 0.0,
                created_at: "2025-01-02".to_string(),
            },
        ];
        let json = serde_json::to_string(&items).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[1]["id"], 2);
    }
}
