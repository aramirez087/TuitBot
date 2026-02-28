//! Claude Code detection and MCP auto-registration.
//!
//! Checks whether the `claude` binary is on PATH and, if so, runs
//! `claude mcp add` to register Tuitbot as an MCP server automatically.

use std::process::Command;

use console::Style;

/// Returns `true` if the `claude` binary is found on PATH.
pub fn detect_claude_code() -> bool {
    Command::new("which")
        .arg("claude")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Register Tuitbot with Claude Code via `claude mcp add`.
pub fn register_with_claude_code(profile: &str) -> anyhow::Result<()> {
    let dim = Style::new().dim();
    let mut args = vec![
        "mcp".to_string(),
        "add".to_string(),
        "-s".to_string(),
        "user".to_string(),
        "tuitbot".to_string(),
        "--".to_string(),
        "tuitbot".to_string(),
        "mcp".to_string(),
        "serve".to_string(),
    ];

    if profile != "write" {
        args.push("--profile".to_string());
        args.push(profile.to_string());
    }

    eprintln!(
        "{}",
        dim.apply_to(format!("Running: claude {}", args.join(" ")))
    );

    let status = Command::new("claude")
        .args(&args)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run claude: {e}"))?;

    if !status.success() {
        anyhow::bail!("claude mcp add exited with status {status}");
    }

    Ok(())
}

/// Returns a JSON snippet for manual MCP configuration.
pub fn mcp_json_snippet(profile: &str) -> String {
    let args = if profile == "write" {
        r#"["mcp", "serve"]"#.to_string()
    } else {
        format!(r#"["mcp", "serve", "--profile", "{profile}"]"#)
    };

    format!(
        r#"{{
  "mcpServers": {{
    "tuitbot": {{
      "command": "tuitbot",
      "args": {args}
    }}
  }}
}}"#
    )
}
