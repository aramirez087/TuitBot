use super::UPDATE_TARGET_ENV;

/// Returns the platform target triple for asset name construction.
pub(super) fn platform_target_for(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu"),
        ("macos", "x86_64") => Some("x86_64-apple-darwin"),
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("windows", "x86_64") => Some("x86_64-pc-windows-msvc"),
        _ => None,
    }
}

pub(super) fn resolve_platform_target(
    override_target: Option<&str>,
    os: &str,
    arch: &str,
) -> Option<String> {
    if let Some(target) = override_target {
        let trimmed = target.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    platform_target_for(os, arch).map(str::to_string)
}

pub(super) fn platform_target() -> Option<String> {
    let override_target = std::env::var(UPDATE_TARGET_ENV).ok();
    resolve_platform_target(
        override_target.as_deref(),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

pub(super) fn archive_extension_for_target(target: &str) -> &'static str {
    if target.contains("-windows-") {
        "zip"
    } else {
        "tar.gz"
    }
}

/// Build the expected asset filename for the given binary name on this platform.
///
/// Returns `Some("{name}-{target}.{ext}")` or `None` if the platform is unsupported.
pub(super) fn asset_name_for_binary(name: &str) -> Option<String> {
    let target = platform_target()?;
    let ext = archive_extension_for_target(&target);
    Some(format!("{name}-{target}.{ext}"))
}

/// Build the expected asset filename for the CLI binary on this platform.
pub(super) fn platform_asset_name() -> Option<String> {
    asset_name_for_binary("tuitbot")
}
