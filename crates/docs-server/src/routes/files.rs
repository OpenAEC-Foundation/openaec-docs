//! File operations routes (WebDAV proxy).

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::webdav::types::CloudEntry;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/{project_id}/files", get(list_root))
    .route("/{project_id}/files/{*path}", get(get_entry))
    .route("/{project_id}/files/{*path}", put(upload_file))
    .route("/{project_id}/files/{*path}", delete(delete_entry))
    .route("/{project_id}/mkdir/{*path}", post(mkdir))
}

/// GET /api/v1/projects/{id}/files — List project root.
async fn list_root(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<CloudEntry>>> {
  let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;
  let entries = nc.list_path(&project.nextcloud_folder, "").await?;
  Ok(Json(entries))
}

/// GET /api/v1/projects/{id}/files/{path..}
/// If path is a directory → list contents. If file → download.
async fn get_entry(
  State(state): State<AppState>,
  auth: AuthUser,
  Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, AppError> {
  let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;

  // Try listing as directory first
  let entries = nc.list_path(&project.nextcloud_folder, &path).await?;
  if !entries.is_empty() || path.ends_with('/') {
    return Ok(Json(entries).into_response());
  }

  // If empty listing, it might be a file — try downloading
  let (bytes, content_type) = nc.download_file(&project.nextcloud_folder, &path).await?;

  let filename = path.rsplit('/').next().unwrap_or("download");
  let response = Response::builder()
    .header(header::CONTENT_TYPE, content_type)
    .header(
      header::CONTENT_DISPOSITION,
      format!("inline; filename=\"{filename}\""),
    )
    .body(Body::from(bytes))
    .map_err(|e| AppError::Internal(format!("response build error: {e}")))?;

  Ok(response)
}

/// PUT /api/v1/projects/{id}/files/{path..} — Upload file.
async fn upload_file(
  State(state): State<AppState>,
  auth: AuthUser,
  Path((project_id, path)): Path<(Uuid, String)>,
  body: axum::body::Bytes,
) -> AppResult<Json<serde_json::Value>> {
  let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;
  nc.upload_file(&project.nextcloud_folder, &path, body.to_vec())
    .await?;
  Ok(Json(serde_json::json!({ "status": "uploaded", "path": path })))
}

/// DELETE /api/v1/projects/{id}/files/{path..} — Delete file or directory.
async fn delete_entry(
  State(state): State<AppState>,
  auth: AuthUser,
  Path((project_id, path)): Path<(Uuid, String)>,
) -> AppResult<Json<serde_json::Value>> {
  let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;
  nc.delete(&project.nextcloud_folder, &path).await?;
  Ok(Json(serde_json::json!({ "status": "deleted" })))
}

/// POST /api/v1/projects/{id}/files/{path..}/mkdir — Create directory.
async fn mkdir(
  State(state): State<AppState>,
  auth: AuthUser,
  Path((project_id, path)): Path<(Uuid, String)>,
) -> AppResult<Json<serde_json::Value>> {
  let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;
  nc.mkdir(&project.nextcloud_folder, &path).await?;
  Ok(Json(serde_json::json!({ "status": "created", "path": path })))
}

/// Helper: resolve project + Nextcloud client for an authenticated request.
async fn resolve_project_nc(
  state: &AppState,
  auth: &AuthUser,
  project_id: Uuid,
) -> AppResult<(crate::webdav::NextcloudClient, crate::models::project::ProjectRow)> {
  let project = db::projects::find_by_id(&state.pool, project_id, &auth.tenant)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;

  let nc = state
    .tenants
    .nextcloud_client(&auth.tenant)
    .ok_or_else(|| {
      AppError::Nextcloud("cloud storage not configured for tenant".to_string())
    })?;

  Ok((nc, project))
}
