//! OpenAEC Docs — BIM Document Management Platform.
//!
//! Starts the Axum web server with PostgreSQL, multi-tenant Nextcloud WebDAV,
//! and OIDC authentication.

mod auth;
mod config;
mod db;
mod error;
mod integrations;
mod models;
mod routes;
mod state;
mod tenant;
mod webdav;

use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::auth::oidc::OidcClient;
use crate::config::Config;
use crate::routes::api_router;
use crate::state::AppState;
use crate::tenant::TenantRegistry;

const MAX_DB_CONNECTIONS: u32 = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();

  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "docs_server=info,tower_http=info".into()),
    )
    .init();

  let config = Config::from_env()?;
  tracing::info!("starting docs-server on {}:{}", config.host, config.port);

  // Database
  let pool = PgPoolOptions::new()
    .max_connections(MAX_DB_CONNECTIONS)
    .connect(&config.database_url)
    .await?;

  tracing::info!("running database migrations");
  sqlx::migrate!("../../migrations").run(&pool).await?;
  tracing::info!("migrations complete");

  // OIDC
  let oidc_client = if config.auth_enabled {
    tracing::info!("auth enabled — discovering OIDC provider");
    let client = OidcClient::discover(
      config.oidc_issuer_url.as_deref().unwrap(),
      config.oidc_client_id.as_deref().unwrap(),
      config.oidc_client_secret.as_deref().unwrap(),
      config.oidc_redirect_uri.as_deref().unwrap(),
    )
    .await?;
    tracing::info!("OIDC provider discovered");
    Some(client)
  } else {
    tracing::info!("auth disabled — open access mode");
    None
  };

  // Multi-tenant Nextcloud
  let tenants = match &config.tenants_config {
    Some(path) => {
      tracing::info!(path = %path, "loading tenant configurations");
      TenantRegistry::load(path).unwrap_or_else(|e| {
        tracing::warn!("failed to load tenants: {e} — running without cloud storage");
        TenantRegistry::empty()
      })
    }
    None => {
      tracing::info!("no TENANTS_CONFIG set — running without cloud storage");
      TenantRegistry::empty()
    }
  };

  // Build application
  let state = AppState::new(pool, config.clone(), oidc_client, tenants);

  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  let static_dir = std::path::Path::new("/app/static");
  let app = if static_dir.exists() {
    tracing::info!("serving frontend from /app/static");
    let serve_dir = ServeDir::new(static_dir)
      .not_found_service(ServeFile::new(static_dir.join("index.html")));
    api_router()
      .fallback_service(serve_dir)
      .layer(cors)
      .layer(TraceLayer::new_for_http())
      .with_state(state)
  } else {
    tracing::info!("no static dir found, API-only mode");
    api_router()
      .layer(cors)
      .layer(TraceLayer::new_for_http())
      .with_state(state)
  };

  let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
  let listener = tokio::net::TcpListener::bind(addr).await?;
  tracing::info!("listening on {addr}");
  axum::serve(listener, app).await?;

  Ok(())
}
