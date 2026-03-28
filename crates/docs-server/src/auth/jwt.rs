//! Session JWT creation and validation.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SESSION_TTL_HOURS: i64 = 24;

/// Claims embedded in the session JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionClaims {
  pub sub: Uuid,
  pub email: String,
  pub name: String,
  pub tenant: String,
  pub iat: i64,
  pub exp: i64,
}

/// Create a signed session JWT.
pub fn create_session_token(
  user_id: Uuid,
  email: &str,
  name: &str,
  tenant: &str,
  secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
  let now = Utc::now();
  let claims = SessionClaims {
    sub: user_id,
    email: email.to_string(),
    name: name.to_string(),
    tenant: tenant.to_string(),
    iat: now.timestamp(),
    exp: (now + Duration::hours(SESSION_TTL_HOURS)).timestamp(),
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
  )
}

/// Validate a session JWT and return the claims.
pub fn validate_session_token(
  token: &str,
  secret: &str,
) -> Result<SessionClaims, jsonwebtoken::errors::Error> {
  let data = decode::<SessionClaims>(
    token,
    &DecodingKey::from_secret(secret.as_bytes()),
    &Validation::default(),
  )?;
  Ok(data.claims)
}
