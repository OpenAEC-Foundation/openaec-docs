//! Authentication route handlers (OIDC login flow).

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::jwt::create_session_token;
use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::user::UserResponse;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/auth/login", get(login))
    .route("/auth/callback", get(callback))
    .route("/auth/me", get(me))
}

async fn login(State(state): State<AppState>) -> Result<Response, AppError> {
  let oidc = state
    .oidc_client
    .as_ref()
    .ok_or_else(|| AppError::Internal("OIDC not configured".to_string()))?;

  let (auth_url, csrf_state, nonce, pkce_verifier) = oidc.authorize_url();

  state.pending_auth.write().await.insert(
    csrf_state,
    PendingAuth {
      pkce_verifier,
      nonce,
    },
  );

  Ok(Redirect::temporary(&auth_url).into_response())
}

#[derive(Debug, Deserialize)]
struct CallbackParams {
  code: String,
  state: String,
}

async fn callback(
  State(state): State<AppState>,
  Query(params): Query<CallbackParams>,
) -> Result<Response, AppError> {
  let oidc = state
    .oidc_client
    .as_ref()
    .ok_or_else(|| AppError::Internal("OIDC not configured".to_string()))?;

  let pending = state
    .pending_auth
    .write()
    .await
    .remove(&params.state)
    .ok_or(AppError::BadRequest(
      "invalid or expired state parameter".to_string(),
    ))?;

  let claims = oidc
    .exchange_code(&params.code, &pending.pkce_verifier, &pending.nonce)
    .await
    .map_err(|e| AppError::Internal(format!("OIDC token exchange failed: {e}")))?;

  let user = db::users::upsert_from_oidc(&state.pool, &claims)
    .await
    .map_err(|e| AppError::Internal(format!("user provisioning failed: {e}")))?;

  tracing::info!(user_id = %user.id, email = %user.email, tenant = %user.tenant, "user authenticated");

  let token = create_session_token(
    user.id,
    &user.email,
    &user.name,
    &user.tenant,
    &state.config.jwt_secret,
  )
  .map_err(|e| AppError::Internal(format!("failed to create session token: {e}")))?;

  let redirect_url = format!("{}?token={}", state.config.frontend_url, token);
  Ok(Redirect::temporary(&redirect_url).into_response())
}

async fn me(auth: AuthUser) -> AppResult<Json<UserResponse>> {
  Ok(Json(UserResponse {
    user_id: auth.user_id,
    email: auth.email,
    name: auth.name,
    tenant: auth.tenant,
    avatar_url: None,
  }))
}

/// Pending OIDC auth state.
pub struct PendingAuth {
  pub pkce_verifier: String,
  pub nonce: String,
}
