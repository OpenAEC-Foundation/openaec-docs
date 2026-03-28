import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "./components/shell/AppShell";
import { ProjectList } from "./pages/ProjectList";
import { ProjectView } from "./pages/ProjectView";

export default function App() {
  return (
    <BrowserRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<ProjectList />} />
          <Route path="/projects/:projectId" element={<ProjectView />} />
        </Routes>
      </AppShell>
    </BrowserRouter>
  );
}
