import { Route, Routes } from "react-router-dom";
import { NotFound } from "./shell/NotFound";
import { SessionGate } from "./shell/SessionGate";
import { SessionPage } from "./views/SessionPage";
import {
  BindingsPage,
  ProvidersPage,
  ProviderDetailPage,
  TasksPage,
} from "./views/legacy/legacyPages";
import {
  ActivityPage,
  AgentsPage,
  HomePage,
  ResourcesPage,
  SystemPage,
  WorkPage,
} from "./views/placeholders";

/**
 * Route map — docs/design/06. Seven frozen spaces + session (utility) +
 * designed 404. Legacy mutation-bearing pages stay reachable at their
 * current routes until their redesign wave (Bindings/Tasks — unlisted,
 * behavior verbatim). Read-only legacy pages are replaced by honest
 * placeholders now.
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
      <Route
        path="/bindings"
        element={
          <SessionGate channel="management" title="Agent Provider bindings">
            <BindingsPage />
          </SessionGate>
        }
      />
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
