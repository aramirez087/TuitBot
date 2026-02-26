//! Golden fixture tests for tool response schema drift detection.
//!
//! Captures the structural shape of tool response `data` fields for each
//! tool family. On first run, generates a golden JSON file; on subsequent
//! runs, asserts the shape hasn't drifted.

#[cfg(test)]
mod family_tests;
#[cfg(test)]
mod generation;
#[cfg(test)]
mod snapshot;

#[cfg(test)]
pub(crate) use types::*;

#[cfg(test)]
mod types {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct GoldenFixtures {
        pub version: String,
        pub generated: String,
        pub families: BTreeMap<String, FixtureFamily>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct FixtureFamily {
        pub description: String,
        pub tools: Vec<String>,
        pub data_keys: Vec<String>,
        pub has_pagination: bool,
        pub sample_shape: Value,
    }

    pub fn extract_keys(val: &Value) -> Vec<String> {
        match val {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => vec![],
        }
    }

    pub fn shape_of(val: &Value) -> Value {
        match val {
            Value::Object(map) => {
                let shape: BTreeMap<String, Value> =
                    map.iter().map(|(k, v)| (k.clone(), shape_of(v))).collect();
                serde_json::to_value(shape).unwrap()
            }
            Value::Array(arr) => {
                if let Some(first) = arr.first() {
                    serde_json::json!([shape_of(first)])
                } else {
                    serde_json::json!([])
                }
            }
            Value::String(_) => serde_json::json!("string"),
            Value::Number(_) => serde_json::json!("number"),
            Value::Bool(_) => serde_json::json!("boolean"),
            Value::Null => serde_json::json!("null"),
        }
    }
}
