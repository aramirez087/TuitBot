# Session 03 — DM Endpoint Matrix

**Date:** 2026-02-26
**Branch:** `feat/mcp_x_api_coverage`
**Spec Version:** 1.1.0

---

## DM Read Endpoints (5)

| Tool Name | Method | Path | Scopes | Profiles |
|-----------|--------|------|--------|----------|
| `x_v2_dm_conversations` | GET | `/2/dm_conversations` | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_conversation_by_id` | GET | `/2/dm_conversations/{id}` | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_events_by_conversation` | GET | `/2/dm_conversations/{id}/dm_events` | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_events_by_participant` | GET | `/2/dm_conversations/with/{participant_id}/dm_events` | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |
| `x_v2_dm_events` | GET | `/2/dm_events` | `dm.read`, `users.read` | ApiRO, Write, Admin, UtilWrite |

## DM Write Endpoints (3)

| Tool Name | Method | Path | Scopes | Profiles |
|-----------|--------|------|--------|----------|
| `x_v2_dm_send_in_conversation` | POST | `/2/dm_conversations/{id}/messages` | `dm.write`, `dm.read`, `users.read` | Write, Admin, UtilWrite |
| `x_v2_dm_send_to_participant` | POST | `/2/dm_conversations/with/{participant_id}/messages` | `dm.write`, `dm.read`, `users.read` | Write, Admin, UtilWrite |
| `x_v2_dm_create_group` | POST | `/2/dm_conversations` | `dm.write`, `dm.read`, `users.read` | Write, Admin, UtilWrite |

## Parameter Summary

### Common Read Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes (by-id tools) | Conversation ID |
| `participant_id` | string | Yes (by-participant tools) | User ID of DM participant |
| `max_results` | integer | No | Maximum results (default: 100) |
| `pagination_token` | string | No | Token for pagination |
| `dm_event_fields` | string[] | No | DM event fields (id, text, event_type, created_at, sender_id) |
| `expansions` | string[] | No | Expansion fields |

### Common Write Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes (in-conversation) | Conversation ID |
| `participant_id` | string | Yes (to-participant) | Target user ID |
| `participant_ids` | string[] | Yes (create-group) | Group participant user IDs |
| `text` | string | Yes | Message text content |
| `attachments` | string[] | No | Media IDs to attach |

## Profile Distribution

| Profile | DM Reads | DM Writes | Total DM Tools |
|---------|----------|-----------|----------------|
| Readonly | 0 | 0 | 0 |
| ApiReadonly | 5 | 0 | 5 |
| Write | 5 | 3 | 8 |
| Admin | 5 | 3 | 8 |
| UtilityReadonly | 0 | 0 | 0 |
| UtilityWrite | 5 | 3 | 8 |

## Safety Properties

- All DM reads: `Lane::Shared`, `requires_db: false`, error codes: `X_READ_ERR`
- All DM writes: `Lane::Shared` (utility profiles present), `requires_db: false` (via utility bypass), error codes: `X_WRITE_ERR`
- DM writes in Write/Admin profiles route through policy gateway at runtime (server handler layer)
- DM writes in UtilityWrite profiles bypass policy (consistent with existing utility profile behavior)
- 3 DM mutations added to boundary test denylist
- `dm.read` and `dm.write` added to `REQUIRED_SCOPES` for OAuth flow
- DM scope diagnostics added to `FEATURE_SCOPE_MAP` for degraded-feature reporting
