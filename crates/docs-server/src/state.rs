//! Shared application state passed to all Axum handlers.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::auth::oidc::OidcClient;
use crate::config::Config;
use crate::routes::auth_routes::PendingAuth;
use crate::tenant::TenantRegistry;

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
  pub pool: PgPool,
  pub config: Arc<Config>,
  /// OIDC client (None when auth is disabled).
  pub oidc_client: Option<OidcClient>,
  /// Pending OIDC auth flows (csrf_token → PendingAuth).
  pub pending_auth: Arc<RwLock<HashMap<String, PendingAuth>>>,
  /// Multi-tenant Nextcloud registry.
  pub tenants: Arc<TenantRegistry>,
}

impl AppState {
  pub fn new(
    pool: PgPool,
    config: Config,
    oidc_client: Option<OidcClient>,
    tenants: TenantRegistry,
  ) -> Self {
    Self {
      pool,
      config: Arc::new(config),
      oidc_client,
      pending_auth: Arc::new(RwLock::new(HashMap::new())),
      tenants: Arc::new(tenants),
    }
  }
}
