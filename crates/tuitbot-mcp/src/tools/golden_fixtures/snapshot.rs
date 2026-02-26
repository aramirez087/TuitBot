use std::path::PathBuf;

use super::generation::generate_fixtures;
use super::GoldenFixtures;
use crate::tools::test_mocks::artifacts_dir;

#[tokio::test]
async fn golden_snapshot_matches() {
    let fixtures = generate_fixtures().await;
    let json = serde_json::to_string_pretty(&fixtures).unwrap();

    let dir = artifacts_dir();
    std::fs::create_dir_all(&dir).expect("create artifacts dir");
    let fixture_path = dir.join("session-09-golden-fixtures.json");

    if fixture_path.exists() {
        let existing = std::fs::read_to_string(&fixture_path).expect("read golden file");
        let existing_fixtures: GoldenFixtures =
            serde_json::from_str(&existing).expect("parse golden file");

        // Compare structural shapes (ignore generated timestamp)
        for (family_name, expected) in &existing_fixtures.families {
            let actual = fixtures
                .families
                .get(family_name)
                .unwrap_or_else(|| panic!("Missing family in generated fixtures: {family_name}"));
            assert_eq!(
                actual.data_keys, expected.data_keys,
                "Data keys drifted for family {family_name}"
            );
            assert_eq!(
                actual.sample_shape, expected.sample_shape,
                "Shape drifted for family {family_name}"
            );
            assert_eq!(
                actual.has_pagination, expected.has_pagination,
                "Pagination flag drifted for family {family_name}"
            );
        }

        // Also check no families were removed
        for family_name in fixtures.families.keys() {
            assert!(
                existing_fixtures.families.contains_key(family_name),
                "New family {family_name} not in golden file — update snapshot"
            );
        }
    } else {
        // First run: write the golden file
        std::fs::write(&fixture_path, &json).expect("write golden file");
    }

    // Always write the golden report
    write_golden_report(&fixtures, &dir);
}

fn write_golden_report(fixtures: &GoldenFixtures, dir: &PathBuf) {
    let mut md = String::from("# Session 09 — Schema Golden Report\n\n");
    md.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    ));
    md.push_str("| Family | Tools | Keys | Pagination | Status |\n");
    md.push_str("|--------|-------|------|------------|--------|\n");
    for (name, family) in &fixtures.families {
        md.push_str(&format!(
            "| {} | {} | {} | {} | PASS |\n",
            name,
            family.tools.len(),
            family.data_keys.len(),
            if family.has_pagination { "yes" } else { "no" },
        ));
    }
    md.push_str(&format!(
        "\n**Total families:** {}\n",
        fixtures.families.len()
    ));

    std::fs::write(dir.join("session-09-schema-golden-report.md"), &md)
        .expect("write golden report");
}
