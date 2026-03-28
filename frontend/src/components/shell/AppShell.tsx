import { useCallback, useEffect, useState } from "react";
import { AppBar } from "./AppBar";
import { Ribbon } from "./Ribbon";
import { StatusBar } from "./StatusBar";
import { Backstage } from "./Backstage";

interface AppShellProps {
  children: React.ReactNode;
  projectCount?: number;
  fileCount?: number;
}

export function AppShell({ children, projectCount, fileCount }: AppShellProps) {
  const [showBackstage, setShowBackstage] = useState(false);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape" && showBackstage) {
        setShowBackstage(false);
      }
    },
    [showBackstage]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div className="h-screen flex flex-col">
      <AppBar />
      <Ribbon onOpenBackstage={() => setShowBackstage(true)} />

      <main className="flex-1 overflow-auto">{children}</main>

      <StatusBar projectCount={projectCount} fileCount={fileCount} />

      {showBackstage && <Backstage onClose={() => setShowBackstage(false)} />}
    </div>
  );
}
