use semver::Version;

use super::github::{has_update_assets, GitHubRelease};

fn parse_version_with_prefixes(tag: &str, prefixes: &[&str]) -> Option<Version> {
    for prefix in prefixes {
        let candidate = if prefix.is_empty() {
            tag
        } else if let Some(stripped) = tag.strip_prefix(prefix) {
            stripped
        } else {
            continue;
        };

        if let Ok(version) = Version::parse(candidate) {
            return Some(version);
        }
    }

    None
}

/// Extract a semver `Version` from a tag like `tuitbot-cli-v0.2.0`.
pub(super) fn parse_version_from_tag(tag: &str) -> Option<Version> {
    parse_version_with_prefixes(tag, &["tuitbot-cli-v", "v", ""])
}

fn parse_server_release_version_from_tag(tag: &str) -> Option<Version> {
    parse_version_with_prefixes(tag, &["tuitbot-server-v", "tuitbot-cli-v", "v", ""])
}

/// Returns true if `latest` is strictly newer than `current`.
pub(super) fn is_newer(latest: &Version, current: &Version) -> bool {
    latest > current
}

/// Return the newest parseable, non-draft, non-prerelease release.
pub(super) fn latest_known_release(
    releases: &[GitHubRelease],
) -> Option<(&GitHubRelease, Version)> {
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter_map(|r| parse_version_from_tag(&r.tag_name).map(|v| (r, v)))
        .max_by(|(_, a), (_, b)| a.cmp(b))
}

/// Return the newest release newer than `current` that includes this platform asset.
pub(super) fn latest_compatible_release<'a>(
    releases: &'a [GitHubRelease],
    current: &Version,
    asset_name: &str,
) -> Option<(&'a GitHubRelease, Version)> {
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter(|r| has_update_assets(r, asset_name))
        .filter_map(|r| parse_version_from_tag(&r.tag_name).map(|v| (r, v)))
        .filter(|(_, v)| is_newer(v, current))
        .max_by(|(_, a), (_, b)| a.cmp(b))
}

/// Return the newest non-draft, non-prerelease release that includes
/// the server platform archive and SHA256SUMS.
///
/// Unlike `latest_compatible_release`, this does not require the release to be
/// newer than a specific version — it just finds the best release with server assets.
pub(super) fn latest_release_with_server_asset<'a>(
    releases: &'a [GitHubRelease],
    server_asset_name: &str,
) -> Option<(&'a GitHubRelease, Version)> {
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter(|r| has_update_assets(r, server_asset_name))
        .filter_map(|r| parse_server_release_version_from_tag(&r.tag_name).map(|v| (r, v)))
        .max_by(|(_, a), (_, b)| a.cmp(b))
}
