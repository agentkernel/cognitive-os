# CognitiveOS Personal User Journeys

- Status: target Linux 1.0 journeys
- Product boundary: [Linux 1.0 scope](linux-1.0-scope.md)
- Resource semantics: [Cognitive resource model](cognitive-resource-model.md)
- Authority behavior: [Personal architecture](../../architecture/personal/README.md)

Each journey separates visible interaction from daemon authority facts. The
Shell and per-Agent sidecar may explain, translate, propose or observe. The
Rust daemon alone resolves identities, authorizes, persists, dispatches,
reconciles and accepts.

## 1. Install Personal and reach first conversation

1. User verifies and runs the Linux installer.
2. Rust verifies the signed bundle, stages immutable bytes, installs the one
   `cognitiveos-personal.service` user service and confirms numeric-loopback
   liveness/readiness.
3. `cognitive init` creates the data layout and selects an approved
   `SecretStore`: desktop Secret Service or the headless encrypted-vault path.
   It asks for the Provider key through hidden input only after the backend is
   ready.
4. Daemon discovers Provider models and persists a non-secret selected-model
   snapshot.
5. User selects a Standard Workspace. The preview states that Extended Home is
   empty until explicit paths and operations are added.
6. Product previews official Pi acquisition with exact package/version/source,
   network use, sidecar identity and no automatic runtime capability.
7. User admits acquisition. Daemon verifies identity/SRI/digests, commits the
   immutable installation, registration and sidecar binding, then activates a
   supervised instance.
8. Pi hosts the Shell under client credentials and sends model requests through
   the daemon Provider proxy.

**Identity shown:** package, installation, registration, sidecar, instance and
process are separate rows. The Shell session is separate from all of them.

**Failure exits:** invalid signature, unavailable or locked selected
`SecretStore`, incompatible
Node, package/sidecar drift, failed health or Provider probe fail closed with no
active registration.

**Secret/evidence:** key material stays in the selected approved `SecretStore`.
Evidence holds only redacted status, versions and digests.

### Headless and foreground variants

- A headless daemon with a locked vault starts in read-only diagnostic mode.
  The operator unlocks it over an SSH TTY, or explicitly configures unattended
  unlock with systemd encrypted credential material. Provider/user secret
  values never enter the service unit, credential, environment or argv.
- Foreground recovery uses the same artifact, data, authority service and
  `SecretStore` port. It does not start a second backend or bind a public API.
- If the selected secret backend cannot unlock, Provider and Agent execution
  stay unavailable while doctor/status and deterministic recovery remain
  usable.

## 2. Navigate and inspect cognitive resources

User asks: "What can my Agents use, what is running, and what is blocked?"

1. Pi emits an interpretation candidate.
2. Daemon resolves it as read-only management queries.
3. Shell presents the five top-level spaces:
   - Home;
   - Agents;
   - Tasks;
   - Resources with Memory, Skills, Tools and Context;
   - Activity with Run, Process, Effect and Evidence.
4. Family views include related Budget, Permission, Model, Artifact and Event
   facts without presenting them as extra resource families.
5. Unknown and not-run fields remain explicit; Pi does not fill them from
   conversation or process state.

No confirmation is required for authorized Tier 0 inspection. The management
bearer remains separate from the Task bearer.

`CognitiveResourceManifest`, when used, lists only what is discoverable for the
current `ActivityContext`. Discovering a name does not grant read or action.

## 3. Remember explicit knowledge

User asks: "Remember that this workspace uses the checked-in formatter for 90
days."

1. Daemon records the explicit request and creates a `MemoryCandidate` with
   workspace scope, purpose, provenance and requested expiry.
2. Admission validates permission, content bounds, conflict policy and current
   versions.
3. The preview shows candidate source, scope, purpose, conflict disposition and
   retention.
4. On admission, daemon creates a versioned `MemoryObject` and the linked
   `MemoryAdmissionDecision` in SQLite.
5. FTS5 and metadata indexes update as derived data.
6. Shell shows the object version, provenance, expiry and supported forget
   action.

Explicit `remember` is high-confidence user intent but does not bypass policy.

## 4. Review an Agent Memory proposal and forget Memory

1. During a Task, Pi proposes a reusable fact as a `MemoryCandidate`.
2. Daemon does not admit it automatically merely because the Agent is trusted
   for execution.
3. User or policy reviews purpose, scope, provenance and any conflicting
   version; daemon records admit, reject or keep-separate disposition.
4. Later, user asks to forget the admitted object.
5. Daemon expires access, records the forget transition and durable tombstone,
   then invalidates FTS, cache and active Context references.
6. A stale sidecar proposal cannot resurrect the forgotten version.

Embeddings, vector/graph retrieval and automatic extraction of full
conversation history are absent from this journey.

## 5. Import and govern a local Skill

User asks: "Import this local Skill and pin the exact revision."

1. Daemon previews the local source, normalized `SKILL.md`, bounded
   `resources/` and `scripts/`, digest and compatibility result.
2. Import creates an immutable package/revision. Editing the source later does
   not mutate that revision.
3. User can install, list, inspect, pin, enable, disable and remove it.
4. Enablement makes the revision eligible for Context selection under policy;
   it grants no Tool or other capability.
5. Skill instructions/resources can enter a Task's Context after authorization.
6. If a Skill script is needed, daemon resolves a registered Tool and previews
   that Tool operation. The Skill never executes the script directly.

An unknown script runner has dispatch count zero. No marketplace, automatic
download or Skill chaining occurs.

## 6. Create a governed Task and inspect its Context

1. User states a goal and desired outcome.
2. Daemon durably records raw intent before Pi interpretation.
3. Pi may propose targets and Context needs; daemon resolves exact identities.
4. Shell presents a server-issued canonical preview with Standard Workspace,
   any Extended Home entries, model, Pi instance/sidecar, Skill revisions,
   Tools, deadline/retry/step/cost bounds, permissions, external mutations and
   acceptance criteria.
5. User admits the exact digest. Stale versions or changed scope require a new
   preview.
6. Daemon creates Task/Loop and an epoch-fenced `AgentExecution`.
7. Daemon creates the Task's real `ContextRequest` and resolves its
   `ContextView` from Task/current state, Memory, Skill instructions/resources,
   Tool summaries, artifacts/evidence, workspace and explicit Task inputs.
8. Authorization/filtering occurs before deterministic priority and FTS
   ranking. A required unavailable source fails closed.
9. Shell shows selected source versions/digests and every omission, truncation,
   conflict, stale source or budget loss.
10. Refresh preserves unchanged stable prefix segments, binds a new view digest
    and exposes an explicit delta from the base digest.
11. Shell watches progress, budgets, Activity, Effects and evidence.
12. Independent verification decides completion.

This journey does not claim that a future `TaskContract` already contains
fixed resource refs/constraints, sidecar adapter identity or Context policy.
Those are future contract directions requiring separate contract work.

## 7. Use registered workspace Tools

User asks: "Patch these files and run the registered checks."

1. Daemon resolves workspace read/search/write/patch and bounded check Tool
   descriptors from the static registry.
2. Descriptor digest, current availability, canonical paths, arguments,
   working directory, timeout and output bounds appear in the preview.
3. Reversible Standard Workspace writes use a low-friction recovery journal
   with intended paths, before/after identity and rollback status.
4. The bounded check process runs under the registered descriptor. Process
   observations appear under Activity/Process.
5. Exit zero is evidence input, not automatic Task completion.

If a Tool is unknown, descriptor-drifted, disabled or quarantined, daemon
dispatches it zero times and explains the availability reason.

## 8. Fetch read-only information and perform an external mutation

### Read-only HTTP fetch

1. User asks for a document from an allowed URL.
2. Daemon resolves the static read-only fetch descriptor and applies origin,
   redirect, size, time and content bounds.
3. Fetch carries no ambient cookies, Provider credentials or arbitrary write
   method.
4. Result becomes a provenance-bound Context or Artifact input.

### External or irreversible operation

1. Shell previews exact target, Tool descriptor, capability, idempotency and
   rollback/reconcile expectations.
2. Daemon persists Intent and Effect before dispatch.
3. Sidecar or Tool adapter performs only the admitted request.
4. Daemon records receipt or unknown outcome and reconciles with the original
   identity.
5. Activity shows Effect and Evidence separately from Process.

A successful external response does not complete the Task.

## 9. Extend workspace access without ambient home access

User asks: "Also read this configuration file in my home directory."

1. Daemon resolves the requested path but does not grant the home directory.
2. Preview adds one Extended Home entry with purpose, allowed operation,
   expiry/retention and affected Task.
3. User grants the bounded entry.
4. Sidecar and Tool receive only that resolved path and operation.
5. A sibling path or write attempt fails closed and dispatches zero times.
6. Revocation invalidates subsequent Context refresh and Tool access.

The same profile may enable ordinary outbound network access. It never grants
Secret Store contents, SSH/GPG keys, browser credential/profile stores,
CognitiveOS authority/bootstrap data, Docker/system sockets, system
directories, privilege elevation, service management or package management.
Publication, repository push and other remote mutations remain separately
typed and governed.

## 10. Pause, resume and recover work

User asks: "Pause this Task after a safe checkpoint."

1. Shell resolves whether the target is Task, execution, instance, sidecar or
   process and shows the distinction.
2. Daemon disables new dispatch under a fresh control epoch.
3. Worker checkpoints bounded state and reconciles pending Effects.
4. Projection reports suspended, blocked or reconciliation-needed.
5. Resume rechecks current permission, budget, package/registration/sidecar
   identity and Context digest, then starts a fresh execution epoch.

After daemon, Agent, sidecar, process or network failure:

1. daemon reloads durable Task, scheduler, Memory/Context, registration and
   Effect facts;
2. stale leases, sidecars and executions are fenced;
3. unknown Effects are queried/reconciled with original idempotency keys;
4. current policy and Tool availability are reauthorized;
5. a compatible checkpoint and fresh Context delta are restored;
6. daemon resumes, replaces, suspends or quarantines with an explicit reason.

Killing a process is not a pause protocol, cancellation or authority success.
Process remains observation/supervision data, not a new domain.

## 11. Upgrade, roll back or uninstall Pi

1. User asks to upgrade Pi.
2. Preview shows old/new package, installation and sidecar identities, npm
   source/digests, compatibility, affected instances/Tasks, permissions and
   rollback.
3. Daemon acquires a new immutable installation without changing the active
   registration.
4. Health and sidecar checks run with no inherited Provider secret.
5. Daemon fences/migrates affected executions and atomically activates the new
   registration binding.
6. Failed activation restores the prior complete binding; incomplete rollback
   remains a visible durable failure.

Uninstall previews package bytes, registration, sidecar, instances, Tasks,
pending Effects, capability leases and retained data. Daemon blocks new
dispatch, fences work, reconciles/quarantines Effects and then removes the
installation binding. Task history, Memory, Skills, evidence, Provider
configuration and secrets remain unless separate retention or purge policy
applies.

Pi is the only Linux 1.0 qualification. The journey cannot be relabeled for
another Agent without independent evidence.

## 12. Upgrade, roll back or uninstall Personal

Desktop, headless and foreground operation use the same signed artifact,
daemon and application services.

- Product upgrade verifies and stages that artifact, restarts the canonical
  service, confirms pointer/unit/process/liveness consistency and then issues a
  receipt.
- Failure restores prior binary, unit, pointer and database compatibility.
- Product uninstall previews service, binaries and retained data separately;
  deleting user data is a Tier 2 operation.

No mode introduces a second database writer or alternate authority backend.
Removing Personal never prints or exports secret material.

## 13. Backup and restore

1. `cognitive backup` previews included authority data, Memory, Skill registry,
   Task, Context metadata, runtime registrations and evidence while explicitly
   excluding secrets.
2. Daemon quiesces required writes and produces versioned, digest-bound backup
   metadata.
3. Restore verifies integrity, compatibility, tombstones and migrations before
   dispatch is re-enabled.
4. Derived FTS indexes can be rebuilt from SQLite authority data.
5. User rebinds required secrets through the selected approved `SecretStore`
   prompt or headless TTY unlock path.

An archive containing Provider keys is a critical failure, not a convenience.

## 14. Diagnose and obtain support

`cognitive doctor --bundle` gathers redacted platform, service, database,
SecretStore backend/locked state, Provider model, Standard Workspace/Extended Home policy, Pi
installation/registration/sidecar/instance, Tool availability, Context loss,
Task, Process, Effect and recovery facts. It provides stable errors, evidence
digests and next actions without prompts, raw Provider traffic, key material,
SecretRefs or sensitive SQLite/Memory/Context content.

Support output describes what is known, unknown and not-run. It never converts
a local smoke result into a Gate, release or Profile claim.

## 15. Linux and hardware evolution non-journey

Users do not install a Personal kernel module, eBPF control plane, device
scheduler or distributed authority for Linux 1.0. Linux service, filesystem,
process, secret, network and future hardware integration remain behind bounded
software ports. Hardware acceleration cannot bypass daemon authorization, CAS,
budget, Intent/Effect or acceptance.
