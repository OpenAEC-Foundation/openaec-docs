//! Project database queries.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::{
  CreateDirectoryConfigRequest, CreateProjectRequest, DirectoryConfigRow, ProjectRow,
  UpdateProjectRequest,
};

/// List all active projects for a tenant.
pub async fn list_by_tenant(pool: &PgPool, tenant: &str) -> Result<Vec<ProjectRow>, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "SELECT id, tenant, name, nextcloud_folder, description, status,
            created_by, created_at, updated_at
     FROM projects
     WHERE tenant = $1 AND status != 'archived'
     ORDER BY name",
  )
  .bind(tenant)
  .fetch_all(pool)
  .await
}

/// Find a project by ID (scoped to tenant).
pub async fn find_by_id(
  pool: &PgPool,
  project_id: Uuid,
  tenant: &str,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "SELECT id, tenant, name, nextcloud_folder, description, status,
            created_by, created_at, updated_at
     FROM projects
     WHERE id = $1 AND tenant = $2",
  )
  .bind(project_id)
  .bind(tenant)
  .fetch_optional(pool)
  .await
}

/// Create a new project.
pub async fn create(
  pool: &PgPool,
  tenant: &str,
  req: &CreateProjectRequest,
  created_by: Uuid,
) -> Result<ProjectRow, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "INSERT INTO projects (tenant, name, nextcloud_folder, description, created_by)
     VALUES ($1, $2, $3, $4, $5)
     RETURNING id, tenant, name, nextcloud_folder, description, status,
               created_by, created_at, updated_at",
  )
  .bind(tenant)
  .bind(&req.name)
  .bind(&req.nextcloud_folder)
  .bind(&req.description)
  .bind(created_by)
  .fetch_one(pool)
  .await
}

/// Update a project.
pub async fn update(
  pool: &PgPool,
  project_id: Uuid,
  tenant: &str,
  req: &UpdateProjectRequest,
) -> Result<Option<ProjectRow>, sqlx::Error> {
  sqlx::query_as::<_, ProjectRow>(
    "UPDATE projects
     SET name = COALESCE($3, name),
         description = COALESCE($4, description),
         status = COALESCE($5, status),
         updated_at = now()
     WHERE id = $1 AND tenant = $2
     RETURNING id, tenant, name, nextcloud_folder, description, status,
               created_by, created_at, updated_at",
  )
  .bind(project_id)
  .bind(tenant)
  .bind(&req.name)
  .bind(&req.description)
  .bind(&req.status)
  .fetch_optional(pool)
  .await
}

/// Delete (archive) a project.
pub async fn archive(
  pool: &PgPool,
  project_id: Uuid,
  tenant: &str,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query(
    "UPDATE projects SET status = 'archived', updated_at = now()
     WHERE id = $1 AND tenant = $2",
  )
  .bind(project_id)
  .bind(tenant)
  .execute(pool)
  .await?;
  Ok(result.rows_affected() > 0)
}

// ── Directory configurations ──────────────────────────────────

/// List directory configs for a project.
pub async fn list_directory_configs(
  pool: &PgPool,
  project_id: Uuid,
) -> Result<Vec<DirectoryConfigRow>, sqlx::Error> {
  sqlx::query_as::<_, DirectoryConfigRow>(
    "SELECT id, project_id, path, display_name, sort_order, icon,
            allowed_extensions, read_only
     FROM directory_configs
     WHERE project_id = $1
     ORDER BY sort_order, display_name",
  )
  .bind(project_id)
  .fetch_all(pool)
  .await
}

/// Add a directory config.
pub async fn create_directory_config(
  pool: &PgPool,
  project_id: Uuid,
  req: &CreateDirectoryConfigRequest,
) -> Result<DirectoryConfigRow, sqlx::Error> {
  let extensions = serde_json::to_value(&req.allowed_extensions).unwrap_or_default();
  sqlx::query_as::<_, DirectoryConfigRow>(
    "INSERT INTO directory_configs
       (project_id, path, display_name, sort_order, icon, allowed_extensions, read_only)
     VALUES ($1, $2, $3, $4, $5, $6, $7)
     RETURNING id, project_id, path, display_name, sort_order, icon,
               allowed_extensions, read_only",
  )
  .bind(project_id)
  .bind(&req.path)
  .bind(&req.display_name)
  .bind(req.sort_order)
  .bind(&req.icon)
  .bind(extensions)
  .bind(req.read_only)
  .fetch_one(pool)
  .await
}

/// Delete a directory config.
pub async fn delete_directory_config(
  pool: &PgPool,
  config_id: Uuid,
  project_id: Uuid,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query(
    "DELETE FROM directory_configs WHERE id = $1 AND project_id = $2",
  )
  .bind(config_id)
  .bind(project_id)
  .execute(pool)
  .await?;
  Ok(result.rows_affected() > 0)
}
