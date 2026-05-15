CREATE TABLE media (
  id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  key          TEXT        UNIQUE NOT NULL,
  url          TEXT        NOT NULL,
  filename     TEXT        NOT NULL,
  content_type TEXT        NOT NULL,
  size         BIGINT      NOT NULL DEFAULT 0,
  label        TEXT,
  uploaded_by  UUID        REFERENCES users(id) ON DELETE SET NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ON media (created_at DESC);
CREATE INDEX ON media (uploaded_by);
