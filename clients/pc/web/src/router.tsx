import { Navigate, Route, Routes } from "react-router-dom";
import { NotFound } from "./shell/NotFound";
import { SessionGate } from "./shell/SessionGate";
import { SessionPage } from "./views/SessionPage";
import { CreateWizardPage } from "./views/opc/CreateWizardPage";
import { KnowledgePage } from "./views/opc/KnowledgePage";
import { ProjectsPage } from "./views/opc/ProjectsPage";
import { SettingsPage } from "./views/opc/SettingsPage";
import { TodayPage } from "./views/opc/TodayPage";
import { HomePage } from "./views/home/HomePage";
import { ProviderDetailPage } from "./views/providers/ProviderDetailPage";
import { ProvidersPage } from "./views/providers/ProvidersPage";
import { NewTaskPage } from "./views/work/NewTaskPage";
import { WorkPage } from "./views/work/WorkPage";
import { WorkDetailPage } from "./views/work/detail/WorkDetailPage";
import { AgentsPage } from "./views/agents/AgentsPage";
import { AgentDetailPage } from "./views/agents/AgentDetailPage";
import { ResourcesPage } from "./views/resources/ResourcesPage";
import { MemoryPage } from "./views/resources/MemoryPage";
import { SkillsPage } from "./views/resources/SkillsPage";
import { ToolsPage } from "./views/resources/ToolsPage";
import { ActivityPage } from "./views/activity/ActivityPage";
import { SystemPage } from "./views/system/SystemPage";

/**
 * Route map — Personal 2.0 L1 is Today / Projects / Knowledge; Settings is
 * chrome. `/projects/new` is the five-step create wizard (P12-T02), not L1.
 * Linux 1.0 Home remains at /home. Providers/Work/Agents/Resources/
 * Activity/System stay as real secondary routes. Bindings folded into
 * Providers; /tasks → /work. ⌘K is chrome, not a space. Team/Inbox/#/hitl
 * are not routes. HITL is the Projects canvas (`?preview=`), reached from
 * Today by deep link.
 */
export function AppRoutes() {
  return (
    <Routes>
      <Route path="/session" element={<SessionPage />} />
      <Route
        path="/"
        element={
          <SessionGate channel="management" title="Today">
            <TodayPage />
          </SessionGate>
        }
      />
      <Route
        path="/projects/new"
        element={
          <SessionGate channel="management" title="Create Project">
            <CreateWizardPage />
          </SessionGate>
        }
      />
      <Route
        path="/projects"
        element={
          <SessionGate channel="management" title="Projects">
            <ProjectsPage />
          </SessionGate>
        }
      />
      <Route
        path="/knowledge"
        element={
          <SessionGate channel="management" title="Knowledge">
            <KnowledgePage />
          </SessionGate>
        }
      />
      <Route
        path="/settings"
        element={
          <SessionGate channel="management" title="Settings">
            <SettingsPage />
          </SessionGate>
        }
      />
      <Route
        path="/home"
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
        path="/work/:taskRef"
        element={
          <SessionGate channel="task" title="Task">
            <WorkDetailPage />
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
            <AgentDetailPage />
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
      <Route path="/tasks" element={<Navigate to="/work" replace />} />
      <Route
        path="/activity"
        element={
          <SessionGate channel="management" title="Activity">
            <ActivityPage />
          </SessionGate>
        }
      />
      <Route
        path="/resources/memory"
        element={
          <SessionGate channel="management" title="Memory">
            <MemoryPage />
          </SessionGate>
        }
      />
      <Route
        path="/resources/skill"
        element={
          <SessionGate channel="management" title="Skills">
            <SkillsPage />
          </SessionGate>
        }
      />
      <Route
        path="/resources/tool"
        element={
          <SessionGate channel="management" title="Tools">
            <ToolsPage />
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
