# PRDV1-03: Approval Queue Professionalization

## Goal

Upgrade approvals from a basic queue to a full governance workflow with context,
review attribution, edit history, and batch safeguards.

## Already built (do not redo)

- `approval_queue` table and endpoints exist.
- Dashboard approval page supports list, approve, reject, and edit content.
- Composer-mode mutation routing already sends items to approval queue.

## Gaps to implement

1. Approval item completeness:
   - final text
   - media
   - target
   - tool
   - `why` reason
   - score
   - detected risks
2. Review metadata and auditability:
   - `approved_by` / `rejected_by`
   - reviewer notes
   - edit history (who edited, when, before/after)
   - precise timestamps
3. Batch actions with safeguards:
   - max `N` per batch
   - confirmation payload includes impacted IDs and risks
4. Full mutation routing in composer mode:
   - no bypasses for mutation tools

## Primary code touchpoints

- `migrations/*approval*.sql`
- `crates/tuitbot-core/src/storage/approval_queue.rs`
- `crates/tuitbot-server/src/routes/approval.rs`
- `crates/tuitbot-mcp/src/tools/approval.rs`
- `crates/tuitbot-mcp/src/tools/policy_gate.rs`
- `dashboard/src/lib/api.ts`
- `dashboard/src/lib/stores/approval.ts`
- `dashboard/src/routes/(app)/approval/+page.svelte`
- `crates/tuitbot-core/src/storage/action_log.rs`

## Implementation tasks

1. Schema migration.
   - Add reviewer identity columns and notes.
   - Add risk/QA payload columns (JSON text).
   - Add `request_tool`, `request_why`, `batch_id`, `edited_at`, `edited_by`.
2. Storage layer updates.
   - New CRUD for notes, reviewer, edit audit.
   - Batch approve API with configurable max batch size.
3. API contract updates.
   - `POST /api/approval/{id}/approve` accepts actor + notes.
   - `POST /api/approval/{id}/reject` accepts actor + notes.
   - `POST /api/approval/approve-all` replaced or constrained by guarded batch endpoint.
4. MCP integration.
   - When routing via policy gate, pass “why” and risk placeholders.
5. Dashboard UX updates.
   - Show “why” + risk tags in row detail.
   - Force reason input on reject.
   - Batch selection with max-N validation.
6. Audit log integration.
   - Every approval/rejection/edit emits structured action log entry.

## Acceptance criteria

- Every approved/rejected item has actor + timestamp.
- Edit actions preserve traceability (before/after + editor).
- Batch approvals cannot exceed configured max.
- Composer mode mutation items always pass through approval queue.

## Verification commands

```bash
cargo test -p tuitbot-core approval_queue
cargo test -p tuitbot-server approval
cargo test -p tuitbot-mcp approval
```

## Out of scope

- Policy template logic (PRDV1-02).
- Language/brand QA logic (PRDV1-06).
