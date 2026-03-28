import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { projects as projectsApi } from "../api/client";
import type { Project } from "../types/api";

export function ProjectList() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    projectsApi
      .list()
      .then(setProjects)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <span className="text-text-subtle">Projecten laden...</span>
      </div>
    );
  }

  if (projects.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-3">
        <span className="text-5xl opacity-20">&#x1F4C2;</span>
        <p className="text-text-muted">Geen projecten gevonden</p>
        <p className="text-text-subtle text-sm">
          Maak een project aan of koppel een Nextcloud projectmap
        </p>
      </div>
    );
  }

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Projecten</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {projects.map((project) => (
          <Link
            key={project.id}
            to={`/projects/${project.id}`}
            className="block p-5 bg-white rounded-lg border border-border hover:border-amber/50
                       hover:shadow-md transition-all group no-underline"
          >
            <div className="flex items-start justify-between mb-2">
              <h3 className="font-semibold text-text group-hover:text-amber transition-colors">
                {project.name}
              </h3>
              <span
                className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                  project.status === "active"
                    ? "bg-success/10 text-success"
                    : "bg-scaffold-gray/10 text-scaffold-gray"
                }`}
              >
                {project.status}
              </span>
            </div>
            {project.description && (
              <p className="text-sm text-text-muted line-clamp-2 mb-3">
                {project.description}
              </p>
            )}
            <div className="flex items-center gap-2 text-xs text-text-subtle">
              <span>&#x1F4C1;</span>
              <span>{project.nextcloud_folder}</span>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
