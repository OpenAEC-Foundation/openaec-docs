import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { projects as projectsApi, files as filesApi, directories as dirsApi, manifests as manifestsApi } from "../api/client";
import type { Project, CloudEntry, DirectoryConfig, ManifestInfo, WefcManifest } from "../types/api";

export function ProjectView() {
  const { projectId } = useParams<{ projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [entries, setEntries] = useState<CloudEntry[]>([]);
  const [dirConfigs, setDirConfigs] = useState<DirectoryConfig[]>([]);
  const [manifestList, setManifestList] = useState<ManifestInfo[]>([]);
  const [activeManifest, setActiveManifest] = useState<WefcManifest | null>(null);
  const [activeManifestName, setActiveManifestName] = useState("");
  const [currentPath, setCurrentPath] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!projectId) return;
    Promise.all([
      projectsApi.get(projectId),
      dirsApi.list(projectId),
      manifestsApi.list(projectId).catch(() => [] as ManifestInfo[]),
    ]).then(([proj, dirs, mans]) => {
      setProject(proj);
      setDirConfigs(dirs);
      setManifestList(mans);
    });
  }, [projectId]);

  useEffect(() => {
    if (!projectId) return;
    setLoading(true);
    filesApi
      .list(projectId, currentPath)
      .then(setEntries)
      .catch(() => setEntries([]))
      .finally(() => setLoading(false));
  }, [projectId, currentPath]);

  const navigateTo = (name: string) => {
    setCurrentPath(currentPath ? `${currentPath}/${name}` : name);
  };

  const navigateUp = () => {
    const parts = currentPath.split("/");
    parts.pop();
    setCurrentPath(parts.join("/"));
  };

  const pathParts = currentPath ? currentPath.split("/") : [];

  return (
    <div className="flex h-full">
      {/* Sidebar: Directory tree */}
      <div className="w-56 border-r border-border bg-concrete/50 p-3 overflow-y-auto shrink-0">
        <Link
          to="/"
          className="text-xs text-text-subtle hover:text-amber no-underline block mb-3"
        >
          &larr; Alle projecten
        </Link>
        {project && (
          <h3 className="text-sm font-semibold mb-3 truncate">{project.name}</h3>
        )}

        <button
          onClick={() => { setCurrentPath(""); setActiveManifest(null); setActiveManifestName(""); }}
          className={`w-full text-left text-xs px-2 py-1.5 rounded transition-colors ${
            currentPath === "" && !activeManifest
              ? "bg-amber/10 text-amber font-medium"
              : "text-text-muted hover:bg-white"
          }`}
        >
          &#x1F4C1; Projectroot
        </button>

        {dirConfigs.map((dir) => (
          <button
            key={dir.id}
            onClick={() => { setCurrentPath(dir.path); setActiveManifest(null); setActiveManifestName(""); }}
            className={`w-full text-left text-xs px-2 py-1.5 rounded transition-colors mt-0.5 ${
              currentPath === dir.path && !activeManifest
                ? "bg-amber/10 text-amber font-medium"
                : "text-text-muted hover:bg-white"
            }`}
          >
            &#x1F4C1; {dir.display_name}
          </button>
        ))}

        {manifestList.length > 0 && (
          <>
            <div className="mt-4 mb-1 text-[10px] uppercase tracking-wider text-text-subtle font-semibold">
              Manifesten
            </div>
            {manifestList.map((m) => (
              <button
                key={m.name}
                onClick={() => {
                  if (!projectId) return;
                  setActiveManifestName(m.name);
                  manifestsApi.get(projectId, m.name).then(setActiveManifest).catch(() => setActiveManifest(null));
                }}
                className={`w-full text-left text-xs px-2 py-1.5 rounded transition-colors mt-0.5 ${
                  activeManifestName === m.name
                    ? "bg-amber/10 text-amber font-medium"
                    : "text-text-muted hover:bg-white"
                }`}
              >
                &#x1F4DC; {m.name}
              </button>
            ))}
          </>
        )}
      </div>

      {/* Main content: File list or Manifest viewer */}
      <div className="flex-1 p-4 overflow-y-auto">
        {activeManifest ? (
          /* ── Manifest viewer ─────────────────────────────── */
          <div>
            <div className="flex items-center gap-2 mb-4">
              <button
                onClick={() => { setActiveManifest(null); setActiveManifestName(""); }}
                className="text-xs text-text-subtle hover:text-amber transition-colors"
              >
                &larr; Terug naar bestanden
              </button>
              <span className="text-xs text-text-subtle opacity-30">/</span>
              <span className="text-xs font-medium">{activeManifestName}</span>
            </div>

            {/* Header info */}
            <div className="bg-concrete/50 rounded-lg p-3 mb-4 text-xs space-y-1">
              <div className="font-semibold text-sm mb-2">Manifest header</div>
              <div><span className="text-text-subtle">Schema:</span> {activeManifest.header.schema} v{activeManifest.header.schemaVersion}</div>
              <div><span className="text-text-subtle">File ID:</span> <span className="font-mono text-[11px]">{activeManifest.header.fileId}</span></div>
              <div><span className="text-text-subtle">Applicatie:</span> {activeManifest.header.application}{activeManifest.header.applicationVersion ? ` (${activeManifest.header.applicationVersion})` : ""}</div>
              <div><span className="text-text-subtle">Laatst gewijzigd:</span> {activeManifest.header.timestamp}</div>
              {activeManifest.header.description && (
                <div><span className="text-text-subtle">Beschrijving:</span> {activeManifest.header.description}</div>
              )}
            </div>

            {/* Data objects */}
            <div className="text-sm font-semibold mb-2">
              Data objecten ({activeManifest.data.length})
            </div>
            {activeManifest.data.length === 0 ? (
              <div className="text-text-subtle text-sm">Geen objecten in dit manifest.</div>
            ) : (
              <div className="space-y-1">
                {activeManifest.data.map((obj, i) => (
                  <div
                    key={obj.guid || i}
                    className="bg-white border border-border rounded p-3 text-xs"
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <span className="bg-amber/10 text-amber px-1.5 py-0.5 rounded text-[10px] font-medium">
                        {obj.type || "unknown"}
                      </span>
                      <span className="font-medium text-sm">{obj.name || "Naamloos"}</span>
                    </div>
                    {obj.path && (
                      <div className="text-text-subtle font-mono text-[11px]">{obj.path}</div>
                    )}
                    {obj.guid && (
                      <div className="text-text-subtle font-mono text-[10px] mt-0.5">GUID: {obj.guid}</div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : (
          /* ── File browser ────────────────────────────────── */
          <>
            {/* Breadcrumb */}
            <div className="flex items-center gap-1 text-xs text-text-muted mb-4">
              <button
                onClick={() => setCurrentPath("")}
                className="hover:text-amber transition-colors"
              >
                {project?.name || "Project"}
              </button>
              {pathParts.map((part, i) => (
                <span key={i} className="flex items-center gap-1">
                  <span className="opacity-30">/</span>
                  <button
                    onClick={() =>
                      setCurrentPath(pathParts.slice(0, i + 1).join("/"))
                    }
                    className="hover:text-amber transition-colors"
                  >
                    {part}
                  </button>
                </span>
              ))}
            </div>

            {loading ? (
              <div className="text-text-subtle text-sm">Laden...</div>
            ) : entries.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-48 text-text-subtle">
                <span className="text-4xl opacity-20 mb-2">&#x1F4C2;</span>
                <p className="text-sm">Lege map</p>
              </div>
            ) : (
              <div className="space-y-0.5">
                {currentPath && (
                  <button
                    onClick={navigateUp}
                    className="flex items-center gap-3 w-full p-2 rounded hover:bg-concrete
                               transition-colors text-left text-sm text-text-muted"
                  >
                    <span className="text-lg">&#x2191;</span>
                    <span>..</span>
                  </button>
                )}
                {/* Directories first, then files */}
                {entries
                  .sort((a, b) => {
                    if (a.is_directory !== b.is_directory)
                      return a.is_directory ? -1 : 1;
                    return a.name.localeCompare(b.name);
                  })
                  .map((entry) => (
                    <button
                      key={entry.name}
                      onClick={() => entry.is_directory && navigateTo(entry.name)}
                      className={`flex items-center gap-3 w-full p-2 rounded transition-colors text-left text-sm
                        ${entry.is_directory ? "hover:bg-concrete cursor-pointer" : "hover:bg-concrete/50"}`}
                    >
                      <span className="text-lg w-6 text-center">
                        {entry.is_directory ? "&#x1F4C1;" : fileIcon(entry.name)}
                      </span>
                      <span className="flex-1 truncate">{entry.name}</span>
                      {!entry.is_directory && (
                        <span className="text-xs text-text-subtle">
                          {formatSize(entry.size)}
                        </span>
                      )}
                    </button>
                  ))}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

function fileIcon(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "ifc":
      return "&#x1F3D7;";
    case "pdf":
      return "&#x1F4C4;";
    case "dwg":
    case "dxf":
      return "&#x1F4D0;";
    case "jpg":
    case "jpeg":
    case "png":
      return "&#x1F5BC;";
    default:
      return "&#x1F4C4;";
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}
