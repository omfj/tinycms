DROP INDEX IF EXISTS documents_slug_idx;
ALTER TABLE documents DROP COLUMN IF EXISTS slug;
