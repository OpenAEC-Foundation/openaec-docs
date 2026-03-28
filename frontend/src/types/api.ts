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
