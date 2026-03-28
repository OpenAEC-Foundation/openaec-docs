//! User models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a user.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
  pub id: Uuid,
  pub sub: String,
  pub email: String,
  pub name: String,
  pub avatar_url: Option<String>,
  pub tenant: String,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// API response for a user.
#[derive(Debug, Serialize)]
pub struct UserResponse {
  pub user_id: Uuid,
  pub email: String,
  pub name: String,
  pub tenant: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub avatar_url: Option<String>,
}

impl From<UserRow> for UserResponse {
  fn from(row: UserRow) -> Self {
    Self {
      user_id: row.id,
      email: row.email,
      name: row.name,
      tenant: row.tenant,
      avatar_url: row.avatar_url,
    }
  }
}

/// OIDC claims used for user provisioning.
#[derive(Debug, Deserialize)]
pub struct OidcUserClaims {
  pub sub: String,
  pub email: String,
  pub name: String,
  pub avatar_url: Option<String>,
  pub tenant: String,
}
