//! Application configuration loaded from environment variables.

/// Server, database, and integration configuration.
#[derive(Debug, Clone)]
pub struct Config {
  pub database_url: String,
  pub host: String,
  pub port: u16,
  /// Enable authentication middleware.
  pub auth_enabled: bool,
  /// OIDC issuer URL.
  pub oidc_issuer_url: Option<String>,
  /// OIDC client ID.
  pub oidc_client_id: Option<String>,
  /// OIDC client secret.
  pub oidc_client_secret: Option<String>,
  /// OIDC redirect URI.
  pub oidc_redirect_uri: Option<String>,
  /// Secret used to sign session JWTs.
  pub jwt_secret: String,
  /// Frontend URL to redirect to after OIDC callback.
  pub frontend_url: String,
  /// Path to tenants.json config file.
  pub tenants_config: Option<String>,
  /// BIM Validator base URL for integration.
  pub bim_validator_url: Option<String>,
  /// BCF Platform base URL for integration.
  pub bcf_platform_url: Option<String>,
}

impl Config {
  /// Load configuration from environment variables.
  pub fn from_env() -> Result<Self, ConfigError> {
    let database_url =
      std::env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL"))?;

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
      .unwrap_or_else(|_| "3000".to_string())
      .parse::<u16>()
      .map_err(|_| ConfigError::Invalid("PORT", "must be a valid u16"))?;

    let auth_enabled = std::env::var("AUTH_ENABLED")
      .unwrap_or_else(|_| "false".to_string())
      .parse::<bool>()
      .unwrap_or(false);

    let oidc_issuer_url = std::env::var("OIDC_ISSUER_URL").ok();
    let oidc_client_id = std::env::var("OIDC_CLIENT_ID").ok();
    let oidc_client_secret = std::env::var("OIDC_CLIENT_SECRET").ok();
    let oidc_redirect_uri = std::env::var("OIDC_REDIRECT_URI").ok();

    let jwt_secret = std::env::var("JWT_SECRET")
      .unwrap_or_else(|_| "dev-secret-change-me-in-production-32chars!".to_string());

    let frontend_url =
      std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let tenants_config = std::env::var("TENANTS_CONFIG").ok();
    let bim_validator_url = std::env::var("BIM_VALIDATOR_URL").ok();
    let bcf_platform_url = std::env::var("BCF_PLATFORM_URL").ok();

    if auth_enabled {
      if oidc_issuer_url.is_none() {
        return Err(ConfigError::Missing(
          "OIDC_ISSUER_URL (required when AUTH_ENABLED=true)",
        ));
      }
      if oidc_client_id.is_none() {
        return Err(ConfigError::Missing(
          "OIDC_CLIENT_ID (required when AUTH_ENABLED=true)",
        ));
      }
      if oidc_client_secret.is_none() {
        return Err(ConfigError::Missing(
          "OIDC_CLIENT_SECRET (required when AUTH_ENABLED=true)",
        ));
      }
      if oidc_redirect_uri.is_none() {
        return Err(ConfigError::Missing(
          "OIDC_REDIRECT_URI (required when AUTH_ENABLED=true)",
        ));
      }
    }

    Ok(Self {
      database_url,
      host,
      port,
      auth_enabled,
      oidc_issuer_url,
      oidc_client_id,
      oidc_client_secret,
      oidc_redirect_uri,
      jwt_secret,
      frontend_url,
      tenants_config,
      bim_validator_url,
      bcf_platform_url,
    })
  }
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
  #[error("missing required environment variable: {0}")]
  Missing(&'static str),
  #[error("invalid value for {0}: {1}")]
  Invalid(&'static str, &'static str),
}
