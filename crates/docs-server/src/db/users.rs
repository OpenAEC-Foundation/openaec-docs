//! User database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{OidcUserClaims, UserRow};

/// Find a user by ID.
pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "SELECT id, sub, email, name, avatar_url, tenant, created_at, updated_at
     FROM users WHERE id = $1",
  )
  .bind(user_id)
  .fetch_optional(pool)
  .await
}

/// Upsert a user from OIDC claims (insert or update on conflict).
pub async fn upsert_from_oidc(
  pool: &PgPool,
  claims: &OidcUserClaims,
) -> Result<UserRow, sqlx::Error> {
  sqlx::query_as::<_, UserRow>(
    "INSERT INTO users (sub, email, name, avatar_url, tenant)
     VALUES ($1, $2, $3, $4, $5)
     ON CONFLICT (sub) DO UPDATE
       SET email = EXCLUDED.email,
           name = EXCLUDED.name,
           avatar_url = EXCLUDED.avatar_url,
           tenant = EXCLUDED.tenant,
           updated_at = now()
     RETURNING id, sub, email, name, avatar_url, tenant, created_at, updated_at",
  )
  .bind(&claims.sub)
  .bind(&claims.email)
  .bind(&claims.name)
  .bind(&claims.avatar_url)
  .bind(&claims.tenant)
  .fetch_one(pool)
  .await
}
