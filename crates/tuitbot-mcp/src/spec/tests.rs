//! Tests for the spec pack and generator pipeline.

use std::collections::HashSet;

use super::endpoints::SPEC_ENDPOINTS;
use super::generator::{generate_spec_tools, generate_tool_schemas};
use super::params::HttpMethod;
use super::X_API_SPEC_VERSION;

#[test]
fn spec_version_is_valid_semver() {
    let parts: Vec<&str> = X_API_SPEC_VERSION.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "spec version must be semver: {X_API_SPEC_VERSION}"
    );
    for part in &parts {
        part.parse::<u32>()
            .unwrap_or_else(|_| panic!("invalid semver component: {part}"));
    }
}

#[test]
fn spec_has_expected_endpoint_count() {
    assert_eq!(
        SPEC_ENDPOINTS.len(),
        36,
        "expected 36 spec endpoints (charter defines 36 new tools)"
    );
}

#[test]
fn no_duplicate_tool_names_in_spec() {
    let mut seen = HashSet::new();
    for ep in SPEC_ENDPOINTS {
        assert!(
            seen.insert(ep.tool_name),
            "duplicate spec tool name: {}",
            ep.tool_name
        );
    }
}

#[test]
fn all_tool_names_follow_naming_convention() {
    for ep in SPEC_ENDPOINTS {
        assert!(
            ep.tool_name.starts_with("x_v2_"),
            "tool {} must start with x_v2_",
            ep.tool_name
        );
    }
}

#[test]
fn all_paths_start_with_slash() {
    for ep in SPEC_ENDPOINTS {
        assert!(
            ep.path.starts_with('/'),
            "tool {} path must start with /: {}",
            ep.tool_name,
            ep.path
        );
    }
}

#[test]
fn mutations_are_post_put_or_delete() {
    for ep in SPEC_ENDPOINTS {
        if ep.method.is_mutation() {
            assert!(
                matches!(
                    ep.method,
                    HttpMethod::Post | HttpMethod::Put | HttpMethod::Delete
                ),
                "tool {} marked as mutation but method is GET",
                ep.tool_name
            );
        }
    }
}

#[test]
fn all_endpoints_have_scopes() {
    for ep in SPEC_ENDPOINTS {
        assert!(
            !ep.scopes.is_empty(),
            "tool {} must declare at least one OAuth scope",
            ep.tool_name
        );
    }
}

#[test]
fn all_endpoints_have_error_codes() {
    for ep in SPEC_ENDPOINTS {
        assert!(
            !ep.error_codes.is_empty(),
            "tool {} must have at least one error code",
            ep.tool_name
        );
    }
}

#[test]
fn all_endpoints_have_at_least_one_profile() {
    for ep in SPEC_ENDPOINTS {
        assert!(
            !ep.profiles.is_empty(),
            "tool {} must be assigned to at least one profile",
            ep.tool_name
        );
    }
}

#[test]
fn generator_produces_correct_count() {
    let tools = generate_spec_tools();
    assert_eq!(tools.len(), SPEC_ENDPOINTS.len());
}

#[test]
fn generator_output_is_sorted() {
    let tools = generate_spec_tools();
    for pair in tools.windows(2) {
        assert!(
            pair[0].name <= pair[1].name,
            "tools not sorted: {} > {}",
            pair[0].name,
            pair[1].name
        );
    }
}

#[test]
fn generator_output_is_deterministic() {
    let run1 = generate_spec_tools();
    let run2 = generate_spec_tools();
    assert_eq!(run1.len(), run2.len());
    for (a, b) in run1.iter().zip(run2.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.category, b.category);
        assert_eq!(a.lane, b.lane);
        assert_eq!(a.mutation, b.mutation);
    }
}

#[test]
fn schema_generator_produces_valid_schemas() {
    let schemas = generate_tool_schemas();
    assert_eq!(schemas.len(), SPEC_ENDPOINTS.len());
    for schema in &schemas {
        assert!(!schema.name.is_empty());
        assert!(!schema.description.is_empty());
        assert!(!schema.method.is_empty());
        assert!(!schema.path.is_empty());
        assert!(!schema.scopes.is_empty());
        // input_schema must be a JSON object
        assert!(
            schema.input_schema.is_object(),
            "schema for {} is not an object",
            schema.name
        );
    }
}

#[test]
fn schema_output_is_deterministic() {
    let json1 = serde_json::to_string_pretty(&generate_tool_schemas()).unwrap();
    let json2 = serde_json::to_string_pretty(&generate_tool_schemas()).unwrap();
    assert_eq!(json1, json2, "schema generation is not deterministic");
}

#[test]
fn mutation_tools_require_db_in_generated_entries() {
    let tools = generate_spec_tools();
    for t in &tools {
        if t.mutation {
            assert!(
                t.requires_db,
                "mutation tool {} should require db for policy/idempotency",
                t.name
            );
        }
    }
}

#[test]
fn read_tools_are_shared_lane() {
    let tools = generate_spec_tools();
    for t in &tools {
        if !t.mutation {
            assert_eq!(
                t.lane,
                crate::tools::manifest::Lane::Shared,
                "read tool {} should be in shared lane",
                t.name
            );
        }
    }
}

#[test]
fn all_api_versions_are_v2() {
    for ep in SPEC_ENDPOINTS {
        assert_eq!(
            ep.api_version, "v2",
            "tool {} uses non-v2 api_version: {}",
            ep.tool_name, ep.api_version
        );
    }
}

#[test]
fn groups_match_expected_set() {
    let valid_groups: HashSet<&str> = ["tweets", "users", "lists", "mutes", "blocks", "spaces"]
        .iter()
        .copied()
        .collect();
    for ep in SPEC_ENDPOINTS {
        assert!(
            valid_groups.contains(ep.group),
            "tool {} has unexpected group: {}",
            ep.tool_name,
            ep.group
        );
    }
}

#[test]
fn category_distribution() {
    let tools = generate_spec_tools();
    let mut cats: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for t in &tools {
        let cat = serde_json::to_string(&t.category).unwrap();
        *cats.entry(cat).or_default() += 1;
    }
    // We should have at least Read, List, Moderation, Engage categories
    assert!(
        cats.len() >= 3,
        "expected at least 3 categories in generated tools"
    );
}
