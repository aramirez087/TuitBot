use super::*;
use tuitbot_core::config::Config;

// ============================================================================
// LoopFilter::from_args
// ============================================================================

fn tick_args(loops: Option<Vec<&str>>) -> TickArgs {
    TickArgs {
        dry_run: false,
        ignore_schedule: false,
        loops: loops.map(|v| v.into_iter().map(String::from).collect()),
        require_approval: false,
    }
}

#[test]
fn loop_filter_default_all_enabled() {
    let args = tick_args(None);
    let filter = LoopFilter::from_args(&args).unwrap();

    assert!(filter.analytics);
    assert!(filter.discovery);
    assert!(filter.mentions);
    assert!(filter.target);
    assert!(filter.content);
    assert!(filter.thread);
}

#[test]
fn loop_filter_specific_loops() {
    let args = tick_args(Some(vec!["discovery", "mentions"]));
    let filter = LoopFilter::from_args(&args).unwrap();

    assert!(!filter.analytics);
    assert!(filter.discovery);
    assert!(filter.mentions);
    assert!(!filter.target);
    assert!(!filter.content);
    assert!(!filter.thread);
}

#[test]
fn loop_filter_empty_vec_errors() {
    let args = tick_args(Some(vec![]));
    let err = LoopFilter::from_args(&args).unwrap_err();
    assert!(
        err.to_string().contains("--loops cannot be empty"),
        "expected empty error, got: {err}"
    );
}

#[test]
fn loop_filter_unknown_name_errors() {
    let args = tick_args(Some(vec!["discovery", "bogus"]));
    let err = LoopFilter::from_args(&args).unwrap_err();
    assert!(
        err.to_string().contains("unknown loop 'bogus'"),
        "expected unknown loop error, got: {err}"
    );
}

#[test]
fn loop_filter_single_loop() {
    let args = tick_args(Some(vec!["analytics"]));
    let filter = LoopFilter::from_args(&args).unwrap();

    assert!(filter.analytics);
    assert!(!filter.discovery);
    assert!(!filter.mentions);
    assert!(!filter.target);
    assert!(!filter.content);
    assert!(!filter.thread);
}

// ============================================================================
// compute_enrichment_tip
// ============================================================================

#[test]
fn enrichment_tip_fully_enriched_returns_none() {
    let mut config = Config::default();
    config.business.brand_voice = Some("Friendly expert".to_string());
    config.business.persona_opinions = vec!["Rust is great".to_string()];
    config.targets.accounts = vec!["levelsio".to_string()];

    assert!(compute_enrichment_tip(&config).is_none());
}

#[test]
fn enrichment_tip_empty_config_suggests_voice() {
    let config = Config::default();
    let tip = compute_enrichment_tip(&config);

    assert!(tip.is_some());
    let tip = tip.unwrap();
    assert!(
        tip.contains("voice"),
        "expected tip to mention 'voice', got: {tip}"
    );
}

#[test]
fn enrichment_tip_voice_done_suggests_persona() {
    let mut config = Config::default();
    config.business.brand_voice = Some("Friendly".to_string());

    let tip = compute_enrichment_tip(&config).unwrap();
    assert!(
        tip.contains("persona"),
        "expected tip to mention 'persona', got: {tip}"
    );
}

#[test]
fn enrichment_tip_voice_persona_done_suggests_targeting() {
    let mut config = Config::default();
    config.business.brand_voice = Some("Friendly".to_string());
    config.business.persona_opinions = vec!["Strong opinion".to_string()];

    let tip = compute_enrichment_tip(&config).unwrap();
    assert!(
        tip.contains("targeting"),
        "expected tip to mention 'targeting', got: {tip}"
    );
}

#[test]
fn enrichment_tip_contains_settings_enrich() {
    let config = Config::default();
    let tip = compute_enrichment_tip(&config).unwrap();

    assert!(
        tip.contains("tuitbot settings enrich"),
        "expected tip to include command, got: {tip}"
    );
}

// ============================================================================
// TickOutput serialization
// ============================================================================

fn sample_tick_output(enrichment_tip: Option<String>, errors: Vec<LoopErrorJson>) -> TickOutput {
    TickOutput {
        success: errors.is_empty(),
        tier: "free".to_string(),
        schedule_active: true,
        dry_run: true,
        approval_mode: false,
        duration_ms: 42,
        loops: LoopResults {
            analytics: LoopOutcome::Completed {
                detail: "ok".to_string(),
            },
            discovery: LoopOutcome::Skipped {
                reason: "filtered".to_string(),
            },
            mentions: LoopOutcome::Skipped {
                reason: "filtered".to_string(),
            },
            target: LoopOutcome::Skipped {
                reason: "filtered".to_string(),
            },
            content: LoopOutcome::Skipped {
                reason: "filtered".to_string(),
            },
            thread: LoopOutcome::Skipped {
                reason: "filtered".to_string(),
            },
        },
        errors,
        enrichment_tip,
    }
}

#[test]
fn tick_output_json_omits_null_enrichment_tip() {
    let output = sample_tick_output(None, vec![]);
    let json = serde_json::to_value(&output).unwrap();

    assert!(
        json.get("enrichment_tip").is_none(),
        "enrichment_tip should be absent when None"
    );
}

#[test]
fn tick_output_json_includes_enrichment_tip() {
    let output = sample_tick_output(Some("Run settings enrich".to_string()), vec![]);
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(
        json["enrichment_tip"].as_str().unwrap(),
        "Run settings enrich"
    );
}

#[test]
fn loop_outcome_completed_json() {
    let outcome = LoopOutcome::Completed {
        detail: "followers=100".to_string(),
    };
    let json = serde_json::to_value(&outcome).unwrap();

    assert_eq!(json["status"], "completed");
    assert_eq!(json["detail"], "followers=100");
}

#[test]
fn loop_outcome_skipped_json() {
    let outcome = LoopOutcome::Skipped {
        reason: "outside active hours".to_string(),
    };
    let json = serde_json::to_value(&outcome).unwrap();

    assert_eq!(json["status"], "skipped");
    assert_eq!(json["reason"], "outside active hours");
}

#[test]
fn loop_outcome_failed_json() {
    let outcome = LoopOutcome::Failed {
        error: "connection refused".to_string(),
    };
    let json = serde_json::to_value(&outcome).unwrap();

    assert_eq!(json["status"], "failed");
    assert_eq!(json["error"], "connection refused");
}

#[test]
fn tick_output_success_flag() {
    let output_ok = sample_tick_output(None, vec![]);
    assert!(output_ok.success);

    let output_err = sample_tick_output(
        None,
        vec![LoopErrorJson {
            loop_name: "discovery".to_string(),
            error: "timeout".to_string(),
        }],
    );
    assert!(!output_err.success);
}
