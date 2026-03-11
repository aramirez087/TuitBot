-- Preserve scheduling intent through the approval queue.
-- When a user schedules content that requires approval, the target time
-- is stored here. On approval the system bridges to scheduled_content
-- using this timestamp instead of posting immediately.
ALTER TABLE approval_queue ADD COLUMN scheduled_for TEXT DEFAULT NULL;
