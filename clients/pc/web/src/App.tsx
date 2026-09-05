import { HashRouter } from "react-router-dom";
import { AppRoutes } from "./router";
import { AppShell } from "./shell/AppShell";
import { SessionScope } from "./shell/SessionScope";

/**
 * Composition root. All structure lives in shell/, router.tsx, views/,
 * components/, state/, and data/. Logic modules (api, channels, session,
 * policy, probe, taskDraft, watch, watchSse, identities) are untouched.
 */
export function App() {
  return (
    <HashRouter>
      <SessionScope>
        <AppShell>
          <AppRoutes />
        </AppShell>
      </SessionScope>
    </HashRouter>
  );
}

/** Test-only: leftover Linux 1.0 Home/Work/Agents/Providers pages. Not product chrome. */
export function LinuxLegacyApp() {
  return (
    <HashRouter>
      <SessionScope>
        <AppShell>
          <AppRoutes includeRetiredLinux />
        </AppShell>
      </SessionScope>
    </HashRouter>
  );
}
