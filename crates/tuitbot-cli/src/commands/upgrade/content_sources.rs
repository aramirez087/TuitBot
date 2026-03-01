/// Upgrade group support for deployment mode, connectors, and content sources.
///
/// These groups were introduced in the deployment-aware content source setup
/// (Sessions 01-06). They detect and patch the `deployment_mode`,
/// `[connectors.google_drive]`, and `[content_sources]` config sections.
use anyhow::Result;
use console::Style;
use dialoguer::Select;
use toml_edit::{value, DocumentMut};

// ---------------------------------------------------------------------------
// Patching
// ---------------------------------------------------------------------------

/// Patch `deployment_mode = "<mode>"` at the document root.
pub(super) fn patch_deployment_mode(doc: &mut DocumentMut, mode: &str) {
    if doc.get("deployment_mode").is_some() {
        return;
    }

    doc.insert("deployment_mode", value(mode));

    if let Some(mut key) = doc.key_mut("deployment_mode") {
        key.leaf_decor_mut().set_prefix(
            "\n# --- Deployment Mode ---\n\
             # Controls which content source types are available.\n\
             # \"desktop\" (default), \"self_host\", or \"cloud\"\n",
        );
    }
}

/// Patch `[connectors.google_drive]` with client_id and client_secret.
pub(super) fn patch_connectors(doc: &mut DocumentMut, client_id: &str, client_secret: &str) {
    if doc.get("connectors").is_some() {
        return;
    }

    let mut gd_table = toml_edit::Table::new();
    gd_table.insert("client_id", value(client_id));
    gd_table.insert("client_secret", value(client_secret));
    gd_table.insert(
        "redirect_uri",
        value("http://localhost:3001/api/connectors/google-drive/callback"),
    );

    let mut connectors = toml_edit::Table::new();
    connectors.insert("google_drive", toml_edit::Item::Table(gd_table));

    connectors.decor_mut().set_prefix(
        "\n# --- Connectors ---\n\
         # OAuth application credentials for remote source linking.\n\
         # Get these from Google Cloud Console > APIs & Services > Credentials.\n",
    );

    doc.insert("connectors", toml_edit::Item::Table(connectors));
}

/// Patch an empty `[content_sources]` scaffold section.
pub(super) fn patch_content_sources(doc: &mut DocumentMut) {
    if doc.get("content_sources").is_some() {
        return;
    }

    let table = toml_edit::Table::new();
    let mut item = toml_edit::Item::Table(table);
    if let Some(t) = item.as_table_mut() {
        t.decor_mut().set_prefix(
            "\n# --- Content Sources ---\n\
             # Configure via the dashboard: Settings > Content Sources.\n\
             # Desktop: point to your Obsidian vault or notes folder.\n\
             # Self-hosted/Cloud: connect Google Drive.\n",
        );
    }

    doc.insert("content_sources", item);
}

// ---------------------------------------------------------------------------
// Legacy SA-key notice
// ---------------------------------------------------------------------------

/// Check raw TOML content for `service_account_key` entries without a sibling
/// `connection_id`. Returns true if any were found (and a notice was printed).
pub(super) fn print_legacy_sa_key_notice(content: &str) -> bool {
    let table: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let root = match table.as_table() {
        Some(t) => t,
        None => return false,
    };

    let sources = match root
        .get("content_sources")
        .and_then(|cs| cs.as_table())
        .and_then(|cs| cs.get("sources"))
        .and_then(|s| s.as_array())
    {
        Some(arr) => arr,
        None => return false,
    };

    let has_legacy = sources.iter().any(|src| {
        let has_sa = src
            .get("service_account_key")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty());
        let has_conn = src
            .get("connection_id")
            .and_then(|v| v.as_integer())
            .is_some();
        has_sa && !has_conn
    });

    if has_legacy {
        let dim = Style::new().dim();
        let tip = Style::new().cyan().bold();
        eprintln!();
        eprintln!(
            "{}",
            tip.apply_to("Tip: Your Google Drive source uses a service-account key (legacy).")
        );
        eprintln!(
            "{}",
            dim.apply_to(
                "  Consider switching to a linked account for simpler setup.\n  \
                 Open the dashboard > Settings > Content Sources > Connect Google Drive."
            )
        );
        eprintln!();
    }

    has_legacy
}

// ---------------------------------------------------------------------------
// Interactive prompts
// ---------------------------------------------------------------------------

/// Prompt user for deployment mode selection.
pub(super) fn prompt_deployment_mode() -> Result<String> {
    let modes = &["desktop", "self_host", "cloud"];
    let descriptions = &[
        "Desktop  -- Tauri native app with local file picker",
        "Self-Host -- Docker/VPS with browser dashboard",
        "Cloud    -- Managed service (no local filesystem access)",
    ];

    let selection = Select::new()
        .with_prompt("How do you run Tuitbot?")
        .items(descriptions)
        .default(0)
        .interact()?;

    Ok(modes[selection].to_string())
}

/// Prompt user for Google Drive OAuth credentials (or skip).
pub(super) fn prompt_connectors() -> Result<Option<(String, String)>> {
    let dim = Style::new().dim();
    eprintln!(
        "{}",
        dim.apply_to(
            "  If you use Google Drive for content, enter your GCP OAuth credentials.\n  \
             You can skip this and configure via environment variables or the dashboard later."
        )
    );

    let skip = dialoguer::Confirm::new()
        .with_prompt("Skip Google Drive connector setup?")
        .default(true)
        .interact()?;

    if skip {
        return Ok(None);
    }

    let client_id: String = dialoguer::Input::new()
        .with_prompt("GCP OAuth Client ID")
        .interact_text()?;

    let client_secret: String = dialoguer::Input::new()
        .with_prompt("GCP OAuth Client Secret")
        .interact_text()?;

    Ok(Some((client_id, client_secret)))
}

// ---------------------------------------------------------------------------
// Detection helper (for tests)
// ---------------------------------------------------------------------------

/// Check if a TOML string has any `service_account_key` without `connection_id`.
/// Exposed for testing.
#[cfg(test)]
pub(super) fn has_legacy_sa_key(content: &str) -> bool {
    let table: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let root = match table.as_table() {
        Some(t) => t,
        None => return false,
    };

    let sources = match root
        .get("content_sources")
        .and_then(|cs| cs.as_table())
        .and_then(|cs| cs.get("sources"))
        .and_then(|s| s.as_array())
    {
        Some(arr) => arr,
        None => return false,
    };

    sources.iter().any(|src| {
        let has_sa = src
            .get("service_account_key")
            .and_then(|v| v.as_str())
            .is_some_and(|s| !s.is_empty());
        let has_conn = src
            .get("connection_id")
            .and_then(|v| v.as_integer())
            .is_some();
        has_sa && !has_conn
    })
}
