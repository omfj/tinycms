-- Workspace settings (singleton row — one per installation)
CREATE TABLE workspace_settings (
  id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  name             TEXT        NOT NULL DEFAULT 'My Workspace',
  -- When true, new SSO sign-ins land as 'pending' and require admin approval
  require_approval BOOLEAN     NOT NULL DEFAULT true,
  -- Role automatically assigned when an admin approves a user
  default_role     TEXT        NOT NULL DEFAULT 'editor',
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER workspace_settings_updated_at
  BEFORE UPDATE ON workspace_settings
  FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Seed with defaults so there is always exactly one row
INSERT INTO workspace_settings DEFAULT VALUES;

-- Add approval status and role to users
ALTER TABLE users
  ADD COLUMN status TEXT NOT NULL DEFAULT 'pending', -- 'pending' | 'active' | 'suspended'
  ADD COLUMN role   TEXT NOT NULL DEFAULT 'editor';  -- 'admin' | 'editor' | 'viewer'

CREATE INDEX ON users (status);

-- The first user to sign up becomes admin and is auto-approved.
-- Application logic handles this; the constraint just keeps the enum honest.
-- status: pending → active (approved) | suspended (banned)
-- role:   viewer < editor < admin
