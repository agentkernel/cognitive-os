import { Navigate, Route, Routes } from "react-router-dom";
import { NotFound } from "./shell/NotFound";
import { SessionGate } from "./shell/SessionGate";
import { SessionPage } from "./views/SessionPage";
import { HomePage } from "./views/home/HomePage";
import { TasksPage } from "./views/legacy/legacyPages";
import { ProviderDetailPage } from "./views/providers/ProviderDetailPage";
import { ProvidersPage } from "./views/providers/ProvidersPage";
import { NewTaskPage } from "./views/work/NewTaskPage";
import { WorkPage } from "./views/work/WorkPage";
import {
  ActivityPage,
  AgentsPage,
  ResourcesPage,
  SystemPage,
} from "./views/placeholders";

/**
 * Route map — docs/design/06. Seven frozen spaces + session (utility) +
 * designed 404. Home is the W3 attention surface; Providers is the W2
 * redesign (accounts master + five-section detail); Bindings folded into
 * Providers (DD-04) and redirects there. Work is the W4 inventory plus the
 * governed creation flow; there is deliberately no /work/:id route until W5
 * delivers a real detail view. The legacy governed-task page stays reachable
 * at /tasks until W5 migrates its watch/observation diagnostics.
 */
export function AppRoutes() {
  return (
    <Routes>
      <Route path="/session" element={<SessionPage />} />
      <Route
        path="/"
        element={
          <SessionGate channel="management" title="Home">
            <HomePage />
          </SessionGate>
        }
      />
      <Route
        path="/work"
        element={
          <SessionGate channel="task" title="Work">
            <WorkPage />
          </SessionGate>
        }
      />
      <Route
        path="/work/new"
        element={
          <SessionGate channel="task" title="New task">
            <NewTaskPage />
          </SessionGate>
        }
      />
      <Route
        path="/agents"
        element={
          <SessionGate channel="management" title="Agents">
            <AgentsPage />
          </SessionGate>
        }
      />
      <Route
        path="/agents/:id"
        element={
          <SessionGate channel="management" title="Agent detail">
            <AgentsPage />
          </SessionGate>
        }
      />
      <Route
        path="/providers"
        element={
          <SessionGate channel="management" title="Providers">
            <ProvidersPage />
          </SessionGate>
        }
      />
      <Route
        path="/providers/:id"
        element={
          <SessionGate channel="management" title="Provider account">
            <ProviderDetailPage />
          </SessionGate>
        }
      />
      <Route path="/bindings" element={<Navigate to="/providers" replace />} />
      <Route
        path="/tasks"
        element={
          <SessionGate channel="task" title="Tasks">
            <TasksPage />
          </SessionGate>
        }
      />
      <Route
        path="/activity"
        element={
          <SessionGate channel="management" title="Activity">
            <ActivityPage />
          </SessionGate>
        }
      />
      <Route
        path="/resources"
        element={
          <SessionGate channel="management" title="Resources">
            <ResourcesPage />
          </SessionGate>
        }
      />
      <Route
        path="/system"
        element={
          <SessionGate channel="management" title="System">
            <SystemPage />
          </SessionGate>
        }
      />
      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}
