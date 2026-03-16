/// Pipe-safe stdout helpers for CLI output.
///
/// Rust's `println!` panics on broken pipe (EPIPE). This module provides
/// `write_stdout` which converts broken pipe errors into a clean process
/// exit, matching standard Unix tool behavior (`cat`, `head`, `grep`).
use std::io::{self, Write};

use crate::commands::OutputFormat;

/// Centralized output context that carries `--quiet` and `--output` flags.
///
/// Commands use this to decide what and how to print. All user-facing
/// output should flow through these helpers so that `--quiet` suppresses
/// informational messages and `--output json` emits structured JSON
/// instead of human text.
#[derive(Debug, Clone, Copy)]
pub struct CliOutput {
    pub quiet: bool,
    pub format: OutputFormat,
}

impl CliOutput {
    pub fn new(quiet: bool, format: OutputFormat) -> Self {
        Self { quiet, format }
    }

    pub fn is_json(self) -> bool {
        self.format.is_json()
    }

    /// Print an informational message to stderr (suppressed by `--quiet`).
    pub fn info(&self, msg: &str) {
        if !self.quiet && !self.is_json() {
            eprintln!("{msg}");
        }
    }

    /// Print an error message to stderr. In JSON mode, emits a JSON error
    /// envelope to stdout instead.
    pub fn error(&self, msg: &str) -> anyhow::Result<()> {
        if self.is_json() {
            let json = serde_json::json!({ "error": msg });
            write_stdout(&json.to_string())
        } else {
            eprintln!("Error: {msg}");
            Ok(())
        }
    }

    /// Emit a JSON value to stdout (used by `--output json` paths).
    pub fn json(&self, value: &impl serde::Serialize) -> anyhow::Result<()> {
        write_stdout(&serde_json::to_string(value)?)
    }
}

/// Reset SIGPIPE to default behavior on Unix.
///
/// Rust sets SIGPIPE to SIG_IGN by default, which causes write() to return
/// EPIPE instead of killing the process. Resetting to SIG_DFL restores
/// standard Unix pipe behavior: the process is terminated silently when
/// writing to a closed pipe.
///
/// This is the same approach used by ripgrep, fd, and other robust Rust CLIs.
pub fn reset_sigpipe() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

/// Write a string to stdout, exiting cleanly on broken pipe.
///
/// Returns `Ok(())` on success. On `BrokenPipe`, calls `std::process::exit(0)`
/// immediately — the consumer already got what it needed. Other IO errors
/// are propagated as `anyhow::Error`.
pub fn write_stdout(s: &str) -> anyhow::Result<()> {
    let result = (|| -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(s.as_bytes())?;
        handle.write_all(b"\n")?;
        handle.flush()
    })();

    match result {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            std::process::exit(0);
        }
        Err(e) => Err(anyhow::anyhow!("Failed to write to stdout: {e}")),
    }
}

/// Check whether an `anyhow::Error` chain contains a broken pipe IO error.
///
/// Used in the top-level error handler to exit cleanly if a broken pipe
/// propagated through `?` before being caught by `write_stdout`.
pub fn is_broken_pipe(err: &anyhow::Error) -> bool {
    for cause in err.chain() {
        if let Some(io_err) = cause.downcast_ref::<io::Error>() {
            if io_err.kind() == io::ErrorKind::BrokenPipe {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_broken_pipe_detects_broken_pipe() {
        let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe closed");
        let anyhow_err = anyhow::Error::new(io_err);
        assert!(is_broken_pipe(&anyhow_err));
    }

    #[test]
    fn test_is_broken_pipe_rejects_other_errors() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let anyhow_err = anyhow::Error::new(io_err);
        assert!(!is_broken_pipe(&anyhow_err));
    }

    #[test]
    fn test_is_broken_pipe_detects_nested_broken_pipe() {
        let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe closed");
        let anyhow_err: anyhow::Error = anyhow::anyhow!("write failed").context(io_err.to_string());
        // String context loses the type — only direct IO errors are detected
        assert!(!is_broken_pipe(&anyhow_err));

        // Direct wrapping preserves the type
        let io_err2 = io::Error::new(io::ErrorKind::BrokenPipe, "pipe closed");
        let anyhow_err2 = anyhow::Error::new(io_err2).context("write failed");
        assert!(is_broken_pipe(&anyhow_err2));
    }

    #[test]
    fn test_write_stdout_succeeds_with_valid_output() {
        // write_stdout to actual stdout should work in test context
        // (stdout is connected to the test harness, not a broken pipe)
        assert!(write_stdout("hello").is_ok());
    }

    // ── CliOutput ─────────────────────────────────────────────────────

    #[test]
    fn cli_output_new() {
        let out = CliOutput::new(false, OutputFormat::Text);
        assert!(!out.quiet);
        assert!(!out.is_json());
    }

    #[test]
    fn cli_output_json_mode() {
        let out = CliOutput::new(false, OutputFormat::Json);
        assert!(out.is_json());
    }

    #[test]
    fn cli_output_quiet_mode() {
        let out = CliOutput::new(true, OutputFormat::Text);
        assert!(out.quiet);
        assert!(!out.is_json());
    }

    #[test]
    fn cli_output_info_does_not_panic_when_quiet() {
        let out = CliOutput::new(true, OutputFormat::Text);
        out.info("this should be suppressed");
    }

    #[test]
    fn cli_output_info_does_not_panic_when_json() {
        let out = CliOutput::new(false, OutputFormat::Json);
        out.info("this should be suppressed in json mode");
    }

    #[test]
    fn cli_output_info_does_not_panic_in_normal_mode() {
        let out = CliOutput::new(false, OutputFormat::Text);
        out.info("normal message");
    }

    #[test]
    fn cli_output_error_text_mode() {
        let out = CliOutput::new(false, OutputFormat::Text);
        // Should not fail
        assert!(out.error("test error").is_ok());
    }

    #[test]
    fn cli_output_error_json_mode() {
        let out = CliOutput::new(false, OutputFormat::Json);
        // In JSON mode, error writes to stdout
        assert!(out.error("test error").is_ok());
    }

    #[test]
    fn cli_output_json_serializes_value() {
        let out = CliOutput::new(false, OutputFormat::Json);
        let val = serde_json::json!({"key": "value"});
        assert!(out.json(&val).is_ok());
    }

    #[test]
    fn cli_output_debug_impl() {
        let out = CliOutput::new(false, OutputFormat::Text);
        let debug = format!("{:?}", out);
        assert!(debug.contains("CliOutput"));
        assert!(debug.contains("quiet: false"));
    }

    #[test]
    fn cli_output_clone() {
        let out = CliOutput::new(true, OutputFormat::Json);
        let cloned = out;
        assert_eq!(out.quiet, cloned.quiet);
        assert_eq!(out.is_json(), cloned.is_json());
    }

    // ── is_broken_pipe additional ─────────────────────────────────────

    #[test]
    fn is_broken_pipe_with_non_io_error() {
        let err = anyhow::anyhow!("not an IO error");
        assert!(!is_broken_pipe(&err));
    }

    #[test]
    fn is_broken_pipe_with_permission_denied() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "no access");
        let err = anyhow::Error::new(io_err);
        assert!(!is_broken_pipe(&err));
    }
}
