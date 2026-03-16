//! Mutes and Blocks endpoint definitions (6 endpoints).

use crate::tools::manifest::ToolCategory;

use crate::spec::params::{EndpointDef, HttpMethod, ParamDef, ParamType};

use super::shared::{
    PARAM_MAX_RESULTS, PARAM_PAGINATION_TOKEN, PARAM_TARGET_USER_ID, PARAM_USER_FIELDS,
    PARAM_USER_ID, WRITE_UP_AND_API_RO_AND_UTIL_WRITE, WRITE_UP_AND_UTIL_WRITE, X_READ_ERR,
    X_WRITE_ERR,
};

pub(super) static USER_ENDPOINTS: &[EndpointDef] = &[
    // ── Mutes (3) ──────────────────────────────────────────────────
    EndpointDef {
        tool_name: "x_v2_mutes_list",
        description: "Get users you have muted",
        method: HttpMethod::Get,
        path: "/2/users/{user_id}/muting",
        category: ToolCategory::Moderation,
        profiles: WRITE_UP_AND_API_RO_AND_UTIL_WRITE,
        scopes: &["mute.read", "users.read"],
        params: &[
            PARAM_USER_ID,
            PARAM_MAX_RESULTS,
            PARAM_PAGINATION_TOKEN,
            PARAM_USER_FIELDS,
        ],
        error_codes: X_READ_ERR,
        api_version: "v2",
        group: "mutes",
        host: None,
    },
    EndpointDef {
        tool_name: "x_v2_mutes_create",
        description: "Mute a user",
        method: HttpMethod::Post,
        path: "/2/users/{user_id}/muting",
        category: ToolCategory::Moderation,
        profiles: WRITE_UP_AND_UTIL_WRITE,
        scopes: &["mute.write", "users.read"],
        params: &[
            PARAM_USER_ID,
            ParamDef {
                name: "target_user_id",
                param_type: ParamType::String,
                required: true,
                description: "ID of the user to mute",
                default: None,
            },
        ],
        error_codes: X_WRITE_ERR,
        api_version: "v2",
        group: "mutes",
        host: None,
    },
    EndpointDef {
        tool_name: "x_v2_mutes_delete",
        description: "Unmute a user",
        method: HttpMethod::Delete,
        path: "/2/users/{user_id}/muting/{target_user_id}",
        category: ToolCategory::Moderation,
        profiles: WRITE_UP_AND_UTIL_WRITE,
        scopes: &["mute.write", "users.read"],
        params: &[PARAM_USER_ID, PARAM_TARGET_USER_ID],
        error_codes: X_WRITE_ERR,
        api_version: "v2",
        group: "mutes",
        host: None,
    },
    // ── Blocks (3) ─────────────────────────────────────────────────
    EndpointDef {
        tool_name: "x_v2_blocks_list",
        description: "Get users you have blocked",
        method: HttpMethod::Get,
        path: "/2/users/{user_id}/blocking",
        category: ToolCategory::Moderation,
        profiles: WRITE_UP_AND_API_RO_AND_UTIL_WRITE,
        scopes: &["block.read", "users.read"],
        params: &[
            PARAM_USER_ID,
            PARAM_MAX_RESULTS,
            PARAM_PAGINATION_TOKEN,
            PARAM_USER_FIELDS,
        ],
        error_codes: X_READ_ERR,
        api_version: "v2",
        group: "blocks",
        host: None,
    },
    EndpointDef {
        tool_name: "x_v2_blocks_create",
        description: "Block a user",
        method: HttpMethod::Post,
        path: "/2/users/{user_id}/blocking",
        category: ToolCategory::Moderation,
        profiles: WRITE_UP_AND_UTIL_WRITE,
        scopes: &["block.write", "users.read"],
        params: &[
            PARAM_USER_ID,
            ParamDef {
                name: "target_user_id",
                param_type: ParamType::String,
                required: true,
                description: "ID of the user to block",
                default: None,
            },
        ],
        error_codes: X_WRITE_ERR,
        api_version: "v2",
        group: "blocks",
        host: None,
    },
    EndpointDef {
        tool_name: "x_v2_blocks_delete",
        description: "Unblock a user",
        method: HttpMethod::Delete,
        path: "/2/users/{user_id}/blocking/{target_user_id}",
        category: ToolCategory::Moderation,
        profiles: WRITE_UP_AND_UTIL_WRITE,
        scopes: &["block.write", "users.read"],
        params: &[PARAM_USER_ID, PARAM_TARGET_USER_ID],
        error_codes: X_WRITE_ERR,
        api_version: "v2",
        group: "blocks",
        host: None,
    },
];
