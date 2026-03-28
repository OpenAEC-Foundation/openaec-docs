//! Project models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database row for a project.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectRow {
  pub id: Uuid,
  pub tenant: String,
  pub name: String,
  pub nextcloud_folder: String,
  pub description: String,
  pub status: String,
  pub created_by: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// API response for a project.
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
  pub id: Uuid,
  pub name: String,
  pub nextcloud_folder: String,
  pub description: String,
  pub status: String,
  pub created_at: DateTime<Utc>,
}

impl From<ProjectRow> for ProjectResponse {
  fn from(row: ProjectRow) -> Self {
    Self {
      id: row.id,
      name: row.name,
      nextcloud_folder: row.nextcloud_folder,
      description: row.description,
      status: row.status,
      created_at: row.created_at,
    }
  }
}

/// Request body for creating a project.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
  pub name: String,
  pub nextcloud_folder: String,
  #[serde(default)]
  pub description: String,
}

/// Request body for updating a project.
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
  pub name: Option<String>,
  pub description: Option<String>,
  pub status: Option<String>,
}

/// Directory configuration for a project.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DirectoryConfigRow {
  pub id: Uuid,
  pub project_id: Uuid,
  pub path: String,
  pub display_name: String,
  pub sort_order: i32,
  pub icon: String,
  pub allowed_extensions: serde_json::Value,
  pub read_only: bool,
}

/// Request body for adding a directory config.
#[derive(Debug, Deserialize)]
pub struct CreateDirectoryConfigRequest {
  pub path: String,
  pub display_name: String,
  #[serde(default)]
  pub sort_order: i32,
  #[serde(default = "default_icon")]
  pub icon: String,
  #[serde(default)]
  pub allowed_extensions: Vec<String>,
  #[serde(default)]
  pub read_only: bool,
}

fn default_icon() -> String {
  "folder".to_string()
}
