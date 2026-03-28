//! Project CRUD routes.

use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::project::{CreateProjectRequest, ProjectResponse, UpdateProjectRequest};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/", get(list_projects).post(create_project))
    .route("/{id}", get(get_project).put(update_project).delete(archive_project))
}

/// GET /api/v1/projects
async fn list_projects(
  State(state): State<AppState>,
  auth: AuthUser,
) -> AppResult<Json<Vec<ProjectResponse>>> {
  let projects = db::projects::list_by_tenant(&state.pool, &auth.tenant).await?;
  Ok(Json(projects.into_iter().map(Into::into).collect()))
}

/// POST /api/v1/projects
async fn create_project(
  State(state): State<AppState>,
  auth: AuthUser,
  Json(req): Json<CreateProjectRequest>,
) -> AppResult<Json<ProjectResponse>> {
  if req.name.is_empty() || req.nextcloud_folder.is_empty() {
    return Err(AppError::BadRequest("name and nextcloud_folder are required".to_string()));
  }

  let project = db::projects::create(&state.pool, &auth.tenant, &req, auth.user_id).await?;
  Ok(Json(project.into()))
}

/// GET /api/v1/projects/{id}
async fn get_project(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(id): Path<Uuid>,
) -> AppResult<Json<ProjectResponse>> {
  let project = db::projects::find_by_id(&state.pool, id, &auth.tenant)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;
  Ok(Json(project.into()))
}

/// PUT /api/v1/projects/{id}
async fn update_project(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(id): Path<Uuid>,
  Json(req): Json<UpdateProjectRequest>,
) -> AppResult<Json<ProjectResponse>> {
  let project = db::projects::update(&state.pool, id, &auth.tenant, &req)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".to_string()))?;
  Ok(Json(project.into()))
}

/// DELETE /api/v1/projects/{id}
async fn archive_project(
  State(state): State<AppState>,
  auth: AuthUser,
  Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
  let archived = db::projects::archive(&state.pool, id, &auth.tenant).await?;
  if !archived {
    return Err(AppError::NotFound("project not found".to_string()));
  }
  Ok(Json(serde_json::json!({ "status": "archived" })))
}
