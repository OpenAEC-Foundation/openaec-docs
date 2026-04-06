import type { User, Project, CloudEntry, DirectoryConfig, ManifestInfo, WefcManifest, WefcDataObject } from "../types/api";

const TOKEN_KEY = "docs_token";

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem(TOKEN_KEY);
  const headers: Record<string, string> = {
    ...((options.headers as Record<string, string>) || {}),
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }
  if (!(options.body instanceof FormData)) {
    headers["Content-Type"] = "application/json";
  }

  const resp = await fetch(path, { ...options, headers });
  if (!resp.ok) {
    const body = await resp.text();
    throw new ApiError(resp.status, body);
  }
  return resp.json();
}

export const auth = {
  loginUrl: () => "/auth/login",
  me: () => request<User>("/auth/me"),
};

export const projects = {
  list: () => request<Project[]>("/api/v1/projects"),
  get: (id: string) => request<Project>(`/api/v1/projects/${id}`),
  create: (data: { name: string; nextcloud_folder: string; description?: string }) =>
    request<Project>("/api/v1/projects", {
      method: "POST",
      body: JSON.stringify(data),
    }),
};

export const files = {
  list: (projectId: string, path = "") =>
    request<CloudEntry[]>(`/api/v1/projects/${projectId}/files${path ? `/${encodeURIComponent(path)}` : ""}`),
};

export const directories = {
  list: (projectId: string) =>
    request<DirectoryConfig[]>(`/api/v1/projects/${projectId}/directories`),
  scan: (projectId: string) =>
    request<{ path: string; suggested_display_name: string }[]>(
      `/api/v1/projects/${projectId}/directories/scan`,
      { method: "POST" }
    ),
};

export const manifests = {
  list: (projectId: string) =>
    request<ManifestInfo[]>(`/api/v1/projects/${projectId}/manifests`),
  get: (projectId: string, name: string) =>
    request<WefcManifest>(`/api/v1/projects/${projectId}/manifests/${encodeURIComponent(name)}`),
  upsert: (projectId: string, name: string, object: WefcDataObject) =>
    request<WefcManifest>(`/api/v1/projects/${projectId}/manifests/${encodeURIComponent(name)}`, {
      method: "PUT",
      body: JSON.stringify(object),
    }),
};
