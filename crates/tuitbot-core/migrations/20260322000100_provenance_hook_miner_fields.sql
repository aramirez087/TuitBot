-- Hook Miner attribution fields for vault_provenance_links.
-- All nullable for backward compatibility with existing rows.
ALTER TABLE vault_provenance_links ADD COLUMN angle_kind TEXT;
ALTER TABLE vault_provenance_links ADD COLUMN signal_kind TEXT;
ALTER TABLE vault_provenance_links ADD COLUMN signal_text TEXT;
ALTER TABLE vault_provenance_links ADD COLUMN source_role TEXT;
