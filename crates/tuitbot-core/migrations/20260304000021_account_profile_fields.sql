-- Add display name and avatar URL columns to accounts for profile display.
ALTER TABLE accounts ADD COLUMN x_display_name TEXT;
ALTER TABLE accounts ADD COLUMN x_avatar_url TEXT;
