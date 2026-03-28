-- OpenAEC Docs — Initial schema
-- Phase 1: users, projects, project_members, directory_configs, activity_log

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ═══════════════════════════════════════════════════════════════
-- Users (JIT provisioned via OIDC)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE users (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sub        TEXT UNIQUE NOT NULL,
  email      TEXT NOT NULL,
  name       TEXT NOT NULL,
  avatar_url TEXT,
  tenant     TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_tenant ON users (tenant);

-- ═══════════════════════════════════════════════════════════════
-- Projects (linked to a Nextcloud project folder)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE projects (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant           TEXT NOT NULL,
  name             TEXT NOT NULL,
  nextcloud_folder TEXT NOT NULL,
  description      TEXT NOT NULL DEFAULT '',
  status           TEXT NOT NULL DEFAULT 'active'
                   CHECK (status IN ('active', 'archived', 'template')),
  created_by       UUID REFERENCES users (id),
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant, nextcloud_folder)
);

CREATE INDEX idx_projects_tenant ON projects (tenant);
CREATE INDEX idx_projects_status ON projects (tenant, status);

-- ═══════════════════════════════════════════════════════════════
-- Project membership + roles
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE project_members (
  project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  role       TEXT NOT NULL DEFAULT 'member'
             CHECK (role IN ('owner', 'admin', 'member', 'viewer')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (project_id, user_id)
);

-- ═══════════════════════════════════════════════════════════════
-- Directory configurations per project
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE directory_configs (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id          UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  path                TEXT NOT NULL,
  display_name        TEXT NOT NULL,
  sort_order          INTEGER NOT NULL DEFAULT 0,
  icon                TEXT NOT NULL DEFAULT 'folder',
  allowed_extensions  JSONB NOT NULL DEFAULT '[]',
  read_only           BOOLEAN NOT NULL DEFAULT false,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (project_id, path)
);

CREATE INDEX idx_dirconfigs_project ON directory_configs (project_id);

-- ═══════════════════════════════════════════════════════════════
-- Directory templates (reusable presets per tenant)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE directory_templates (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant      TEXT NOT NULL,
  name        TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  directories JSONB NOT NULL,
  created_by  UUID REFERENCES users (id),
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant, name)
);

-- ═══════════════════════════════════════════════════════════════
-- Activity log (audit trail)
-- ═══════════════════════════════════════════════════════════════

CREATE TABLE activity_log (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id  UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
  user_id     UUID REFERENCES users (id),
  action      TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id   UUID,
  details     JSONB NOT NULL DEFAULT '{}',
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_activity_project ON activity_log (project_id, created_at DESC);
CREATE INDEX idx_activity_user ON activity_log (user_id, created_at DESC);
