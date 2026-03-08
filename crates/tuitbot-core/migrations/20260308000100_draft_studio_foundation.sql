-- Draft Studio foundation: additive columns on scheduled_content + supporting tables.

-- 1. Additive columns on scheduled_content
ALTER TABLE scheduled_content ADD COLUMN title TEXT DEFAULT NULL;
ALTER TABLE scheduled_content ADD COLUMN notes TEXT DEFAULT NULL;
ALTER TABLE scheduled_content ADD COLUMN archived_at TEXT DEFAULT NULL;

-- 2. Revision snapshots
CREATE TABLE IF NOT EXISTS content_revisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_id INTEGER NOT NULL REFERENCES scheduled_content(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL,
    content TEXT NOT NULL,
    content_type TEXT NOT NULL,
    trigger_kind TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_content_revisions_content_id ON content_revisions(content_id);

-- 3. Tags
CREATE TABLE IF NOT EXISTS content_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT DEFAULT NULL,
    UNIQUE(account_id, name)
);

-- 4. Tag assignments (many-to-many)
CREATE TABLE IF NOT EXISTS content_tag_assignments (
    content_id INTEGER NOT NULL REFERENCES scheduled_content(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES content_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (content_id, tag_id)
);

-- 5. Activity log
CREATE TABLE IF NOT EXISTS content_activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_id INTEGER NOT NULL REFERENCES scheduled_content(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL,
    action TEXT NOT NULL,
    detail TEXT DEFAULT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_content_activity_content_id ON content_activity(content_id);
