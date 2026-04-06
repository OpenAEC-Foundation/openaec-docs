//! Manifest (`.wefc`) routes — list, read, and upsert project manifests.
//!
//! Uses the existing [`NextcloudClient`] for WebDAV I/O and
//! [`openaec_cloud::ProjectManifest`] for parsing/serializing.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use openaec_cloud::{ManifestInfo, ProjectManifest};

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::webdav::NextcloudClient;

/// Manifest file extension used to filter `.wefc` files.
const WEFC_EXT: &str = ".wefc";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{project_id}/manifests", get(list_manifests))
        .route(
            "/{project_id}/manifests/{name}",
            get(get_manifest).put(upsert_manifest_object),
        )
}

/// GET /api/v1/projects/{id}/manifests — list all `.wefc` files in project root.
async fn list_manifests(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(project_id): Path<Uuid>,
) -> AppResult<Json<Vec<ManifestInfo>>> {
    let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;

    let entries = nc.list_path(&project.nextcloud_folder, "").await?;

    let manifests: Vec<ManifestInfo> = entries
        .into_iter()
        .filter(|e| !e.is_directory && e.name.ends_with(WEFC_EXT))
        .map(|e| ManifestInfo {
            name: e.name,
            size: e.size,
            last_modified: e.last_modified,
        })
        .collect();

    Ok(Json(manifests))
}

/// GET /api/v1/projects/{id}/manifests/{name} — read and parse a manifest.
async fn get_manifest(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((project_id, name)): Path<(Uuid, String)>,
) -> AppResult<Json<ProjectManifest>> {
    let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;

    let filename = ensure_wefc_ext(&name);
    let (bytes, _content_type) = nc
        .download_file(&project.nextcloud_folder, &filename)
        .await?;

    let manifest = ProjectManifest::from_bytes(&bytes).map_err(|e| {
        AppError::Internal(format!("manifest parse error: {e}"))
    })?;

    Ok(Json(manifest))
}

/// PUT /api/v1/projects/{id}/manifests/{name} — upsert an object in a manifest.
///
/// Reads the existing manifest (or creates a new one), merges the provided
/// data object (matched on `guid`), and writes back.
async fn upsert_manifest_object(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((project_id, name)): Path<(Uuid, String)>,
    Json(object): Json<serde_json::Value>,
) -> AppResult<Json<ProjectManifest>> {
    let (nc, project) = resolve_project_nc(&state, &auth, project_id).await?;

    let filename = ensure_wefc_ext(&name);

    // Validate: object must have a `guid` field
    if object.get("guid").and_then(|v| v.as_str()).is_none() {
        return Err(AppError::BadRequest(
            "object must contain a 'guid' field".to_string(),
        ));
    }

    // Read existing manifest or create a new one
    let mut manifest = match nc
        .download_file(&project.nextcloud_folder, &filename)
        .await
    {
        Ok((bytes, _)) => ProjectManifest::from_bytes(&bytes).map_err(|e| {
            AppError::Internal(format!("manifest parse error: {e}"))
        })?,
        Err(AppError::NotFound(_)) => ProjectManifest::new("openaec-docs"),
        Err(e) => return Err(e),
    };

    // Upsert the object
    manifest.add_or_update(object);

    // Serialize and write back
    let bytes = manifest
        .to_bytes()
        .map_err(|e| AppError::Internal(format!("manifest serialize error: {e}")))?;

    nc.upload_file(&project.nextcloud_folder, &filename, bytes)
        .await?;

    Ok(Json(manifest))
}

/// Ensure the filename ends with `.wefc`.
fn ensure_wefc_ext(name: &str) -> String {
    if name.ends_with(WEFC_EXT) {
        name.to_string()
    } else {
        format!("{name}{WEFC_EXT}")
    }
}

/// Helper: resolve project + Nextcloud client for an authenticated request.
async fn resolve_project_nc(
    state: &AppState,
    auth: &AuthUser,
    project_id: Uuid,
) -> AppResult<(NextcloudClient, crate::models::project::ProjectRow)> {
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
