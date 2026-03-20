//! Tests for json_to_toml, merge_toml, and redact_service_account_keys.

use crate::routes::settings::validation::{json_to_toml, merge_toml, redact_service_account_keys};

// ── json_to_toml tests ────────────────────────────────────────────

#[test]
fn json_to_toml_string() {
    let json = serde_json::json!("hello");
    let toml = json_to_toml(&json).unwrap();
    assert_eq!(toml, toml::Value::String("hello".to_string()));
}

#[test]
fn json_to_toml_integer() {
    let json = serde_json::json!(42);
    let toml = json_to_toml(&json).unwrap();
    assert_eq!(toml, toml::Value::Integer(42));
}

#[test]
fn json_to_toml_float() {
    let json = serde_json::json!(3.14);
    let toml = json_to_toml(&json).unwrap();
    assert_eq!(toml, toml::Value::Float(3.14));
}

#[test]
fn json_to_toml_bool() {
    let json = serde_json::json!(true);
    let toml = json_to_toml(&json).unwrap();
    assert_eq!(toml, toml::Value::Boolean(true));
}

#[test]
fn json_to_toml_null_in_array_rejected() {
    let json = serde_json::json!(null);
    let err = json_to_toml(&json);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("null"));
}

#[test]
fn json_to_toml_object_skips_null_values() {
    let json = serde_json::json!({
        "key": "value",
        "null_key": null,
        "number": 1
    });
    let toml = json_to_toml(&json).unwrap();
    let table = toml.as_table().unwrap();
    assert_eq!(table.len(), 2);
    assert!(table.contains_key("key"));
    assert!(table.contains_key("number"));
    assert!(!table.contains_key("null_key"));
}

#[test]
fn json_to_toml_array() {
    let json = serde_json::json!(["a", "b", "c"]);
    let toml = json_to_toml(&json).unwrap();
    let arr = toml.as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn json_to_toml_nested_object() {
    let json = serde_json::json!({"outer": {"inner": "value"}});
    let toml = json_to_toml(&json).unwrap();
    let outer = toml.get("outer").unwrap().as_table().unwrap();
    assert_eq!(
        outer.get("inner").unwrap(),
        &toml::Value::String("value".to_string())
    );
}

#[test]
fn json_to_toml_empty_object() {
    let json = serde_json::json!({});
    let toml = json_to_toml(&json).unwrap();
    assert!(toml.as_table().unwrap().is_empty());
}

#[test]
fn json_to_toml_empty_array() {
    let json = serde_json::json!([]);
    let toml = json_to_toml(&json).unwrap();
    assert!(toml.as_array().unwrap().is_empty());
}

#[test]
fn json_to_toml_deeply_nested() {
    let json = serde_json::json!({"a": {"b": {"c": "deep"}}});
    let toml = json_to_toml(&json).unwrap();
    let val = toml.get("a").unwrap().get("b").unwrap().get("c").unwrap();
    assert_eq!(val, &toml::Value::String("deep".to_string()));
}

#[test]
fn json_to_toml_null_in_array_member_rejected() {
    let json = serde_json::json!(["a", null, "b"]);
    assert!(json_to_toml(&json).is_err());
}

#[test]
fn json_to_toml_mixed_array() {
    let json = serde_json::json!([1, 2, 3]);
    let toml = json_to_toml(&json).unwrap();
    let arr = toml.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], toml::Value::Integer(1));
}

#[test]
fn json_to_toml_boolean_false() {
    let json = serde_json::json!(false);
    assert_eq!(json_to_toml(&json).unwrap(), toml::Value::Boolean(false));
}

#[test]
fn json_to_toml_negative_integer() {
    let json = serde_json::json!(-42);
    assert_eq!(json_to_toml(&json).unwrap(), toml::Value::Integer(-42));
}

#[test]
fn json_to_toml_zero() {
    let json = serde_json::json!(0);
    assert_eq!(json_to_toml(&json).unwrap(), toml::Value::Integer(0));
}

#[test]
fn json_to_toml_negative_float() {
    let json = serde_json::json!(-3.14);
    assert_eq!(json_to_toml(&json).unwrap(), toml::Value::Float(-3.14));
}

#[test]
fn json_to_toml_large_integer() {
    let json = serde_json::json!(i64::MAX);
    assert_eq!(json_to_toml(&json).unwrap(), toml::Value::Integer(i64::MAX));
}

#[test]
fn json_to_toml_nested_array_of_objects() {
    let json = serde_json::json!([{"name": "a", "value": 1}, {"name": "b", "value": 2}]);
    let toml = json_to_toml(&json).unwrap();
    let arr = toml.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(
        arr[0].get("name").unwrap(),
        &toml::Value::String("a".to_string())
    );
}

#[test]
fn json_to_toml_array_of_arrays() {
    let json = serde_json::json!([[1, 2], [3, 4]]);
    let toml = json_to_toml(&json).unwrap();
    let arr = toml.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0].as_array().unwrap().len(), 2);
}

#[test]
fn json_to_toml_object_with_all_null_values() {
    let json = serde_json::json!({"a": null, "b": null});
    let toml = json_to_toml(&json).unwrap();
    assert!(toml.as_table().unwrap().is_empty());
}

#[test]
fn json_to_toml_string_with_special_chars() {
    let json = serde_json::json!("hello\nworld\t!");
    assert_eq!(
        json_to_toml(&json).unwrap(),
        toml::Value::String("hello\nworld\t!".to_string())
    );
}

#[test]
fn json_to_toml_empty_string() {
    let json = serde_json::json!("");
    assert_eq!(
        json_to_toml(&json).unwrap(),
        toml::Value::String(String::new())
    );
}

#[test]
fn json_to_toml_single_element_array() {
    let json = serde_json::json!(["only"]);
    let toml = json_to_toml(&json).unwrap();
    let arr = toml.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], toml::Value::String("only".to_string()));
}

// ── merge_toml tests ──────────────────────────────────────────────

#[test]
fn merge_toml_overwrites_scalar() {
    let mut base: toml::Value = toml::from_str(r#"key = "old""#).unwrap();
    let patch: toml::Value = toml::from_str(r#"key = "new""#).unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(
        base.get("key").unwrap(),
        &toml::Value::String("new".to_string())
    );
}

#[test]
fn merge_toml_adds_new_key() {
    let mut base: toml::Value = toml::from_str(r#"existing = 1"#).unwrap();
    let patch: toml::Value = toml::from_str(r#"new_key = 2"#).unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(base.get("new_key").unwrap(), &toml::Value::Integer(2));
    assert_eq!(base.get("existing").unwrap(), &toml::Value::Integer(1));
}

#[test]
fn merge_toml_deep_merge_tables() {
    let mut base: toml::Value = toml::from_str("[section]\na = 1\nb = 2").unwrap();
    let patch: toml::Value = toml::from_str("[section]\nb = 3\nc = 4").unwrap();
    merge_toml(&mut base, &patch);
    let section = base.get("section").unwrap().as_table().unwrap();
    assert_eq!(section.get("a").unwrap(), &toml::Value::Integer(1));
    assert_eq!(section.get("b").unwrap(), &toml::Value::Integer(3));
    assert_eq!(section.get("c").unwrap(), &toml::Value::Integer(4));
}

#[test]
fn merge_toml_scalar_replaces_table() {
    let mut base: toml::Value = toml::from_str("[section]\nkey = \"value\"").unwrap();
    let patch: toml::Value = toml::from_str(r#"section = "scalar""#).unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(
        base.get("section").unwrap(),
        &toml::Value::String("scalar".to_string())
    );
}

#[test]
fn merge_toml_empty_patch_no_change() {
    let mut base: toml::Value = toml::from_str(r#"key = "value""#).unwrap();
    let patch: toml::Value = toml::from_str("").unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(
        base.get("key").unwrap(),
        &toml::Value::String("value".to_string())
    );
}

#[test]
fn merge_toml_multiple_new_keys() {
    let mut base: toml::Value = toml::from_str("a = 1").unwrap();
    let patch: toml::Value = toml::from_str("b = 2\nc = 3\nd = 4").unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(base.get("a").unwrap(), &toml::Value::Integer(1));
    assert_eq!(base.get("b").unwrap(), &toml::Value::Integer(2));
    assert_eq!(base.get("c").unwrap(), &toml::Value::Integer(3));
    assert_eq!(base.get("d").unwrap(), &toml::Value::Integer(4));
}

#[test]
fn merge_toml_deeply_nested_tables() {
    let mut base: toml::Value = toml::from_str("[a]\n[a.b]\n[a.b.c]\nd = 1").unwrap();
    let patch: toml::Value = toml::from_str("[a.b.c]\nd = 2\ne = 3").unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(base["a"]["b"]["c"]["d"], toml::Value::Integer(2));
    assert_eq!(base["a"]["b"]["c"]["e"], toml::Value::Integer(3));
}

#[test]
fn merge_toml_array_replacement() {
    let mut base: toml::Value = toml::from_str("arr = [1, 2, 3]").unwrap();
    let patch: toml::Value = toml::from_str("arr = [4, 5]").unwrap();
    merge_toml(&mut base, &patch);
    let arr = base.get("arr").unwrap().as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], toml::Value::Integer(4));
}

#[test]
fn merge_toml_bool_overwrite() {
    let mut base: toml::Value = toml::from_str("flag = true").unwrap();
    let patch: toml::Value = toml::from_str("flag = false").unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(base.get("flag").unwrap(), &toml::Value::Boolean(false));
}

#[test]
fn merge_toml_string_overwrite() {
    let mut base: toml::Value = toml::from_str(r#"name = "old""#).unwrap();
    let patch: toml::Value = toml::from_str(r#"name = "new""#).unwrap();
    merge_toml(&mut base, &patch);
    assert_eq!(
        base.get("name").unwrap(),
        &toml::Value::String("new".to_string())
    );
}

// ── redact_service_account_keys tests ─────────────────────────────

#[test]
fn redact_service_account_keys_replaces_values() {
    let mut json = serde_json::json!({
        "content_sources": {
            "sources": [
                { "path": "/a", "service_account_key": "secret123" },
                { "path": "/b", "service_account_key": null },
                { "path": "/c" }
            ]
        }
    });
    redact_service_account_keys(&mut json);
    let sources = json["content_sources"]["sources"].as_array().unwrap();
    assert_eq!(sources[0]["service_account_key"], "[redacted]");
    assert!(sources[1]["service_account_key"].is_null());
    assert!(sources[2].get("service_account_key").is_none());
}

#[test]
fn redact_service_account_keys_no_sources() {
    let mut json = serde_json::json!({"business": {"name": "test"}});
    redact_service_account_keys(&mut json);
}

#[test]
fn redact_service_account_keys_empty_sources_array() {
    let mut json = serde_json::json!({"content_sources": {"sources": []}});
    redact_service_account_keys(&mut json);
    assert!(json["content_sources"]["sources"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn redact_service_account_keys_missing_content_sources() {
    let mut json = serde_json::json!({"other": "data"});
    redact_service_account_keys(&mut json);
    assert_eq!(json["other"], "data");
}

#[test]
fn redact_service_account_keys_non_string_value() {
    let mut json = serde_json::json!({
        "content_sources": {"sources": [{"service_account_key": 12345}]}
    });
    redact_service_account_keys(&mut json);
    assert_eq!(
        json["content_sources"]["sources"][0]["service_account_key"],
        "[redacted]"
    );
}

#[test]
fn redact_service_account_keys_multiple_sources() {
    let mut json = serde_json::json!({
        "content_sources": {"sources": [
            {"service_account_key": "s1"},
            {"service_account_key": "s2"},
            {"service_account_key": "s3"}
        ]}
    });
    redact_service_account_keys(&mut json);
    for s in json["content_sources"]["sources"].as_array().unwrap() {
        assert_eq!(s["service_account_key"], "[redacted]");
    }
}
