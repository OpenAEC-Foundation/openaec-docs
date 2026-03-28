# OpenAEC Docs — BIM Document Management Platform

## Overzicht

BIM/bouw documentmanagement platform op `docs.open-aec.com`.
Nextcloud als storage backend, PostgreSQL voor metadata, Authentik SSO.
Integreert met BIM Validator en BCF Platform.

## Tech Stack

- **Backend:** Rust (Axum) — `crates/docs-server/`
- **Frontend:** React + TypeScript + Tailwind — `frontend/`
- **Database:** PostgreSQL 16
- **Storage:** Nextcloud WebDAV (geen eigen bestandsopslag)
- **Auth:** Authentik OIDC

## Architectuur

```
Browser (SPA) → Rust/Axum API → PostgreSQL (metadata)
                              → Nextcloud WebDAV (bestanden)
                              → BIM Validator API (validatie)
                              → BCF Platform API (issues)
```

- Multi-tenant: leest `/etc/openaec/tenants.json`, per-request NC client
- Auth: OIDC met PKCE, session JWT (24h)
- WebDAV proxy: frontend → backend → Nextcloud (geen CORS issues)

## Commando's

```bash
# Development
cargo run -p docs-server              # backend
cd frontend && npm run dev            # frontend

# Build
cargo build --release -p docs-server
cd frontend && npm run build

# Tests
cargo test -p docs-server
cd frontend && npm test

# Docker
docker compose up                     # dev (met lokale postgres)
docker compose -f docker-compose.prod.yml up  # productie
```

## Mapstructuur

```
crates/docs-server/src/
  main.rs              — Entry point, migrations, server boot
  config.rs            — Config::from_env()
  state.rs             — AppState (pool, config, tenants, oidc)
  error.rs             — AppError → HTTP status mapping
  tenant.rs            — TenantRegistry, per-tenant NC client
  auth/                — OIDC + JWT session tokens
  webdav/              — Enhanced Nextcloud WebDAV client
  db/                  — SQLx database queries
  models/              — Serde request/response structs
  routes/              — Axum route handlers
  integrations/        — HTTP clients voor BIM Validator + BCF Platform
migrations/            — SQL migrations (sequentieel genummerd)
frontend/              — React SPA
```

## Conventies

- Kopieer bewezen patronen van `openaec-bcf-platform` (auth, webdav, error handling)
- `SQLX_OFFLINE=true` voor compile-time checks zonder database
- Geen hardcoded Nextcloud paden — alles via tenant config
- Document metadata in PostgreSQL, bestanden in Nextcloud
- Alle API routes onder `/api/v1/`
