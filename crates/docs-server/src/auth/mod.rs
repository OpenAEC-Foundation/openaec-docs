//! Authentication and authorization.
//!
//! Supports session JWT issued after OIDC login.
//! When `AUTH_ENABLED=false`, all routes are accessible without authentication.

pub mod jwt;
pub mod oidc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::db;
use crate::error::AppError;
use crate::state::AppState;

/// Authenticated user identity, extracted from the request.
#[derive(Debug, Clone)]
pub struct AuthUser {
  pub user_id: Uuid,
  pub email: String,
  pub name: String,
  pub tenant: String,
}

impl FromRequestParts<AppState> for AuthUser {
  type Rejection = AppError;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    if !state.config.auth_enabled {
      return Ok(AuthUser {
        user_id: Uuid::nil(),
        email: "anonymous@local".to_string(),
        name: "Anonymous".to_string(),
        tenant: String::new(),
      });
    }

    let auth_header = parts
      .headers
      .get("authorization")
      .and_then(|v| v.to_str().ok())
      .ok_or(AppError::Unauthorized)?;

    let token = auth_header
      .strip_prefix("Bearer ")
      .ok_or(AppError::Unauthorized)?;

    let claims = jwt::validate_session_token(token, &state.config.jwt_secret)
      .map_err(|_| AppError::Unauthorized)?;

    let user = db::users::find_by_id(&state.pool, claims.sub)
      .await
      .map_err(|_| AppError::Unauthorized)?
      .ok_or(AppError::Unauthorized)?;

    Ok(AuthUser {
      user_id: user.id,
      email: user.email,
      name: user.name,
      tenant: user.tenant,
    })
  }
}

/// Optional authentication extractor.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AppState> for OptionalAuthUser {
  type Rejection = std::convert::Infallible;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let user = AuthUser::from_request_parts(parts, state).await.ok();
    Ok(OptionalAuthUser(user))
  }
}
