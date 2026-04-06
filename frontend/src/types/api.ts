export interface User {
  user_id: string;
  email: string;
  name: string;
  tenant: string;
  avatar_url?: string;
}

export interface Project {
  id: string;
  name: string;
  nextcloud_folder: string;
  description: string;
  status: string;
  created_at: string;
}

export interface CloudEntry {
  name: string;
  is_directory: boolean;
  size: number;
  last_modified: string;
  content_type?: string;
}

export interface DirectoryConfig {
  id: string;
  project_id: string;
  path: string;
  display_name: string;
  sort_order: number;
  icon: string;
  allowed_extensions: string[];
  read_only: boolean;
}

// ── Manifest / WEFC types ───────────────────────────────────

export interface ManifestInfo {
  name: string;
  size: number;
  last_modified: string;
}

export interface WefcHeader {
  schema: string;
  schemaVersion: string;
  fileId: string;
  description?: string;
  timestamp: string;
  application: string;
  applicationVersion?: string;
}

export interface WefcDataObject {
  type: string;
  guid: string;
  name: string;
  path: string;
  [key: string]: unknown;
}

export interface WefcManifest {
  header: WefcHeader;
  data: WefcDataObject[];
}
