//! API route definitions.

use axum::Router;

use crate::state::AppState;

pub mod auth_routes;
pub mod directories;
pub mod files;
pub mod health;
pub mod projects;

/// Build the complete API router.
pub fn api_router() -> Router<AppState> {
  Router::new()
    .merge(health::routes())
    .merge(auth_routes::routes())
    .nest("/api/v1", platform_routes())
}

fn platform_routes() -> Router<AppState> {
  Router::new()
    .nest("/projects", projects::routes())
    .nest("/projects", directories::routes())
    .nest("/projects", files::routes())
}
