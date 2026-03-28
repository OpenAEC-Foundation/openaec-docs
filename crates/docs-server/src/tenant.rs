//! Multi-tenant support.
//!
//! Reads `tenants.json` to discover Nextcloud instances per tenant.
//! Creates per-request NextcloudClient based on the authenticated user's tenant.

use std::collections::HashMap;

use serde::Deserialize;

use crate::webdav::NextcloudClient;

/// A single tenant's configuration (from tenants.json).
#[derive(Debug, Clone, Deserialize)]
pub struct TenantConfig {
  pub name: String,
  pub nextcloud_url: String,
  pub nextcloud_domain: String,
  pub service_user: String,
  pub service_pass_env: String,
}

/// Top-level tenants.json structure.
#[derive(Debug, Deserialize)]
struct TenantsFile {
  tenants: HashMap<String, TenantConfig>,
}

/// Resolved tenant with credentials loaded from environment.
#[derive(Debug, Clone)]
pub struct ResolvedTenant {
  pub slug: String,
  pub name: String,
  pub nextcloud_url: String,
  pub nextcloud_domain: String,
  pub service_user: String,
  pub service_pass: String,
}

/// Registry of all configured tenants.
#[derive(Debug)]
pub struct TenantRegistry {
  tenants: HashMap<String, ResolvedTenant>,
}

impl TenantRegistry {
  /// Load tenants from a JSON file, resolving service passwords from env vars.
  pub fn load(path: &str) -> Result<Self, TenantError> {
    let content = std::fs::read_to_string(path)
      .map_err(|e| TenantError::Config(format!("failed to read {path}: {e}")))?;

    let file: TenantsFile = serde_json::from_str(&content)
      .map_err(|e| TenantError::Config(format!("invalid tenants.json: {e}")))?;

    let mut tenants = HashMap::new();
    for (slug, config) in file.tenants {
      let service_pass = std::env::var(&config.service_pass_env).unwrap_or_default();
      if service_pass.is_empty() {
        tracing::warn!(
          tenant = %slug,
          env_var = %config.service_pass_env,
          "service password not set — cloud storage disabled for this tenant"
        );
      }
      tenants.insert(
        slug.clone(),
        ResolvedTenant {
          slug: slug.clone(),
          name: config.name,
          nextcloud_url: config.nextcloud_url,
          nextcloud_domain: config.nextcloud_domain,
          service_user: config.service_user,
          service_pass,
        },
      );
    }

    tracing::info!(count = tenants.len(), "loaded tenant configurations");
    Ok(Self { tenants })
  }

  /// Create an empty registry (for development without tenants.json).
  pub fn empty() -> Self {
    Self {
      tenants: HashMap::new(),
    }
  }

  /// Get a tenant by slug.
  pub fn get(&self, slug: &str) -> Option<&ResolvedTenant> {
    self.tenants.get(slug)
  }

  /// List all tenant slugs.
  pub fn slugs(&self) -> Vec<&str> {
    self.tenants.keys().map(|s| s.as_str()).collect()
  }

  /// Create a NextcloudClient for a specific tenant.
  pub fn nextcloud_client(&self, slug: &str) -> Option<NextcloudClient> {
    let tenant = self.tenants.get(slug)?;
    if tenant.service_pass.is_empty() {
      return None;
    }
    Some(NextcloudClient::new(
      &tenant.nextcloud_url,
      &tenant.service_user,
      &tenant.service_pass,
    ))
  }
}

/// Tenant configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
  #[error("tenant configuration error: {0}")]
  Config(String),
}
