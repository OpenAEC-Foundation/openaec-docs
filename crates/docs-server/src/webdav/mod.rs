//! Enhanced Nextcloud WebDAV client for document management.
//!
//! Unlike the BCF Platform's tool-specific client, this supports arbitrary
//! path navigation across the entire project folder structure.

mod propfind;
pub mod types;

use reqwest::Client;

use crate::error::{AppError, AppResult};

use types::{CloudEntry, CloudProject, DavEntry};

/// Root folder on Nextcloud where all projects live.
const PROJECTS_ROOT: &str = "Projects";

/// Nextcloud WebDAV client.
#[derive(Debug, Clone)]
pub struct NextcloudClient {
  client: Client,
  webdav_root: String,
  username: String,
  password: String,
}

impl NextcloudClient {
  /// Create a new client.
  pub fn new(base_url: &str, username: &str, password: &str) -> Self {
    let base = base_url.trim_end_matches('/');
    let encoded_user = urlencoding::encode(username);
    Self {
      client: Client::new(),
      webdav_root: format!("{base}/remote.php/dav/files/{encoded_user}"),
      username: username.to_string(),
      password: password.to_string(),
    }
  }

  /// Test if Nextcloud is reachable.
  pub async fn test_connection(&self) -> AppResult<bool> {
    let url = format!("{}/{PROJECTS_ROOT}/", self.webdav_root);
    let resp = self
      .client
      .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
      .basic_auth(&self.username, Some(&self.password))
      .header("Depth", "0")
      .send()
      .await
      .map_err(|e| AppError::Nextcloud(format!("unreachable: {e}")))?;
    Ok(resp.status().is_success() || resp.status().as_u16() == 207)
  }

  /// List all project folders under Projects/.
  pub async fn list_projects(&self) -> AppResult<Vec<CloudProject>> {
    let url = format!("{}/{PROJECTS_ROOT}/", self.webdav_root);
    let entries = self.propfind(&url).await?;
    Ok(
      entries
        .into_iter()
        .filter(|e| e.is_collection)
        .map(|e| CloudProject { name: e.name })
        .collect(),
    )
  }

  /// List entries at an arbitrary path within a project.
  pub async fn list_path(&self, project: &str, path: &str) -> AppResult<Vec<CloudEntry>> {
    let full_path = self.project_path(project, path);
    let url = format!("{}/{full_path}/", self.webdav_root);
    let entries = self.propfind(&url).await;

    match entries {
      Ok(items) => Ok(
        items
          .into_iter()
          .map(|e| CloudEntry {
            name: e.name,
            is_directory: e.is_collection,
            size: e.size,
            last_modified: e.last_modified,
            content_type: e.content_type,
          })
          .collect(),
      ),
      Err(AppError::NotFound(_)) => Ok(vec![]),
      Err(e) => Err(e),
    }
  }

  /// Download a file at an arbitrary path within a project.
  pub async fn download_file(&self, project: &str, path: &str) -> AppResult<(Vec<u8>, String)> {
    let full_path = self.project_path(project, path);
    let url = format!("{}/{full_path}", self.webdav_root);

    let resp = self
      .client
      .get(&url)
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await
      .map_err(|e| AppError::Nextcloud(format!("download failed: {e}")))?;

    if resp.status().as_u16() == 404 {
      return Err(AppError::NotFound(format!("file not found: {path}")));
    }
    if !resp.status().is_success() {
      return Err(AppError::Nextcloud(format!("error: {}", resp.status())));
    }

    let content_type = resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("application/octet-stream")
      .to_string();

    let bytes = resp
      .bytes()
      .await
      .map(|b| b.to_vec())
      .map_err(|e| AppError::Nextcloud(format!("read error: {e}")))?;

    Ok((bytes, content_type))
  }

  /// Upload a file at an arbitrary path within a project.
  pub async fn upload_file(
    &self,
    project: &str,
    path: &str,
    data: Vec<u8>,
  ) -> AppResult<()> {
    // Ensure parent directories exist
    self.ensure_parent_dirs(project, path).await?;

    let full_path = self.project_path(project, path);
    let url = format!("{}/{full_path}", self.webdav_root);

    let resp = self
      .client
      .put(&url)
      .basic_auth(&self.username, Some(&self.password))
      .body(data)
      .send()
      .await
      .map_err(|e| AppError::Nextcloud(format!("upload failed: {e}")))?;

    let status = resp.status().as_u16();
    if status != 201 && status != 204 && !resp.status().is_success() {
      return Err(AppError::Nextcloud(format!("upload error: {status}")));
    }

    Ok(())
  }

  /// Delete a file or directory.
  pub async fn delete(&self, project: &str, path: &str) -> AppResult<()> {
    let full_path = self.project_path(project, path);
    let url = format!("{}/{full_path}", self.webdav_root);

    let resp = self
      .client
      .delete(&url)
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await
      .map_err(|e| AppError::Nextcloud(format!("delete failed: {e}")))?;

    if resp.status().as_u16() == 404 {
      return Err(AppError::NotFound(format!("not found: {path}")));
    }

    Ok(())
  }

  /// Create a directory (MKCOL).
  pub async fn mkdir(&self, project: &str, path: &str) -> AppResult<()> {
    let full_path = self.project_path(project, path);
    self.mkcol(&full_path).await
  }

  /// Build the URL-encoded path for a project subpath.
  fn project_path(&self, project: &str, subpath: &str) -> String {
    let safe_project = urlencoding::encode(project);
    let trimmed = subpath.trim_matches('/');
    if trimmed.is_empty() {
      format!("{PROJECTS_ROOT}/{safe_project}")
    } else {
      // Encode each path segment individually
      let encoded_segments: Vec<String> = trimmed
        .split('/')
        .map(|seg| urlencoding::encode(seg).into_owned())
        .collect();
      format!(
        "{PROJECTS_ROOT}/{safe_project}/{}",
        encoded_segments.join("/")
      )
    }
  }

  /// Ensure all parent directories exist for a given file path.
  async fn ensure_parent_dirs(&self, project: &str, file_path: &str) -> AppResult<()> {
    let trimmed = file_path.trim_matches('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() <= 1 {
      return Ok(()); // File is in project root
    }

    let safe_project = urlencoding::encode(project);
    let mut cumulative = format!("{PROJECTS_ROOT}/{safe_project}");

    // Create each directory segment (excluding the filename)
    for segment in &parts[..parts.len() - 1] {
      let encoded = urlencoding::encode(segment);
      cumulative = format!("{cumulative}/{encoded}");
      self.mkcol(&cumulative).await?;
    }

    Ok(())
  }

  /// Create a single directory via MKCOL.
  async fn mkcol(&self, path: &str) -> AppResult<()> {
    let url = format!("{}/{path}/", self.webdav_root);
    let resp = self
      .client
      .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await
      .map_err(|e| AppError::Nextcloud(format!("mkcol failed: {e}")))?;

    let status = resp.status().as_u16();
    // 201 = created, 405 = already exists — both OK
    if status != 201 && status != 405 && !resp.status().is_success() {
      return Err(AppError::Nextcloud(format!("mkcol {path} failed: {status}")));
    }

    Ok(())
  }

  /// Execute PROPFIND and parse the response.
  async fn propfind(&self, url: &str) -> AppResult<Vec<DavEntry>> {
    let resp = self
      .client
      .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), url)
      .basic_auth(&self.username, Some(&self.password))
      .header("Depth", "1")
      .send()
      .await
      .map_err(|e| AppError::Nextcloud(format!("unreachable: {e}")))?;

    if resp.status().as_u16() == 404 {
      return Err(AppError::NotFound("path not found on nextcloud".to_string()));
    }

    let body = resp
      .text()
      .await
      .map_err(|e| AppError::Nextcloud(format!("read error: {e}")))?;

    propfind::parse_propfind_xml(&body)
  }
}
