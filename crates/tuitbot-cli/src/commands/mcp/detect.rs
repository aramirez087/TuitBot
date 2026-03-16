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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_json_snippet_write_profile() {
        let snippet = mcp_json_snippet("write");
        assert!(snippet.contains(r#""command": "tuitbot""#));
        assert!(snippet.contains(r#""args": ["mcp", "serve"]"#));
        // write profile should NOT include --profile flag
        assert!(!snippet.contains("--profile"));
    }

    #[test]
    fn mcp_json_snippet_readonly_profile() {
        let snippet = mcp_json_snippet("readonly");
        assert!(snippet.contains(r#""command": "tuitbot""#));
        assert!(snippet.contains("--profile"));
        assert!(snippet.contains("readonly"));
    }

    #[test]
    fn mcp_json_snippet_admin_profile() {
        let snippet = mcp_json_snippet("admin");
        assert!(snippet.contains("--profile"));
        assert!(snippet.contains("admin"));
    }

    #[test]
    fn mcp_json_snippet_is_valid_json() {
        let snippet = mcp_json_snippet("write");
        let parsed: serde_json::Value =
            serde_json::from_str(&snippet).expect("snippet should be valid JSON");
        assert!(parsed["mcpServers"]["tuitbot"]["command"].is_string());
    }

    #[test]
    fn mcp_json_snippet_readonly_is_valid_json() {
        let snippet = mcp_json_snippet("readonly");
        let parsed: serde_json::Value =
            serde_json::from_str(&snippet).expect("snippet should be valid JSON");
        let args = parsed["mcpServers"]["tuitbot"]["args"].as_array().unwrap();
        assert_eq!(args.len(), 4);
        assert_eq!(args[2].as_str().unwrap(), "--profile");
        assert_eq!(args[3].as_str().unwrap(), "readonly");
    }
}
