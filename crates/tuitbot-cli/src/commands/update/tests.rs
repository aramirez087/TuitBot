use semver::Version;

use super::github::{
    has_update_assets, parse_sha256sums, verify_sha256, GitHubAsset, GitHubRelease,
};
use super::platform::{
    archive_extension_for_target, asset_name_for_binary, platform_asset_name, platform_target,
    platform_target_for, resolve_platform_target,
};
use super::version::{
    is_newer, latest_compatible_release, latest_known_release, parse_version_from_tag,
};

fn release_with_assets(tag: &str, assets: &[&str]) -> GitHubRelease {
    GitHubRelease {
        tag_name: tag.to_string(),
        draft: false,
        prerelease: false,
        assets: assets
            .iter()
            .map(|name| GitHubAsset {
                name: (*name).to_string(),
                browser_download_url: String::new(),
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Version parsing
// ---------------------------------------------------------------------------

#[test]
fn parse_version_from_cli_tag() {
    let v = parse_version_from_tag("tuitbot-cli-v0.2.0").unwrap();
    assert_eq!(v, Version::new(0, 2, 0));
}

#[test]
fn parse_version_from_v_tag() {
    let v = parse_version_from_tag("v1.0.0").unwrap();
    assert_eq!(v, Version::new(1, 0, 0));
}

#[test]
fn parse_version_from_bare_semver() {
    let v = parse_version_from_tag("0.3.1").unwrap();
    assert_eq!(v, Version::new(0, 3, 1));
}

#[test]
fn parse_version_from_prerelease_tag() {
    let v = parse_version_from_tag("tuitbot-cli-v1.0.0-rc.1").unwrap();
    assert_eq!(v.major, 1);
    assert!(!v.pre.is_empty());
}

#[test]
fn parse_version_invalid_tag() {
    assert!(parse_version_from_tag("not-a-version").is_none());
    assert!(parse_version_from_tag("").is_none());
    assert!(parse_version_from_tag("tuitbot-cli-vgarbage").is_none());
}

// ---------------------------------------------------------------------------
// Version comparison
// ---------------------------------------------------------------------------

#[test]
fn is_newer_returns_true_for_higher_version() {
    let latest = Version::new(0, 2, 0);
    let current = Version::new(0, 1, 0);
    assert!(is_newer(&latest, &current));
}

#[test]
fn is_newer_returns_false_for_same_version() {
    let v = Version::new(0, 1, 0);
    assert!(!is_newer(&v, &v));
}

#[test]
fn is_newer_returns_false_for_older_version() {
    let latest = Version::new(0, 1, 0);
    let current = Version::new(0, 2, 0);
    assert!(!is_newer(&latest, &current));
}

#[test]
fn is_newer_handles_prerelease() {
    let release = Version::new(1, 0, 0);
    let prerelease = Version::parse("1.0.0-rc.1").unwrap();
    // A release is "newer" than its own prerelease
    assert!(is_newer(&release, &prerelease));
}

// ---------------------------------------------------------------------------
// Release selection
// ---------------------------------------------------------------------------

#[test]
fn latest_known_release_skips_draft_and_prerelease() {
    let mut draft = release_with_assets("tuitbot-cli-v9.9.9", &[]);
    draft.draft = true;

    let mut prerelease = release_with_assets("tuitbot-cli-v8.8.8-rc.1", &[]);
    prerelease.prerelease = true;

    let releases = vec![
        release_with_assets("tuitbot-core-v0.1.6", &[]),
        prerelease,
        release_with_assets("tuitbot-cli-v0.1.4", &[]),
        draft,
        release_with_assets("tuitbot-cli-v0.1.6", &[]),
    ];

    let (_, version) = latest_known_release(&releases).expect("release");
    assert_eq!(version, Version::new(0, 1, 6));
}

#[test]
fn latest_compatible_release_picks_newest_with_required_assets() {
    let releases = vec![
        release_with_assets(
            "tuitbot-cli-v0.1.2",
            &[
                "SHA256SUMS",
                "tuitbot-aarch64-unknown-linux-gnu.tar.gz",
                "tuitbot-x86_64-unknown-linux-gnu.tar.gz",
            ],
        ),
        release_with_assets(
            "tuitbot-cli-v0.1.5",
            &["SHA256SUMS", "tuitbot-x86_64-unknown-linux-gnu.tar.gz"],
        ),
        release_with_assets(
            "tuitbot-cli-v0.1.4",
            &["SHA256SUMS", "tuitbot-aarch64-unknown-linux-gnu.tar.gz"],
        ),
    ];

    let current = Version::new(0, 1, 1);
    let (_, version) = latest_compatible_release(
        &releases,
        &current,
        "tuitbot-aarch64-unknown-linux-gnu.tar.gz",
    )
    .expect("compatible release");

    assert_eq!(version, Version::new(0, 1, 4));
}

// ---------------------------------------------------------------------------
// Platform detection
// ---------------------------------------------------------------------------

#[test]
fn platform_target_for_maps_known_targets() {
    assert_eq!(
        platform_target_for("linux", "aarch64"),
        Some("aarch64-unknown-linux-gnu")
    );
    assert_eq!(
        platform_target_for("linux", "x86_64"),
        Some("x86_64-unknown-linux-gnu")
    );
    assert_eq!(
        platform_target_for("macos", "aarch64"),
        Some("aarch64-apple-darwin")
    );
    assert_eq!(
        platform_target_for("windows", "x86_64"),
        Some("x86_64-pc-windows-msvc")
    );
    assert_eq!(platform_target_for("linux", "armv7"), None);
}

#[test]
fn resolve_platform_target_prefers_override() {
    let target = resolve_platform_target(Some("aarch64-unknown-linux-gnu"), "macos", "aarch64");
    assert_eq!(target.as_deref(), Some("aarch64-unknown-linux-gnu"));
}

#[test]
fn resolve_platform_target_ignores_empty_override() {
    let target = resolve_platform_target(Some("   "), "linux", "aarch64");
    assert_eq!(target.as_deref(), Some("aarch64-unknown-linux-gnu"));
}

#[test]
fn archive_extension_for_windows_target_is_zip() {
    assert_eq!(
        archive_extension_for_target("x86_64-pc-windows-msvc"),
        "zip"
    );
    assert_eq!(
        archive_extension_for_target("aarch64-unknown-linux-gnu"),
        "tar.gz"
    );
}

#[test]
fn platform_asset_name_returns_some_on_supported() {
    // This test runs on the host platform; it should return Some on CI
    let name = platform_asset_name();
    if platform_target().is_some() {
        let name = name.unwrap();
        assert!(name.starts_with("tuitbot-"));
        assert!(name.ends_with(".tar.gz") || name.ends_with(".zip"));
    }
}

// ---------------------------------------------------------------------------
// asset_name_for_binary
// ---------------------------------------------------------------------------

#[test]
fn asset_name_for_binary_returns_server_asset() {
    if platform_target().is_some() {
        let name = asset_name_for_binary("tuitbot-server").unwrap();
        assert!(name.starts_with("tuitbot-server-"));
        assert!(name.ends_with(".tar.gz") || name.ends_with(".zip"));
    }
}

#[test]
fn asset_name_for_binary_matches_platform_asset_name() {
    // asset_name_for_binary("tuitbot") should equal platform_asset_name()
    assert_eq!(asset_name_for_binary("tuitbot"), platform_asset_name());
}

// ---------------------------------------------------------------------------
// has_update_assets
// ---------------------------------------------------------------------------

#[test]
fn has_update_assets_requires_both_archive_and_checksums() {
    let release = release_with_assets(
        "tuitbot-cli-v0.1.0",
        &["SHA256SUMS", "tuitbot-x86_64-unknown-linux-gnu.tar.gz"],
    );
    assert!(has_update_assets(
        &release,
        "tuitbot-x86_64-unknown-linux-gnu.tar.gz"
    ));
    assert!(!has_update_assets(
        &release,
        "tuitbot-aarch64-apple-darwin.tar.gz"
    ));
}

#[test]
fn has_update_assets_false_without_checksums() {
    let release = release_with_assets("v0.1.0", &["tuitbot-x86_64-unknown-linux-gnu.tar.gz"]);
    assert!(!has_update_assets(
        &release,
        "tuitbot-x86_64-unknown-linux-gnu.tar.gz"
    ));
}

// ---------------------------------------------------------------------------
// SHA256 verification
// ---------------------------------------------------------------------------

#[test]
fn verify_sha256_valid() {
    let data = b"hello world";
    let hash = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    assert!(verify_sha256(data, hash).is_ok());
}

#[test]
fn verify_sha256_uppercase_expected() {
    let data = b"hello world";
    let hash = "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9";
    assert!(verify_sha256(data, hash).is_ok());
}

#[test]
fn verify_sha256_invalid() {
    let data = b"hello world";
    let hash = "0000000000000000000000000000000000000000000000000000000000000000";
    assert!(verify_sha256(data, hash).is_err());
}

#[test]
fn parse_sha256sums_finds_entry() {
    let content = "\
abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890  tuitbot-x86_64-unknown-linux-gnu.tar.gz
1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef  tuitbot-aarch64-apple-darwin.tar.gz
";
    let hash = parse_sha256sums(content, "tuitbot-aarch64-apple-darwin.tar.gz");
    assert_eq!(
        hash.as_deref(),
        Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
    );
}

#[test]
fn parse_sha256sums_missing_entry() {
    let content = "abcdef1234567890  tuitbot-x86_64-unknown-linux-gnu.tar.gz\n";
    assert!(parse_sha256sums(content, "tuitbot-windows.zip").is_none());
}

#[test]
fn parse_sha256sums_empty() {
    assert!(parse_sha256sums("", "anything").is_none());
}

// ---------------------------------------------------------------------------
// Archive extraction
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
#[test]
fn extract_binary_from_tar_gz() {
    use super::binary::extract_named_binary;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};

    // Create a minimal tar.gz with a "tuitbot" file
    let mut gz_buf = Vec::new();
    {
        let gz = GzEncoder::new(&mut gz_buf, Compression::fast());
        let mut builder = Builder::new(gz);

        let content = b"#!/bin/sh\necho hello";
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();

        builder
            .append_data(&mut header, "tuitbot", &content[..])
            .unwrap();
        builder.finish().unwrap();
        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap();
    }

    let result = extract_named_binary(&gz_buf, "tuitbot").unwrap();
    assert_eq!(result, b"#!/bin/sh\necho hello");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn extract_binary_missing_from_tar_gz() {
    use super::binary::extract_named_binary;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};

    let mut gz_buf = Vec::new();
    {
        let gz = GzEncoder::new(&mut gz_buf, Compression::fast());
        let mut builder = Builder::new(gz);

        let content = b"not the binary";
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        builder
            .append_data(&mut header, "README.md", &content[..])
            .unwrap();
        builder.finish().unwrap();
        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap();
    }

    assert!(extract_named_binary(&gz_buf, "tuitbot").is_err());
}

#[cfg(not(target_os = "windows"))]
#[test]
fn extract_named_binary_finds_server_in_archive() {
    use super::binary::extract_named_binary;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};

    let mut gz_buf = Vec::new();
    {
        let gz = GzEncoder::new(&mut gz_buf, Compression::fast());
        let mut builder = Builder::new(gz);

        // Add a "tuitbot-server" binary
        let content = b"#!/bin/sh\necho server";
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();

        builder
            .append_data(&mut header, "tuitbot-server", &content[..])
            .unwrap();

        // Also add some other file
        let readme = b"README";
        let mut header2 = Header::new_gnu();
        header2.set_size(readme.len() as u64);
        header2.set_mode(0o644);
        header2.set_cksum();

        builder
            .append_data(&mut header2, "README.md", &readme[..])
            .unwrap();

        builder.finish().unwrap();
        let gz = builder.into_inner().unwrap();
        gz.finish().unwrap();
    }

    let result = extract_named_binary(&gz_buf, "tuitbot-server").unwrap();
    assert_eq!(result, b"#!/bin/sh\necho server");

    // And it should NOT find "tuitbot" (different name)
    let gz2 = gz_buf.clone();
    assert!(extract_named_binary(&gz2, "tuitbot").is_err());
}

// ---------------------------------------------------------------------------
// Server detection
// ---------------------------------------------------------------------------

#[test]
fn detect_server_path_returns_a_path_or_none() {
    use super::binary::detect_server_path;
    // On CI / dev machines, tuitbot-server is typically NOT on PATH so we get None.
    // If it *is* installed the path should be a real file.
    match detect_server_path() {
        Some(path) => assert!(path.is_file(), "detected path should be a file: {path:?}"),
        None => {} // expected in most environments
    }
}
