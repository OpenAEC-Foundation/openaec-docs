//! WebDAV types shared across the module.

use serde::Serialize;

/// A parsed entry from a WebDAV PROPFIND response.
#[derive(Debug, Clone)]
pub struct DavEntry {
  pub name: String,
  pub href: String,
  pub is_collection: bool,
  pub size: u64,
  pub last_modified: String,
  pub content_type: String,
  pub etag: String,
}

/// A project folder on Nextcloud.
#[derive(Debug, Serialize)]
pub struct CloudProject {
  pub name: String,
}

/// A file or directory entry for API responses.
#[derive(Debug, Serialize)]
pub struct CloudEntry {
  pub name: String,
  pub is_directory: bool,
  pub size: u64,
  pub last_modified: String,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub content_type: String,
}
