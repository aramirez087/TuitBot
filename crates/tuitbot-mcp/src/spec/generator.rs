//! Generator pipeline: converts [`EndpointDef`] specs into manifest [`ToolEntry`] records
//! and JSON Schema descriptions for each generated tool.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::manifest::{Lane, ToolEntry};

use super::endpoints::SPEC_ENDPOINTS;
use super::params::{EndpointDef, HttpMethod, ParamType};

/// Generate [`ToolEntry`] records for every endpoint in the spec pack.
///
/// The output is sorted alphabetically by tool name for determinism.
/// These entries are designed to be appended to the curated tools in
/// `manifest::all_tools()`.
pub fn generate_spec_tools() -> Vec<ToolEntry> {
    let mut tools: Vec<ToolEntry> = SPEC_ENDPOINTS.iter().map(endpoint_to_tool_entry).collect();
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

/// Generate JSON Schema descriptions for all spec-defined tools.
///
/// Returns a sorted vec of `(tool_name, schema_json)` pairs.
/// Deterministic output — same input always produces same output.
pub fn generate_tool_schemas() -> Vec<ToolSchema> {
    let mut schemas: Vec<ToolSchema> = SPEC_ENDPOINTS.iter().map(endpoint_to_schema).collect();
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    schemas
}

/// JSON Schema description for a generated tool.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub method: String,
    pub path: String,
    pub api_version: String,
    pub group: String,
    pub scopes: Vec<String>,
    pub input_schema: Value,
}

/// Convert one endpoint definition into a manifest [`ToolEntry`].
fn endpoint_to_tool_entry(ep: &EndpointDef) -> ToolEntry {
    ToolEntry {
        name: ep.tool_name.to_owned(),
        category: ep.category,
        lane: if ep.method.is_mutation() {
            Lane::Workflow
        } else {
            Lane::Shared
        },
        mutation: ep.method.is_mutation(),
        requires_x_client: true,
        requires_llm: false,
        requires_db: ep.method.is_mutation(),
        requires_scopes: ep.scopes.iter().map(|s| (*s).to_string()).collect(),
        requires_user_auth: true,
        requires_elevated_access: false,
        profiles: ep.profiles.to_vec(),
        possible_error_codes: ep.error_codes.to_vec(),
    }
}

/// Convert one endpoint definition into a [`ToolSchema`] with JSON Schema input.
fn endpoint_to_schema(ep: &EndpointDef) -> ToolSchema {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for param in ep.params {
        let mut prop = serde_json::Map::new();
        prop.insert(
            "description".to_owned(),
            Value::String(param.description.to_owned()),
        );

        match param.param_type {
            ParamType::String => {
                prop.insert("type".to_owned(), Value::String("string".to_owned()));
            }
            ParamType::Integer => {
                prop.insert("type".to_owned(), Value::String("integer".to_owned()));
            }
            ParamType::Boolean => {
                prop.insert("type".to_owned(), Value::String("boolean".to_owned()));
            }
            ParamType::StringArray => {
                prop.insert("type".to_owned(), Value::String("string".to_owned()));
                prop.insert(
                    "format".to_owned(),
                    Value::String("comma-separated".to_owned()),
                );
            }
        }

        if let Some(default) = param.default {
            prop.insert("default".to_owned(), Value::String(default.to_owned()));
        }

        if param.required {
            required.push(Value::String(param.name.to_owned()));
        }

        properties.insert(param.name.to_owned(), Value::Object(prop));
    }

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_owned(), Value::String("object".to_owned()));
    schema.insert("properties".to_owned(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".to_owned(), Value::Array(required));
    }
    schema.insert("additionalProperties".to_owned(), Value::Bool(false));

    let method_str = match ep.method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
    };

    ToolSchema {
        name: ep.tool_name.to_owned(),
        description: ep.description.to_owned(),
        method: method_str.to_owned(),
        path: ep.path.to_owned(),
        api_version: ep.api_version.to_owned(),
        group: ep.group.to_owned(),
        scopes: ep.scopes.iter().map(|s| (*s).to_owned()).collect(),
        input_schema: Value::Object(schema),
    }
}
