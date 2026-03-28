//! Directory configuration routes.

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::project::{CreateDirectoryConfigRequest, DirectoryConfigRow};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route(
      "/{project_id}/directories",
      get(list_directories).post(create_directory),
    )
    .route(
      "/{project_id}/directories/{dir_id}",
      delete(delete_directory),
    )
    .route("/{project_id}/directories/scan", post(scan_directories))
}

/// GET /api/v1/projects/{project_id}/directories
async fn list_directories(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<DirectoryConfigRow>>> {
  // Verify project access
  db::projects::find_by_id(&state.pool, project_id, &auth.tenant)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

  let configs = db::projects::list_directory_configs(&state.pool, project_id).await?;
  Ok(Json(configs))
}

/// POST /api/v1/projects/{project_id}/directories
async fn create_directory(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(project_id): Path<Uuid>,
  Json(req): Json<CreateDirectoryConfigRequest>,
) -> AppResult<Json<DirectoryConfigRow>> {
  let project = db::projects::find_by_id(&state.pool, project_id, &auth.tenant)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

  if req.path.is_empty() || req.display_name.is_empty() {
    return Err(AppError::BadRequest(
      "path and display_name are required".to_string(),
    ));
  }

  let config =
    db::projects::create_directory_config(&state.pool, project.id, &req).await?;
  Ok(Json(config))
}

/// DELETE /api/v1/projects/{project_id}/directories/{dir_id}
async fn delete_directory(
  State(state): State<AppState>,
  auth: AuthUser,
  Path((project_id, dir_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
  db::projects::find_by_id(&state.pool, project_id, &auth.tenant)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

  let deleted = db::projects::delete_directory_config(&state.pool, dir_id, project_id).await?;
  if !deleted {
    return Err(AppError::NotFound("directory config not found".to_string()));
  }
  Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/// POST /api/v1/projects/{project_id}/directories/scan
/// Scan the Nextcloud project folder and suggest directory configs.
async fn scan_directories(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<SuggestedDirectory>>> {
  let project = db::projects::find_by_id(&state.pool, project_id, &auth.tenant)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

  let nc = state
    .tenants
    .nextcloud_client(&auth.tenant)
    .ok_or_else(|| AppError::Nextcloud("cloud storage not configured for tenant".to_string()))?;

  let entries = nc.list_path(&project.nextcloud_folder, "").await?;

  let suggestions: Vec<SuggestedDirectory> = entries
    .into_iter()
    .filter(|e| e.is_directory)
    .map(|e| SuggestedDirectory {
      path: e.name.clone(),
      suggested_display_name: humanize_dir_name(&e.name),
    })
    .collect();

  Ok(Json(suggestions))
}

#[derive(Debug, serde::Serialize)]
struct SuggestedDirectory {
  path: String,
  suggested_display_name: String,
}

/// Convert directory names like "00_BIM" to "BIM" or "99_overige_documenten" to "Overige documenten".
fn humanize_dir_name(name: &str) -> String {
  // Strip leading number prefix (e.g., "00_", "01_")
  let stripped = if name.len() > 3 && name[..2].chars().all(|c| c.is_ascii_digit()) && name.as_bytes()[2] == b'_' {
    &name[3..]
  } else {
    name
  };

  // Replace underscores with spaces and capitalize first letter
  let humanized = stripped.replace('_', " ");
  let mut chars = humanized.chars();
  match chars.next() {
    None => String::new(),
    Some(first) => first.to_uppercase().to_string() + chars.as_str(),
  }
}
