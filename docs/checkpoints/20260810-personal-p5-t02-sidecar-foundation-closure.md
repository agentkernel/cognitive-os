# P5-T02 Sidecar foundation closure

## Task boundary

- Task: `P5-T02`
- Slices: `D01-D03`
- Branch: `personal/P5-T02-sidecar-foundation`
- Lease: `lease/personal/P5-T02/sidecar-foundation`
- PR: `#181`
- Scope: daemon-private official Pi Agent registration, SidecarSession
  activation, epoch-fenced pause/resume/stop/recover, and non-mutating health
  observation only

## Acceptance mapping

- D01 registers an inactive `registered` AgentInstance from an active official
  Pi installation root, bound to exact package/adapter/protocol/policy digests,
  with fencing epoch seed `1` and zero capabilities. Install ≠ register ≠
  activate; no SidecarSession, process, Effect, or Task completion is created.
- D02 activates one current SidecarSession at a bumped fencing epoch, rejects
  protocol-digest drift and duplicate activation, and keeps process/capability
  claims at zero.
- D03 pauses by fencing and clearing the current SidecarSession pointer;
  resume and recover always create a new epoch-bound session; stop quiesces
  active/paused instances; health returns redacted non-mutating facts with
  `process_bound=false`. Stale-epoch lifecycle requests fail closed.
- Identity separation: AgentInstallation (P5-T01) ≠ AgentInstance ≠
  SidecarSession ≠ OS process; instance_id ≠ session_id; AgentExecution,
  PiSession, ProcessAttempt spawn, Effect, and Task completion remain absent
  and are not substituted by lifecycle or health facts.
- Management-session admin-cli callers cover `register`, `activate`,
  `agent-pause`, `agent-resume`, `agent-stop`, `agent-recover`, and
  `agent-health`.

## Failure-first coverage

- inactive root, adapter-digest mismatch, and duplicate current-root
  registration;
- unregistered root, protocol-digest mismatch, and duplicate activation;
- stale-epoch pause/resume/stop/recover;
- identity and zero-capability assertions on health and recover;
- management CLI lifecycle path with process_bound=false and zero
  capability/Effect/Task counters.

## Validation

- `cargo fmt --all`: passed locally.
- `git diff --check`: passed locally.
- `pnpm run check:consistency`: passed locally.
- Exact native Linux validation at `58ff0a723a8eae0f7fc89d9a99e9fdd55406aa92`:
  runtime focused suite 11/11, admin registration/lifecycle suite 1/1, and
  Clippy for runtime/store/admin-cli passed.
- Required Ubuntu and Windows CI for final implementation revision
  `58ff0a723a8eae0f7fc89d9a99e9fdd55406aa92`: run `31391916831` (closure docs
  may add a follow-on required CI run recorded in PROGRESS).
- Local Windows Rust build/test/Clippy were not run because the registered GNU
  linker failure makes that environment unsupported for Rust validation.

## Explicit non-claims

This closure does not claim OS process spawn or adoption, DaemonProcessSupervisor
production wiring, public AgentExecution/PiSession resources, capability grants,
Effects, Task completion, B09, Gate, release, or Profile. Process supervision
binding for managed-Pi spawn remains deferred to P5-T05/B09. Public
`agent-adapter-manifest` remains Lane-CTR only.
