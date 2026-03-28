interface StatusBarProps {
  projectCount?: number;
  fileCount?: number;
}

export function StatusBar({ projectCount, fileCount }: StatusBarProps) {
  return (
    <div
      className="flex items-center justify-between h-6 px-3 text-[11px] border-t border-border select-none"
      style={{ backgroundColor: "var(--brand-header-bg)", color: "var(--brand-header-text)" }}
    >
      <div className="flex items-center gap-4 opacity-70">
        {projectCount !== undefined && <span>{projectCount} projecten</span>}
        {fileCount !== undefined && <span>{fileCount} bestanden</span>}
      </div>
      <div className="flex items-center gap-1.5 opacity-70">
        <span
          className="w-2 h-2 rounded-full bg-success"
          title="Online"
        />
        <span>Verbonden</span>
      </div>
    </div>
  );
}
